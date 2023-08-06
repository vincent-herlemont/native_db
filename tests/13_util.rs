mod tests;

use struct_db::*;

#[test]
fn test_builder() {
    let tf = tests::init();
    // Create without error
    let mut _db = Builder::new().create(&tf.path("test")).unwrap();
}

#[test]
fn test_builder_with_set_cache_size() {
    let tf = tests::init();
    // Create without error
    let mut _db = Builder::new().set_cache_size(100).create(&tf.path("test")).unwrap();
}


#[test]
fn test_open_unexisting_database() {
    let tf = tests::init();
    // Open an unexisting database
    assert!(Builder::new().open(&tf.path("test")).is_err());
}

#[test]
fn test_open_existing_database() {
    let tf = tests::init();

    // Create a database
    let  db = Builder::new().create(&tf.path("test")).unwrap();
    drop(db);

    // Open an existing database
    let _db = Builder::new().open(&tf.path("test")).unwrap();
}
