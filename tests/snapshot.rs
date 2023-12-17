use native_db::DatabaseBuilder;
use shortcut_assert_fs::TmpFs;

#[test]
fn test_snapshot() {
    let tf = TmpFs::new().unwrap();
    let builder = DatabaseBuilder::new();
    let db = builder.create_in_memory().unwrap();
    db.snapshot(&builder, tf.path("snapshot.db").as_std_path())
        .unwrap();

    // TODO: Check the snapshot is correct

    tf.display_dir_entries();
}
