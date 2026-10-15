//! Synthetic CDR package tests; no private CorelDRAW files are stored in the repository.

use proptest::prelude::*;
use serde_json::json;
use vectorcraft_doc::NodeKind;

use super::*;

fn package(pages: usize) -> Vec<u8> {
    let mut files = vec![
        ("mimetype".to_string(), b"application/x-vnd.corel.zcf.draw.document+zip".to_vec()),
        ("content/root.dat".to_string(), b"RIFF\x04\0\0\0CDRU".to_vec()),
    ];
    for i in 1..=pages {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(3, 2, image::Rgba([60, 120, 180, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        files.push((format!("previews/page{i}.png"), png));
    }
    zip::store(&files).unwrap()
}

fn package_with_external_reference(stream_name: &str) -> Vec<u8> {
    fn chunk(tag: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(body);
        if body.len() & 1 != 0 {
            out.push(0);
        }
        out
    }
    let mut reference = Vec::new();
    reference.extend_from_slice(&0u32.to_le_bytes()); // stream index
    reference.extend_from_slice(&4u32.to_le_bytes()); // body length
    reference.extend_from_slice(&4u32.to_le_bytes()); // body offset
    reference.extend_from_slice(&0u32.to_le_bytes()); // reserved
    let record = chunk(b"loda", &reference);
    let mut page = b"page".to_vec();
    page.extend_from_slice(&record);
    let page = chunk(b"LIST", &page);
    let mut root = b"RIFF".to_vec();
    root.extend_from_slice(&((4 + page.len()) as u32).to_le_bytes());
    root.extend_from_slice(b"CDRU");
    root.extend_from_slice(&page);

    let mut png = Vec::new();
    image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    let files: Vec<(String, Vec<u8>)> = vec![
        ("mimetype".into(), b"application/x-vnd.corel.zcf.draw.document+zip".to_vec()),
        ("content/root.dat".into(), root),
        ("content/dataFileList.dat".into(), format!("{stream_name}\n").into_bytes()),
        ("content/data/page1.dat".into(), vec![0; 8]),
        ("previews/page1.png".into(), png),
    ];
    zip::store(&files).unwrap()
}

#[test]
fn cdr_is_detected_by_package_contents_and_lists_supported_extension() {
    let bytes = package(1);
    assert_eq!(detect("drawing.bin", &bytes).map(|f| f.id), Some("cdr"));
    assert_eq!(detect("drawing.bin", b"not a cdr").map(|f| f.id), None);
    assert!(OPEN_EXTS.contains(&"cdr") && PLACE_EXTS.contains(&"cdr"));
    assert!(format("cdr").is_some_and(|f| f.read && !f.write));
}

#[test]
fn opens_each_page_as_a_preview_artboard_with_clear_loss_warning() {
    let bytes = package(2);
    let loaded = load("sample.cdr", &bytes).unwrap();
    assert!(loaded.preview_only);
    assert_eq!(loaded.doc.artboards.len(), 2);
    assert_eq!(loaded.doc.node_count(), 3, "two page images and one default layer");
    assert!(loaded.warnings[0].contains("page previews only"));
    let mut s = Session::new();
    let r = s.execute("document.open", &json!({"name":"sample.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)})).unwrap();
    assert_eq!(r["format"], "cdr");
    assert_eq!(r["warnings"][0], loaded.warnings[0]);
    assert!(s.doc().unwrap().path.is_none(), "CDR imports must not save over source");

    let mut from_path = Session::new();
    open_bytes(&mut from_path, "sample.cdr", &bytes, Some("sample.cdr".into())).unwrap();
    let state = from_path.doc().unwrap();
    assert_eq!(state.imported_from.as_deref(), Some("sample.cdr"));
    assert!(check_not_lossy_overwrite(state, "sample.cdr", &json!({}), "document.save").is_err());
}

#[test]
fn page_preview_names_use_real_page_numbers_and_keep_each_preview_scale() {
    let mut large = Vec::new();
    image::RgbaImage::from_pixel(12, 20, image::Rgba([60, 120, 180, 255]))
        .write_to(&mut std::io::Cursor::new(&mut large), image::ImageFormat::Png)
        .unwrap();
    let mut small = Vec::new();
    image::RgbaImage::from_pixel(3, 4, image::Rgba([90, 120, 150, 255]))
        .write_to(&mut std::io::Cursor::new(&mut small), image::ImageFormat::Png)
        .unwrap();
    let files: Vec<(String, Vec<u8>)> = vec![
        ("mimetype".into(), b"application/x-vnd.corel.zcf.draw.document+zip".to_vec()),
        ("content/root.dat".into(), b"RIFF\x04\0\0\0CDRU".to_vec()),
        ("previews/page3.png".into(), large),
        ("previews/page4.png".into(), small),
    ];
    let loaded = load("numbered.cdr", &zip::store(&files).unwrap()).unwrap();
    assert_eq!(loaded.doc.artboards.iter().map(|b| (b.id, b.name.as_str())).collect::<Vec<_>>(), [(3, "Page 3 (preview)"), (4, "Page 4 (preview)")]);
    assert_eq!(loaded.doc.artboards[0].rect.height(), 20.0);
    assert_eq!(loaded.doc.artboards[1].rect.height(), 4.0);
    let dimensions = loaded.doc.layers[0]
        .children()
        .unwrap()
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Image(image) => Some((image.width, image.height)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(dimensions, [(12, 20), (3, 4)]);
}

#[test]
fn malformed_or_non_cdr_zip_is_rejected_without_panicking() {
    assert!(load("bad.cdr", b"PK\x03\x04garbage").is_err());
    assert!(!cdr::is_cdr(b"PK\x03\x04garbage"));
    let mut bytes = package(1);
    bytes[0] ^= 1;
    assert!(load("bad.cdr", &bytes).is_err());
}

#[test]
fn root_inventory_walks_lists_and_rejects_invalid_lengths() {
    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        if data.len() & 1 != 0 {
            out.push(0);
        }
        out
    }
    fn list(kind: &[u8; 4], children: &[u8]) -> Vec<u8> {
        let mut body = kind.to_vec();
        body.extend_from_slice(children);
        chunk(b"LIST", &body)
    }
    let nested = list(b"page", &list(b"layr", &list(b"obj ", &[])));
    let mut root = b"RIFF".to_vec();
    root.extend_from_slice(&((4 + nested.len()) as u32).to_le_bytes());
    root.extend_from_slice(b"CDRU");
    root.extend_from_slice(&nested);
    assert_eq!(
        cdr::inspect_root(&root).unwrap(),
        cdr::RootInventory { version: None, pages: 1, layers: 1, objects: 1, groups: 0, external_references: 0, external_bytes: 0 }
    );

    root[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(cdr::inspect_root(&root).is_err());
}

#[test]
fn root_inventory_reads_modern_cdr_version_from_local_record() {
    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        out
    }
    let mut fver = Vec::new();
    fver.extend_from_slice(&u32::MAX.to_le_bytes());
    fver.extend_from_slice(&8u32.to_le_bytes());
    fver.extend_from_slice(&2700u16.to_le_bytes());
    fver.extend_from_slice(&1u16.to_le_bytes());
    fver.extend_from_slice(&0u16.to_le_bytes());
    fver.extend_from_slice(&27u16.to_le_bytes());
    let version = chunk(b"fver", &fver);
    let page = chunk(b"LIST", b"page");
    let mut root = b"RIFF".to_vec();
    root.extend_from_slice(&((4 + version.len() + page.len()) as u32).to_le_bytes());
    root.extend_from_slice(b"CDRU");
    root.extend_from_slice(&version);
    root.extend_from_slice(&page);

    assert_eq!(cdr::inspect_root(&root).unwrap().version, Some(2700));
}

#[test]
fn root_inventory_validates_external_stream_ranges() {
    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        out
    }
    let mut reference = Vec::new();
    reference.extend_from_slice(&0u32.to_le_bytes());
    reference.extend_from_slice(&4u32.to_le_bytes());
    reference.extend_from_slice(&4u32.to_le_bytes());
    reference.extend_from_slice(&0u32.to_le_bytes());
    let record = chunk(b"loda", &reference);
    let mut page = b"page".to_vec();
    page.extend_from_slice(&record);
    let page = chunk(b"LIST", &page);
    let mut root = b"RIFF".to_vec();
    root.extend_from_slice(&((4 + page.len()) as u32).to_le_bytes());
    root.extend_from_slice(b"CDRU");
    root.extend_from_slice(&page);

    let streams = [cdr::DataStream { name: "page1.dat".into(), size: 8 }];
    assert_eq!(
        cdr::inspect_root_with_streams(&root, &streams).unwrap(),
        cdr::RootInventory { version: None, pages: 1, layers: 0, objects: 0, groups: 0, external_references: 1, external_bytes: 4 }
    );
    let short_stream = [cdr::DataStream { name: "page1.dat".into(), size: 7 }];
    assert!(cdr::inspect_root_with_streams(&root, &short_stream).is_err());
    assert!(cdr::inspect_root_with_streams(&root, &[]).is_err());
}

#[test]
fn cdr_package_resolves_ordered_external_stream_members() {
    let bytes = package_with_external_reference("page1.dat");
    let loaded = load("external.cdr", &bytes).unwrap();
    assert_eq!(loaded.doc.artboards.len(), 1);
    assert!(loaded.warnings[0].contains("1 references to 4 bytes"));

    let missing = package_with_external_reference("missing.dat");
    assert!(load("missing.cdr", &missing).is_err());

    let blank_line = package_with_external_reference("page1.dat\n");
    assert!(load("blank-line.cdr", &blank_line).is_err());
}

#[test]
fn places_a_selected_cdr_page_as_a_warned_raster_preview() {
    let bytes = package(2);
    let mut s = Session::new();
    s.execute("document.open", &json!({"name":"sample.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)})).unwrap();
    let before = s.doc().unwrap().doc.node_count();
    let placed = s.execute("file.place", &json!({"name":"sample.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes), "page":2})).unwrap();
    assert_eq!(placed["format"], "cdr");
    assert!(placed["warnings"][0].as_str().unwrap().contains("page previews only"));
    assert!(s.doc().unwrap().doc.node_count() > before);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn mutated_cdr_packages_never_panic(cut in 0usize..4096, edits in prop::collection::vec((0usize..4096, any::<u8>()), 0..12)) {
        let mut bytes = package(2);
        for (at, value) in edits {
            let len = bytes.len();
            if let Some(byte) = bytes.get_mut(at % len) {
                *byte = value;
            }
        }
        bytes.truncate(cut);
        let _ = load("mutated.cdr", &bytes);
    }
}
