use major_upgrade::main_old::main_old;

#[test]
fn test_main_old_creates_v08x_database() {
    // Define the database path
    let db_path = std::env::temp_dir().join("native_db_v08x_example.db");

    // Run the main_old function with the path
    let result = main_old(&db_path);

    // Verify it completed successfully
    assert!(result.is_ok(), "main_old should complete without errors");

    // Verify the database was created
    assert!(db_path.exists(), "Database file should exist");

    // Clean up
    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_v08x_database_operations() {
    use major_upgrade::models::v08x::V08xModel;
    use native_db_v0_8_x::{Builder, Models};

    // Create a test-specific database file path
    let db_path = std::env::temp_dir().join("native_db_v08x_test.db");

    // Clean up if exists
    if db_path.exists() {
        std::fs::remove_file(&db_path).unwrap();
    }

    // Define the model
    let mut models = Models::new();
    models.define::<V08xModel>().unwrap();

    // Create database - path is a file path
    let db = Builder::new().create(&models, &db_path).unwrap();

    // Test insert
    let rw = db.rw_transaction().unwrap();
    rw.insert(V08xModel {
        id: 100,
        name: "Test Model".to_string(),
    })
    .unwrap();
    rw.commit().unwrap();

    // Test read
    let r = db.r_transaction().unwrap();
    let item: V08xModel = r.get().primary(100u32).unwrap().unwrap();
    assert_eq!(item.id, 100);
    assert_eq!(item.name, "Test Model");

    // Test update
    let rw = db.rw_transaction().unwrap();
    let mut updated_item = item.clone();
    updated_item.name = "Updated Model".to_string();
    rw.update(item, updated_item).unwrap();
    rw.commit().unwrap();

    // Verify update
    let r = db.r_transaction().unwrap();
    let item: V08xModel = r.get().primary(100u32).unwrap().unwrap();
    assert_eq!(item.name, "Updated Model");

    // Clean up
    drop(db);
    std::fs::remove_file(&db_path).unwrap();
}
