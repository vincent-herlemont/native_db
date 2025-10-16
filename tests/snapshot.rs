use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::fs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_snapshot() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();

    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item {
        id: 1,
        name: "test".to_string(),
    })
    .unwrap();
    rw.commit().unwrap();

    let db_snapshot = db
        .snapshot(&models, tf.path("snapshot.db").as_std_path())
        .unwrap();

    let r = db_snapshot.r_transaction().unwrap();
    let result_item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(
        Item {
            id: 1,
            name: "test".to_string()
        },
        result_item
    );

    tf.display_dir_entries();
}

#[test]
fn test_snapshot_compact_interaction() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db_path = tf.path("db");
    let snapshot_path = tf.path("snapshot.db");

    // Create DB and insert many items
    let mut db = Builder::new().create(&models, db_path.clone()).unwrap();
    let rw = db.rw_transaction().unwrap();
    for i in 0..1000 {
        rw.insert(Item {
            id: i,
            name: format!("item_{}", i),
        })
        .unwrap();
    }
    rw.commit().unwrap();

    // File size before compact
    let orig_size = fs::metadata(&db_path).unwrap().len();
    println!("Original DB size before compact: {}", orig_size);

    // Take snapshot without compacting
    let db_snapshot = db.snapshot(&models, snapshot_path.as_std_path()).unwrap();
    let snap_size = fs::metadata(&snapshot_path).unwrap().len();
    println!("Snapshot size without compact: {}", snap_size);

    // Compact the original DB, then snapshot again
    db.compact().unwrap();
    let compacted_size = fs::metadata(&db_path).unwrap().len();
    println!("Original DB size after compact: {}", compacted_size);

    let snapshot_path2 = tf.path("snapshot2.db");
    let db_snapshot2 = db.snapshot(&models, snapshot_path2.as_std_path()).unwrap();
    let snap2_size = fs::metadata(&snapshot_path2).unwrap().len();
    println!("Snapshot size after compact: {}", snap2_size);

    // Drop DB handles before reopening files
    drop(db_snapshot);
    drop(db_snapshot2);

    // Compact the first snapshot file
    let mut db_snap = Builder::new().open(&models, &snapshot_path).unwrap();
    db_snap.compact().unwrap();
    let snap_compacted_size = fs::metadata(&snapshot_path).unwrap().len();
    println!("Snapshot file size after compact: {}", snap_compacted_size);

    // Open the compacted snapshot and compact again
    drop(db_snap);
    let mut db_snap2 = Builder::new().open(&models, &snapshot_path).unwrap();
    db_snap2.compact().unwrap();
    let snap_compacted_size2 = fs::metadata(&snapshot_path).unwrap().len();
    println!(
        "Snapshot file size after second compact: {}",
        snap_compacted_size2
    );

    // TODO: The snapshot process does not compact the database file, so the snapshot can be much larger than a compacted database. Compacting the snapshot after creation reduces its size, but not always to the minimum possible. Ideally, snapshot should optionally compact or copy only live data.
    // This test demonstrates the issue reported by the user: https://github.com/vincent-herlemont/native_db/issues/XXX
    //
    // User's report:
    // - Compacting the active database reduces its size.
    // - Snapshotting a non-compacted DB produces a large file.
    // - Compacting the snapshot helps, but not as much as compacting the original and then snapshotting.
    // - Only after replacing the active DB with the compacted snapshot and compacting again does the file reach minimum size.

    // Assertions:
    assert!(
        orig_size > compacted_size,
        "Compacting should reduce the original DB size"
    );
    assert_eq!(
        orig_size, snap_size,
        "Snapshot without compact should match original size"
    );
    assert!(
        snap2_size >= compacted_size,
        "Snapshot after compact should be at least as large as compacted DB (but may be larger)"
    );
    assert!(
        snap_compacted_size < snap_size,
        "Compacting the snapshot should reduce its size"
    );
    assert!(
        snap_compacted_size2 <= snap_compacted_size,
        "Second compaction should not increase size"
    );
    // The compacted snapshot may still be larger than the compacted original DB
    // (implementation detail, but should be close)
    // Print all sizes for manual inspection
    println!("Sizes summary:");
    println!("  Original DB before compact: {}", orig_size);
    println!("  Snapshot (no compact):      {}", snap_size);
    println!("  Original DB after compact:  {}", compacted_size);
    println!("  Snapshot after compact:     {}", snap2_size);
    println!("  Snapshot file after compact:{}", snap_compacted_size);
    println!("  Snapshot file after 2x cmp: {}", snap_compacted_size2);
    // Display for manual inspection
    tf.display_dir_entries();
}
