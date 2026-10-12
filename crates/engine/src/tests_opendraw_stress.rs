//! OpenDraw AI synthetic reliability cases.
//!
//! Real CorelDRAW customer projects and fonts must never be checked into public
//! CI. These deterministic documents exercise data loss and recovery paths.

use std::sync::Arc;
use std::time::Instant;

use serde_json::{Value, json};

use super::*;
use crate::cmd::recovery::MemoryStore;

fn synthetic_document(objects: usize) -> Session {
    let mut session = Session::new();
    session.execute("file.new", &json!({"width": 1000, "height": 1000, "artboards": 4})).unwrap();
    for i in 0..objects {
        let col = i % 25;
        let row = i / 25;
        session
            .execute(
                "shape.rectangle",
                &json!({"x":col as f64 * 30.0,"y":row as f64 * 30.0,"width":20,"height":20}),
            )
            .unwrap();
    }
    session
}

#[test]
fn opendraw_stress_synthetic_save_reopen_preserves_editable_objects() {
    let mut source = synthetic_document(250);
    let expected = source.doc().unwrap().doc.clone();
    let encoded = source.execute("document.serialize", &json!({"format":"vectorcraft"})).unwrap();
    let b64 = encoded["dataBase64"].as_str().unwrap().to_owned();
    assert!(!b64.is_empty());

    let mut reopened = Session::new();
    reopened.execute("document.open", &json!({"name":"opendraw-synthetic.vectorcraft","dataBase64":b64})).unwrap();
    let actual = &reopened.doc().unwrap().doc;
    assert_eq!(actual.artboards.len(), 4);
    assert_eq!(actual.node_count(), expected.node_count());
    assert_eq!(actual.layers, expected.layers, "the editable object tree, not a flattened preview, must survive");
}

#[test]
fn opendraw_stress_recovery_reopens_a_complex_dirty_document() {
    let store = Arc::new(MemoryStore::default());
    let mut source = synthetic_document(180);
    source.recovery.set_store(store.clone());
    let count = source.doc().unwrap().doc.node_count();
    let saved = source.execute("file.recovery.save", &json!({})).unwrap();
    assert_eq!(saved["saved"].as_array().unwrap().len(), 1, "{saved}");
    drop(source); // Simulate an app that stops without a normal document save.

    let mut next = Session::new();
    next.recovery.set_store(store);
    let restored = next.execute("file.recovery.restore", &json!({})).unwrap();
    assert_eq!(restored["restored"].as_array().unwrap().len(), 1, "{restored}");
    assert_eq!(next.doc().unwrap().doc.node_count(), count);
    assert_eq!(next.doc().unwrap().doc.artboards.len(), 4);
    assert!(next.doc().unwrap().is_dirty(), "recovered document must require an explicit save");
}

#[test]
fn opendraw_stress_batch_failure_rolls_back_every_object() {
    let mut session = synthetic_document(80);
    let before = session.doc().unwrap().doc.clone();
    let prior_undo = session.doc().unwrap().history.undo.len();

    let mut commands: Vec<Value> = (0..40)
        .map(|i| json!({"command":"shape.rectangle","params":{"x":i * 8,"y":500,"width":6,"height":6}}))
        .collect();
    commands.push(json!({"command":"invalid.command.must.fail","params":{}}));
    assert!(session.execute("command.batch", &json!({"label":"Synthetic layout","commands":commands})).is_err());
    assert_eq!(session.doc().unwrap().doc, before, "partial AI batches must never corrupt the document");
    assert_eq!(session.doc().unwrap().history.undo.len(), prior_undo);
    assert!(!session.in_interaction());
}

/// Opt-in benchmark: report timings, not an unreliable fixed CI threshold.
#[test]
#[ignore = "run explicitly with --ignored --nocapture on representative hardware"]
fn opendraw_benchmark_two_thousand_objects_and_round_trip() {
    let start = Instant::now();
    let mut session = synthetic_document(2_000);
    let build_time = start.elapsed();
    let save_start = Instant::now();
    let encoded = session.execute("document.serialize", &json!({"format":"vectorcraft"})).unwrap();
    let save_time = save_start.elapsed();
    let open_start = Instant::now();
    session
        .execute("document.open", &json!({"name":"benchmark.vectorcraft","dataBase64":encoded["dataBase64"]}))
        .unwrap();
    let open_time = open_start.elapsed();
    assert_eq!(session.doc().unwrap().doc.artboards.len(), 4);
    eprintln!("OpenDraw synthetic 2000: create={build_time:?}, save={save_time:?}, open={open_time:?}");
}
