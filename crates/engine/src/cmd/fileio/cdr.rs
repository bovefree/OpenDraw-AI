//! Safe first-pass support for modern ZIP-based CorelDRAW documents.
//!
//! Until a clean-room RIFF object reader is available, Open shows the archive's page previews
//! and clearly labels the result as a preview. This is deliberately not presented as editable art.

use std::io::Cursor;

use vectorcraft_doc::{Document, ImageBlob, ImageObject, Node, NodeKind};
use vectorcraft_geom::Affine;

use super::Result;
use super::load::err;
use super::zip::read_entry;

const MAX_ENTRIES: usize = 20_000;
const MAX_UNCOMPRESSED: u64 = 512 * 1024 * 1024;
const MAX_PREVIEW: u64 = 64 * 1024 * 1024;
const MAX_PIXELS: u64 = 32 * 1024 * 1024;
const MAX_TOTAL_PIXELS: u64 = 64 * 1024 * 1024;

pub(super) struct Imported {
    pub doc: Document,
    pub warnings: Vec<String>,
}

/// Identify a ZIP-based CDR by its MIME entry or the CDR signature in its native root stream.
pub(super) fn is_cdr(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04") && read_entry(bytes, "mimetype", 1024).is_ok_and(|m| m == b"application/x-vnd.corel.zcf.draw.document+zip")
        || bytes.starts_with(b"PK\x03\x04") && read_entry(bytes, "content/root.dat", 2 * 1024 * 1024).is_ok_and(|r| r.get(8..12) == Some(b"CDRU"))
}

pub(super) fn import(name: &str, bytes: &[u8]) -> Result<Imported> {
    let entries = super::zip::directory(bytes, MAX_ENTRIES, MAX_UNCOMPRESSED).map_err(err)?;
    entries.iter().find(|e| e.name == "mimetype").ok_or_else(|| err("CDR package has no mimetype entry"))?;
    if read_entry(bytes, "mimetype", 1024).map_err(err)? != b"application/x-vnd.corel.zcf.draw.document+zip" {
        return Err(err("unsupported CDR package type"));
    }
    let root = entries.iter().find(|e| e.name == "content/root.dat").ok_or_else(|| err("CDR package has no content/root.dat"))?;
    if root.uncompressed_size > 16 * 1024 * 1024 || read_entry(bytes, "content/root.dat", 16 * 1024 * 1024).map_err(err)?.get(8..12) != Some(b"CDRU")
    {
        return Err(err("unrecognized CDR root stream"));
    }
    let mut previews = entries.iter().filter(|e| e.name.starts_with("previews/page") && e.name.ends_with(".png")).collect::<Vec<_>>();
    previews.sort_by_key(|e| natural_page_number(&e.name));
    if previews.is_empty() {
        return Err(err("this CDR has no page preview; vector object import is not available for this file yet"));
    }
    if previews.len() > 500 {
        return Err(err("this CDR has more than 500 page previews; refusing to allocate them"));
    }

    let mut doc = Document::new(0.0, 0.0);
    doc.title = name.to_string();
    let mut boards = Vec::new();
    let layer = doc.layers.first().map(|n| n.id).ok_or_else(|| err("new document has no layer"))?;
    let mut total_pixels = 0u64;
    let mut x = 0.0;
    for (i, entry) in previews.iter().enumerate() {
        if entry.uncompressed_size > MAX_PREVIEW {
            return Err(err(format!("CDR page preview {} exceeds the 64 MiB safety limit", i + 1)));
        }
        let png = read_entry(bytes, &entry.name, MAX_PREVIEW as usize).map_err(err)?;
        let image = decode_preview(&png)?;
        let (w, h) = image.dimensions();
        let pixels = u64::from(w) * u64::from(h);
        if pixels > MAX_PIXELS {
            return Err(err(format!("CDR page preview {} exceeds the pixel safety limit", i + 1)));
        }
        total_pixels = total_pixels.checked_add(pixels).ok_or_else(|| err("CDR preview pixel count overflow"))?;
        if total_pixels > MAX_TOTAL_PIXELS {
            return Err(err("CDR page previews exceed the 64-megapixel total safety limit"));
        }
        let mut encoded = Vec::new();
        image.write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png).map_err(err)?;
        let blob = ImageBlob::new("image/png", encoded);
        let key = blob.content_key();
        let id = doc.alloc_id();
        let xf = Affine::translate((x, 0.0));
        let node = Node::new(
            id,
            NodeKind::Image(Box::new(ImageObject { key: key.clone(), width: w, height: h, xf, link: None, placement: Default::default() })),
        );
        let rect = vectorcraft_geom::Rect::new(x, 0.0, x + f64::from(w), f64::from(h));
        boards.push(vectorcraft_doc::Artboard {
            id: i as u32 + 1,
            name: format!("Page {} (preview)", i + 1),
            rect,
            show_center_mark: false,
            show_cross_hairs: false,
        });
        let index = doc.layers.first().and_then(|n| n.children()).map_or(0, |children| children.len());
        doc.insert(Some(layer), index, node).map_err(err)?;
        doc.images.insert(key, blob);
        x += f64::from(w) + 36.0;
    }
    doc.artboards = boards;
    Ok(Imported {
        doc,
        warnings: vec!["CDR opened as page previews only. Vectors, editable text, layers, fills and effects have not been imported; do not save over the source. Full CDR object import is still in development.".into()],
    })
}

fn natural_page_number(name: &str) -> u32 {
    let digits = name.rsplit('/').next().unwrap_or_default().trim_start_matches("page").trim_end_matches(".png");
    digits.parse().unwrap_or(u32::MAX)
}

fn decode_preview(bytes: &[u8]) -> Result<image::RgbaImage> {
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map(|i| i.to_rgba8()).map_err(err)
}
