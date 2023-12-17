use native_db::*;
use shortcut_assert_fs::TmpFs;

#[test]
fn test_builder() {
    let tf = TmpFs::new().unwrap();
    // Create without error
    let mut _db = DatabaseBuilder::new().create(&tf.path("test")).unwrap();
}

#[test]
fn test_builder_with_set_cache_size() {
    let tf = TmpFs::new().unwrap();
    // Create without error
    let mut builder = DatabaseBuilder::new();
    let _db = builder
        .set_cache_size(100)
        .create(&tf.path("test"))
        .unwrap();
}

#[test]
fn test_open_unexisting_database() {
    let tf = TmpFs::new().unwrap();
    // Open an unexisting database
    assert!(DatabaseBuilder::new().open(&tf.path("test")).is_err());
}

#[test]
fn test_open_existing_database() {
    let tf = TmpFs::new().unwrap();

    // Create a database
    let builder = DatabaseBuilder::new();
    let db = builder.create(&tf.path("test")).unwrap();
    drop(db);

    // Open an existing database
    let _db = DatabaseBuilder::new().open(&tf.path("test")).unwrap();
}
