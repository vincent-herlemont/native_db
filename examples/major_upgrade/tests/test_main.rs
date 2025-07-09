use major_upgrade::main_old::main_old;
use major_upgrade::new_main::main;

#[test]
fn test_main_creates_new_database() {
    // Test creating a new database with current version
    let db_path = std::env::temp_dir().join("test_new_db.db");

    // Clean up if exists
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    // Create new database
    let result = main(&db_path);
    assert!(result.is_ok(), "Should create new database successfully");

    // Verify database exists
    assert!(db_path.exists(), "Database file should exist");

    // Run main again - should open existing database
    let result = main(&db_path);
    assert!(result.is_ok(), "Should open existing database successfully");

    // Clean up
    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_main_upgrades_old_database() {
    // Test upgrading from v0.8.x to current version
    let db_path = std::env::temp_dir().join("test_upgrade_db.db");

    // Clean up if exists
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    // First create an old database
    let result = main_old(&db_path);
    assert!(result.is_ok(), "Should create old database successfully");
    assert!(db_path.exists(), "Old database should exist");

    // Now run main which should trigger upgrade
    let result = main(&db_path);
    assert!(result.is_ok(), "Should upgrade database successfully");

    // Verify we can open with current version
    let result = main(&db_path);
    assert!(result.is_ok(), "Should open upgraded database successfully");

    // Clean up
    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_upgrade_preserves_data() {
    use major_upgrade::models::current_version::CurrentModel;
    use major_upgrade::models::v08x::V08xModel;

    let db_path = std::env::temp_dir().join("test_data_preservation.db");

    // Clean up if exists
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    // Create old database with specific data
    {
        let mut models = native_db_v0_8_x::Models::new();
        models.define::<V08xModel>().unwrap();

        let db = native_db_v0_8_x::Builder::new()
            .create(&models, &db_path)
            .unwrap();

        let rw = db.rw_transaction().unwrap();
        rw.insert(V08xModel {
            id: 42,
            name: "Preserved Item".to_string(),
        })
        .unwrap();
        rw.insert(V08xModel {
            id: 99,
            name: "Another Preserved".to_string(),
        })
        .unwrap();
        rw.commit().unwrap();
    }

    // Run upgrade
    let result = main(&db_path);
    assert!(result.is_ok(), "Should upgrade successfully");

    // Verify data was preserved
    {
        let mut models = native_db_current::Models::new();
        models.define::<CurrentModel>().unwrap();

        let db = native_db_current::Builder::new()
            .open(&models, &db_path)
            .unwrap();

        let r = db.r_transaction().unwrap();

        let item1: CurrentModel = r.get().primary(42u32).unwrap().unwrap();
        assert_eq!(item1.id, 42);
        assert_eq!(item1.name, "Preserved Item");

        let item2: CurrentModel = r.get().primary(99u32).unwrap().unwrap();
        assert_eq!(item2.id, 99);
        assert_eq!(item2.name, "Another Preserved");

        let count = r.len().primary::<CurrentModel>().unwrap();
        assert_eq!(count, 2, "Should have exactly 2 items after migration");
    }

    // Clean up
    std::fs::remove_file(&db_path).ok();
}
