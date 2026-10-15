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
const MAX_RIFF_CHUNKS: usize = 50_000;
const MAX_RIFF_DEPTH: usize = 128;
const MAX_STREAM_LIST: usize = 1024 * 1024;
const MAX_DATA_STREAMS: usize = 4096;

pub(super) struct Imported {
    pub doc: Document,
    pub warnings: Vec<String>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub(super) struct RootInventory {
    pub version: Option<u16>,
    pub pages: usize,
    pub layers: usize,
    pub objects: usize,
    pub groups: usize,
    pub external_references: usize,
    pub external_bytes: u64,
}

#[derive(Clone, Debug)]
pub(super) struct DataStream {
    pub name: String,
    pub size: u64,
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
    if root.uncompressed_size > 16 * 1024 * 1024 {
        return Err(err("unrecognized CDR root stream"));
    }
    let root_bytes = read_entry(bytes, "content/root.dat", 16 * 1024 * 1024).map_err(err)?;
    let streams = data_streams(bytes, &entries)?;
    let inventory = inspect_root_with_streams(&root_bytes, &streams)?;
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
        warnings: vec![format!(
            "CDR opened as page previews only. The native root identifies version {} and contains {} page records, {} layers, {} objects and {} groups; {} references to {} bytes of external drawing data were validated, but their artwork is not imported yet. Vectors, editable text, fills and effects are unavailable; do not save over the source. Full CDR object import is still in development.",
            inventory.version.map_or_else(|| "unknown".to_string(), |v| format!("{}.{}", v / 100, v % 100)),
            inventory.pages,
            inventory.layers,
            inventory.objects,
            inventory.groups,
            inventory.external_references,
            inventory.external_bytes
        )],
    })
}

/// Validate and inventory the standard RIFF hierarchy in a modern CDR root stream. This is a
/// bounded structural pass only: artwork payloads live in separate page streams and are not
/// interpreted here.
#[cfg(test)]
pub(super) fn inspect_root(data: &[u8]) -> Result<RootInventory> {
    inspect_root_with_optional_streams(data, None)
}

pub(super) fn inspect_root_with_streams(data: &[u8], streams: &[DataStream]) -> Result<RootInventory> {
    inspect_root_with_optional_streams(data, Some(streams))
}

fn inspect_root_with_optional_streams(data: &[u8], streams: Option<&[DataStream]>) -> Result<RootInventory> {
    if data.get(..4) != Some(b"RIFF") || data.get(8..12) != Some(b"CDRU") {
        return Err(err("unrecognized CDR root stream"));
    }
    let declared = u32::from_le_bytes(data.get(4..8).ok_or_else(|| err("truncated CDR RIFF header"))?.try_into().map_err(err)?) as usize;
    let end = declared.checked_add(8).ok_or_else(|| err("CDR RIFF size overflow"))?;
    if end > data.len() || end < 12 {
        return Err(err("CDR RIFF length exceeds the root stream"));
    }
    let mut inventory = RootInventory::default();
    let mut chunks = 0usize;
    walk_riff(data, 12, end, 0, &mut chunks, &mut inventory, streams)?;
    Ok(inventory)
}

fn walk_riff(
    data: &[u8],
    start: usize,
    end: usize,
    depth: usize,
    chunks: &mut usize,
    inventory: &mut RootInventory,
    streams: Option<&[DataStream]>,
) -> Result<()> {
    if depth > MAX_RIFF_DEPTH {
        return Err(err("CDR RIFF nesting exceeds the safety limit"));
    }
    let mut pos = start;
    while pos < end {
        *chunks = chunks.checked_add(1).ok_or_else(|| err("CDR RIFF chunk count overflow"))?;
        if *chunks > MAX_RIFF_CHUNKS {
            return Err(err("CDR RIFF has too many chunks"));
        }
        let header_end = pos.checked_add(8).ok_or_else(|| err("CDR RIFF offset overflow"))?;
        if header_end > end {
            return Err(err("truncated CDR RIFF chunk header"));
        }
        let tag = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..header_end].try_into().map_err(err)?) as usize;
        let payload_end = header_end.checked_add(size).ok_or_else(|| err("CDR RIFF chunk size overflow"))?;
        if payload_end > end {
            return Err(err("CDR RIFF chunk exceeds its parent"));
        }
        if tag == b"LIST" {
            if size < 4 {
                return Err(err("CDR RIFF LIST chunk has no list type"));
            }
            let kind = &data[header_end..header_end + 4];
            match kind {
                b"page" => inventory.pages += 1,
                b"layr" => inventory.layers += 1,
                b"obj " => inventory.objects += 1,
                b"grp " | b"lnkg" => inventory.groups += 1,
                _ => {}
            }
            walk_riff(data, header_end + 4, payload_end, depth + 1, chunks, inventory, streams)?;
        } else if size == 16 {
            // X6+ stores most record payloads in external streams. The wrapper contains a stream
            // index, byte length and offset; the fourth word is reserved by this container.
            let stream_index = u32::from_le_bytes(data[header_end..header_end + 4].try_into().map_err(err)?);
            let length = u32::from_le_bytes(data[header_end + 4..header_end + 8].try_into().map_err(err)?) as u64;
            let offset = u32::from_le_bytes(data[header_end + 8..header_end + 12].try_into().map_err(err)?) as u64;
            if tag == b"fver" && stream_index == u32::MAX {
                // In the current ZIP container, a local (non-external) record keeps its body
                // after the 8-byte stream/length prefix. fver starts with the numeric CDR version.
                let local_end = header_end
                    .checked_add(8)
                    .and_then(|start| start.checked_add(length as usize))
                    .ok_or_else(|| err("CDR version record range overflow"))?;
                if length < 2 || local_end > payload_end {
                    return Err(err("truncated CDR version record"));
                }
                inventory.version = Some(u16::from_le_bytes(data[header_end + 8..header_end + 10].try_into().map_err(err)?));
            }
            if stream_index != u32::MAX {
                inventory.external_references =
                    inventory.external_references.checked_add(1).ok_or_else(|| err("CDR external reference count overflow"))?;
                inventory.external_bytes = inventory.external_bytes.checked_add(length).ok_or_else(|| err("CDR external byte count overflow"))?;
                if let Some(streams) = streams {
                    let stream =
                        streams.get(stream_index as usize).ok_or_else(|| err(format!("CDR record references missing data stream {stream_index}")))?;
                    let reference_end = offset.checked_add(length).ok_or_else(|| err("CDR external reference range overflow"))?;
                    if reference_end > stream.size {
                        return Err(err(format!("CDR record reference exceeds data stream {}", stream.name)));
                    }
                }
            }
        }
        pos = payload_end.checked_add(size & 1).ok_or_else(|| err("CDR RIFF offset overflow"))?;
        if pos > end {
            return Err(err("truncated CDR RIFF chunk padding"));
        }
    }
    Ok(())
}

/// Resolve the ordered external-stream table from `content/dataFileList.dat`, validating every
/// listed ZIP member before any page data is followed.
fn data_streams(bytes: &[u8], entries: &[super::zip::Entry]) -> Result<Vec<DataStream>> {
    let Some(list) = entries.iter().find(|e| e.name == "content/dataFileList.dat") else {
        return Ok(Vec::new());
    };
    if list.uncompressed_size > MAX_STREAM_LIST as u64 {
        return Err(err("CDR external stream list exceeds the 1 MiB safety limit"));
    }
    let raw = read_entry(bytes, "content/dataFileList.dat", MAX_STREAM_LIST).map_err(err)?;
    let text = std::str::from_utf8(&raw).map_err(err)?;
    let mut streams = Vec::new();
    for line in text.split_terminator('\n') {
        let name = line.strip_suffix('\r').unwrap_or(line);
        if name.is_empty() {
            return Err(err("empty name in CDR external stream list"));
        }
        if streams.len() >= MAX_DATA_STREAMS {
            return Err(err("CDR has more than 4096 external data streams"));
        }
        if name.len() > 255 || name == "." || name == ".." || !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')) {
            return Err(err("unsafe name in CDR external stream list"));
        }
        if streams.iter().any(|s: &DataStream| s.name == name) {
            return Err(err("duplicate name in CDR external stream list"));
        }
        let member = format!("content/data/{name}");
        let entry = entries.iter().find(|e| e.name == member).ok_or_else(|| err(format!("CDR external stream {name} is missing")))?;
        streams.push(DataStream { name: name.to_string(), size: entry.uncompressed_size });
    }
    Ok(streams)
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
