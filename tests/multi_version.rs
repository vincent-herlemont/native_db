use std::path::PathBuf;

// Current version imports
use native_db::{Builder, Models, ToKey};
use native_db_macro::native_db;
use native_db_v0_8_1::{Builder as Builder_v0_8_1, Models as Models_v0_8_1};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Example model using current version (for initial compilation)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct SimpleModel {
    #[primary_key]
    id: u32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db(native_db = v0_8_1)]
struct ModelV0_8_1 {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_current_version() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("test_current.db");
    let _ = std::fs::remove_file(&db_path); // Cleanup any previous test db

    // Initialize database with current version
    let mut models = Models::new();
    models.define::<SimpleModel>()?;
    let db = Builder::new().create(&models, &db_path).unwrap();

    // Basic operations to ensure compilation
    let tx = db.rw_transaction()?;
    tx.insert(SimpleModel {
        id: 1,
        name: "Test".to_string(),
    })?;
    tx.commit()?;

    // Cleanup
    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

// Example of future multi-version tests (commented for now)
/*
#[test]
fn test_multi_version_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Test scenarios where different versions coexist
    // - Create database with v8.0
    // - Insert data using v8.0 model
    // - Read data using v8.1 model
    // - Verify data consistency
    // NOTE: Testing this feature requires aliasing the crate in Cargo.toml,
    // which is problematic for integration tests within the same crate.
    // A dedicated test crate would be needed to test this properly.
    // Example with aliased dependencies:
    //
    // In Cargo.toml:
    // [dev-dependencies]
    // native_db_v1 = { package = "native_db", path = "." }
    //
    // In the test:
    // extern crate native_db_v1;
    // use native_db_v1::native_db_macro as native_db_macro_v1;
    //
    // #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    // #[native_model(id = 2, version = 1)]
    // #[native_db(native_db = native_db_v1, native_db_macro = native_db_v1::native_db_macro)]
    // struct ModelV1 { ... }
    Ok(())
}

#[test]
fn test_migration_path() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Test migration scenarios
    // - Create database with older version
    // - Migrate to newer version
    // - Verify data integrity
    // Test rollback scenarios
    Ok(())
}
*/
