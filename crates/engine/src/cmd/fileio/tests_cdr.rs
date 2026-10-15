//! Synthetic CDR package tests; no private CorelDRAW files are stored in the repository.

use proptest::prelude::*;
use serde_json::json;

use super::*;

fn package(pages: usize) -> Vec<u8> {
    let mut files = vec![
        ("mimetype".to_string(), b"application/x-vnd.corel.zcf.draw.document+zip".to_vec()),
        ("content/root.dat".to_string(), b"RIFF\x20\0\0\0CDRU".to_vec()),
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
fn malformed_or_non_cdr_zip_is_rejected_without_panicking() {
    assert!(load("bad.cdr", b"PK\x03\x04garbage").is_err());
    assert!(!cdr::is_cdr(b"PK\x03\x04garbage"));
    let mut bytes = package(1);
    bytes[0] ^= 1;
    assert!(load("bad.cdr", &bytes).is_err());
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
