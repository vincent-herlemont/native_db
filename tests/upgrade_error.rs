use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct TestModel {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_version_mismatch_error() {
    // Create a temporary directory for the test database
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test.db");

    // First, create a database with current versions
    {
        let mut models = Models::new();
        models.define::<TestModel>().unwrap();
        let db = Builder::new().create(&models, &db_path).unwrap();
        drop(db);
    }

    // Now manually modify the metadata to simulate an old version
    {
        let builder = redb::Builder::new();
        let db = builder.open(&db_path).unwrap();
        let write_tx = db.begin_write().unwrap();
        {
            let mut table = write_tx
                .open_table(redb::TableDefinition::<&str, &str>::new("metadata"))
                .unwrap();
            table.insert("version_native_db", "0.7.0").unwrap();
            table.insert("version_native_model", "0.4.18").unwrap();
        }
        write_tx.commit().unwrap();
        drop(db);
    }

    // Try to open the database again and expect an upgrade error
    let mut models = Models::new();
    models.define::<TestModel>().unwrap();

    match Builder::new().open(&models, &db_path) {
        Err(db_type::Error::UpgradeRequired(upgrade_err)) => {
            let error_string = upgrade_err.to_string();
            assert!(error_string.contains("Database upgrade required:"));
            assert!(error_string.contains("Native DB: 0.7.0 → 0.8.1"));
            assert!(error_string.contains("Native Model: 0.4.18 → 0.4.19"));

            // Check the detailed error fields
            assert_eq!(
                upgrade_err.native_db_version,
                Some(("0.7.0".to_string(), "0.8.1".to_string()))
            );
            assert_eq!(
                upgrade_err.native_model_version,
                Some(("0.4.18".to_string(), "0.4.19".to_string()))
            );
            assert_eq!(upgrade_err.redb_version, None);
        }
        Ok(_) => panic!("Expected upgrade error but database opened successfully"),
        Err(e) => panic!("Expected UpgradeRequired error but got: {:?}", e),
    }
}

#[test]
fn test_no_upgrade_needed() {
    // Create a temporary directory for the test database
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test.db");

    // Create and open a database with current versions
    let mut models = Models::new();
    models.define::<TestModel>().unwrap();

    // Create the database
    {
        let _db = Builder::new().create(&models, &db_path).unwrap();
    }

    // Open the database again - should succeed without upgrade error
    let _db = Builder::new().open(&models, &db_path).unwrap();
}

// TODO: Add test for redb upgrade error scenario
// Testing redb upgrade error would require creating a database with
// redb format version 1, which is not easily doable in a unit test.
// This would typically be tested with integration tests using actual old database files.
