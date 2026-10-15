//! Document file I/O for every frontend (desktop, web, CLI, control channel, headless MCP): open,
//! save, export and serialize by path or bytes (base64), the format table, and `command.batch`.
//!
//! - `load`: `document.open` (native, legacy, SVG/SVGZ, PDF/.ai/.ait, DXF, raster images).
//! - `encode`: one encoder per writable format, with typed options parsed from the params.
//! - `svg`: the SVG Options (styling, fonts, images, object ids, artboards…).
//! - `export`: `document.export` / `serialize` / `exportSelection` / `exportForOffice`.
//! - `screens`: Export for Screens (`document.exportForScreens`, `document.exportSettings`); `zip`
//!   packs its files as one download.
//! - `save`: `document.save`, Save As / a Copy / as Template, New from Template, Revert and the
//!   per-format options (`file.formatOptions`); [`save_with`] is the one save path of every frontend.
//! - `pdf`: PDF settings and presets for every PDF export, `document.exportPdf`.
//! - `pdfimport`: the PDF pages, box and password `document.open` and Place read, `document.pdfInfo`.
//! - `dxf`: DXF export options, `document.exportDxf`, and the formats that can't be written (DWG, PICT).
//! - `eps`: EPS export options (flattened transparency, previews, the embedded document), `document.exportEps`.
//! - `dxfimport`: the DXF options `document.open` and Place read, `document.dxfInfo`.
//! - `metafile`: EMF and WMF export, open and place.
//!
//! [`FORMATS`] is the single list of formats (append-only); open dialogs use [`open_filters`],
//! agents query `document.formats`.

mod affinity;
mod batch;
mod cdr;
pub mod dxf;
pub mod dxfimport;
mod encode;
pub mod eps;
mod export;
pub(crate) mod imagemap;
mod load;
mod metafile;
mod native;
pub mod pdf;
mod pdfimport;
pub mod pngtext;
pub mod ppi;
pub mod psdread;
mod save;
mod screens;
mod svg;
mod text;
mod zip;

use serde_json::{Value, json};

pub use dxf::{UNSUPPORTED, Unsupported, unsupported};
pub use encode::{ARTBOARD_PARAMS, ArtboardPick, Encoded, encode, encode_all, encode_with_warnings};
pub(crate) use encode::{anti_alias, background, with_single_artboard};
pub use export::export_source;
pub(crate) use export::isolated;
pub(crate) use load::check_not_lossy_overwrite;
use load::err;
pub(crate) use load::import_svg;
pub(crate) use load::native_file;
pub(crate) use load::source;
pub use load::{
    Loaded, RasterImage, ReadFile, detect, file_name, load, load_with, open_bytes, open_bytes_with, open_read, open_template, raster_image,
    read_for_open,
};
pub use load::{losses_summary, overwrite_losses};
pub(crate) use native::preview_png;
pub use native::with_compression_pref;
pub use pdfimport::{LoadOptions, page_document};
pub(crate) use save::job_for;
pub use save::{
    SAVE_FORMATS, SaveJob, SaveMode, SavePlan, StreamWrite, export_folder, save_filter…14837 tokens truncated…ok_or("truncated ZIP entry name")?).map_err(|e| e.to_string())?.to_string();
        if name.starts_with('/') || name.contains('\\') || name.split('/').any(|p| p == "..") {
            return Err("unsafe path in CDR package".into());
        }
        out.push(Entry { name, flags, method, crc, compressed_size, uncompressed_size, local_offset });
        at = name_end.checked_add(extra_len).and_then(|n| n.checked_add(comment_len)).ok_or("ZIP entry length overflow")?;
        if at > dir_end {
            return Err("ZIP entry extends past central directory".into());
        }
    }
    if at != dir_end {
        return Err("ZIP central directory size does not match its entries".into());
    }
    Ok(out)
}

/// Read a single file by its central-directory name with a strict expansion bound and CRC check.
pub(super) fn read_entry(zip: &[u8], wanted: &str, max_bytes: usize) -> Result<Vec<u8>, String> {
    let entries = directory(zip, 20_000, 512 * 1024 * 1024)?;
    let e = entries.iter().find(|e| e.name == wanted).ok_or_else(|| format!("ZIP entry {wanted} not found"))?;
    if e.uncompressed_size > max_bytes as u64 {
        return Err(format!("ZIP entry {wanted} exceeds its size limit"));
    }
    let local = usize::try_from(e.local_offset).map_err(|_| "ZIP local offset overflow")?;
    let header_end = local.checked_add(30).ok_or("ZIP local header offset overflow")?;
    if header_end > zip.len() {
        return Err(format!("{wanted}: truncated ZIP local header"));
    }
    let sig = zip.get(local..local.checked_add(4).ok_or("ZIP local offset overflow")?).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if sig != Some(0x0403_4b50) {
        return Err(format!("{wanted}: invalid ZIP local header"));
    }
    let u16_at = |at: usize| zip.get(at..at.checked_add(2)?).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let flags = u16_at(local + 6).ok_or("truncated ZIP local header")?;
    let method = u16_at(local + 8).ok_or("truncated ZIP local header")?;
    if flags != e.flags || method != e.method {
        return Err(format!("{wanted}: local and central ZIP headers differ"));
    }
    let name_len = u16_at(local + 26).ok_or("truncated ZIP local header")? as usize;
    let extra_len = u16_at(local + 28).ok_or("truncated ZIP local header")? as usize;
    let name_start = header_end;
    let name_end = name_start.checked_add(name_len).ok_or("ZIP local name offset overflow")?;
    if zip.get(name_start..name_end) != Some(wanted.as_bytes()) {
        return Err(format!("{wanted}: local and central ZIP names differ"));
    }
    let data = name_end.checked_add(extra_len).ok_or("ZIP data offset overflow")?;
    let comp_len = usize::try_from(e.compressed_size).map_err(|_| "ZIP compressed size overflow")?;
    let compressed = zip.get(data..data.checked_add(comp_len).ok_or("ZIP data size overflow")?).ok_or("truncated ZIP entry data")?;
    let mut out = Vec::with_capacity(e.uncompressed_size as usize);
    match e.method {
        0 => out.extend_from_slice(compressed),
        8 => {
            DeflateDecoder::new(compressed).take(max_bytes as u64 + 1).read_to_end(&mut out).map_err(|e| e.to_string())?;
        }
        _ => return Err("unsupported ZIP compression".into()),
    };
    if out.len() > max_bytes || out.len() as u64 != e.uncompressed_size {
        return Err(format!("{wanted}: invalid expanded size"));
    }
    let mut c = Crc::new();
    c.update(&out);
    if c.sum() != e.crc {
        return Err(format!("{wanted}: CRC mismatch"));
    }
    Ok(out)
}

/// The DOS date of every entry: 1980-01-01, the earliest a ZIP date can say (no clock on the web,
/// and a fixed date keeps archives of the same files identical).
const DOS_DATE: u16 = (1 << 5) | 1;
/// General purpose flag: file names are UTF-8.
const UTF8_NAMES: u16 = 1 << 11;
/// Version needed to extract: 2.0.
const VERSION: u16 = 20;

/// The archive of `files` (`(name, bytes)`; `/` separates folders in a name), stored without
/// compression. Fails past the limits of a ZIP without its 64-bit extension (65535 files, 4 GB).
pub fn store<N: AsRef<str>, B: AsRef<[u8]>>(files: &[(N, B)]) -> Result<Vec<u8>, String> {
    let too_big = || "too much to zip: at most 65535 files and 4 GB".to_string();
    let count = u16::try_from(files.len()).map_err(|_| too_big())?;
    let mut out = Vec::with_capacity(files.iter().map(|(n, b)| 76 + 2 * n.as_ref().len() + b.as_ref().len()).sum::<usize>() + 22);
    let mut central = Vec::new();
    for (name, bytes) in files {
        let (name, bytes) = (name.as_ref().as_bytes(), bytes.as_ref());
        let offset = u32::try_from(out.len()).map_err(|_| too_big())?;
        let size = u32::try_from(bytes.len()).map_err(|_| too_big())?;
        let name_len = u16::try_from(name.len()).map_err(|_| format!("file name too long to zip: {}", String::from_utf8_lossy(name)))?;
        let mut crc = Crc::new();
        crc.update(bytes);
        // The fields both headers share: flags, method (0: stored), time, date, CRC, sizes, name length.
        let mut shared = Vec::with_capacity(26);
        for v in [UTF8_NAMES, 0, 0, DOS_DATE] {
            shared.extend_from_slice(&v.to_le_bytes());
        }
        for v in [crc.sum(), size, size] {
            shared.extend_from_slice(&v.to_le_bytes());
        }
        shared.extend_from_slice(&name_len.to_le_bytes());
        // Local file header.
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&shared);
        out.extend_from_slice(&0u16.to_le_bytes()); // extra field length
        out.extend_from_slice(name);
        out.extend_from_slice(bytes);
        // Central directory record.
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&VERSION.to_le_bytes()); // made by
        central.extend_from_slice(&VERSION.to_le_bytes()); // needed to extract
        central.extend_from_slice(&shared);
        // Extra and comment lengths, disk number, internal and external attributes.
        central.extend_from_slice(&[0; 2 + 2 + 2 + 2 + 4]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let start = u32::try_from(out.len()).map_err(|_| too_big())?;
    let central_len = u32::try_from(central.len()).map_err(|_| too_big())?;
    out.extend_from_slice(&central);
    // End of central directory.
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]); // this disk, the directory's disk
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&central_len.to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length
    Ok(out)
}

/// The entries of a store-only archive written by [`store`] (read through its central directory,
/// checking every CRC) → `(name, bytes)`.
#[cfg(test)]
pub fn entries(zip: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let u16_at = |at: usize| zip.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or("truncated");
    let u32_at = |at: usize| zip.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or("truncated");
    let end = zip.len().checked_sub(22).ok_or("no end record")?;
    if u32_at(end)? != 0x0605_4b50 {
        return Err("no end record".into());
    }
    let count = u16_at(end + 10)? as usize;
    let mut at = u32_at(end + 16)? as usize;
    let mut out = vec![];
    for _ in 0..count {
        if u32_at(at)? != 0x0201_4b50 || u16_at(at + 10)? != 0 {
            return Err("not a stored central record".into());
        }
        let (crc, size, name_len) = (u32_at(at + 16)?, u32_at(at + 20)? as usize, u16_at(at + 28)? as usize);
        let extra = u16_at(at + 30)? as usize + u16_at(at + 32)? as usize;
        let local = u32_at(at + 42)? as usize;
        let name = String::from_utf8(zip.get(at + 46..at + 46 + name_len).ok_or("truncated")?.to_vec()).map_err(|e| e.to_string())?;
        if u32_at(local)? != 0x0403_4b50 {
            return Err(format!("{name}: no local header"));
        }
        let data = local + 30 + u16_at(local + 26)? as usize + u16_at(local + 28)? as usize;
        let bytes = zip.get(data..data + size).ok_or("truncated")?.to_vec();
        let mut c = Crc::new();
        c.update(&bytes);
        if c.sum() != crc {
            return Err(format!("{name}: CRC mismatch"));
        }
        out.push((name, bytes));
        at += 46 + name_len + extra;
    }
    Ok(out)
}
