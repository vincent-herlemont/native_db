use std::path::PathBuf;

// Import both versions with different aliases
use native_db_current::{Builder as CurrentBuilder, Models as CurrentModels, ToKey};
use native_db_v0_8_1::{Builder as V081Builder, Models as V081Models};

use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Model for current version
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db_current::native_db]
struct CurrentModel {
    #[primary_key]
    id: u32,
    name: String,
}

// Model for v0.8.1
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db_v0_8_1::native_db]
struct V081Model {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_current_version_operations() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("test_current_multi.db");
    let _ = std::fs::remove_file(&db_path); // Cleanup any previous test db

    // Initialize database with current version
    let mut models = CurrentModels::new();
    models.define::<CurrentModel>()?;
    let db = CurrentBuilder::new().create(&models, &db_path)?;

    // Basic operations to ensure compilation and functionality
    let tx = db.rw_transaction()?;
    tx.insert(CurrentModel {
        id: 1,
        name: "Current Version Test".to_string(),
    })?;
    tx.commit()?;

    // Verify data can be read back
    let tx = db.r_transaction()?;
    let retrieved: Option<CurrentModel> = tx.get().primary(1u32)?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Current Version Test");

    // Cleanup
    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[test]
fn test_v081_operations() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("test_v081_multi.db");
    let _ = std::fs::remove_file(&db_path); // Cleanup any previous test db

    // Initialize database with v0.8.1
    let mut models = V081Models::new();
    models.define::<V081Model>()?;
    let db = V081Builder::new().create(&models, &db_path)?;

    // Basic operations to ensure compilation and functionality
    let tx = db.rw_transaction()?;
    tx.insert(V081Model {
        id: 1,
        name: "V0.8.1 Test".to_string(),
    })?;
    tx.commit()?;

    // Verify data can be read back
    let tx = db.r_transaction()?;
    let retrieved: Option<V081Model> = tx.get().primary(1u32)?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "V0.8.1 Test");

    // Cleanup
    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[test]
fn test_version_isolation() -> Result<(), Box<dyn std::error::Error>> {
    // Test that both versions can coexist and operate independently
    let current_db_path = PathBuf::from("test_current_isolation.db");
    let v081_db_path = PathBuf::from("test_v081_isolation.db");

    // Cleanup any previous test dbs
    let _ = std::fs::remove_file(&current_db_path);
    let _ = std::fs::remove_file(&v081_db_path);

    // Set up current version database
    let mut current_models = CurrentModels::new();
    current_models.define::<CurrentModel>()?;
    let current_db = CurrentBuilder::new().create(&current_models, &current_db_path)?;

    // Set up v0.8.1 database
    let mut v081_models = V081Models::new();
    v081_models.define::<V081Model>()?;
    let v081_db = V081Builder::new().create(&v081_models, &v081_db_path)?;

    // Perform operations with both versions simultaneously
    let current_tx = current_db.rw_transaction()?;
    current_tx.insert(CurrentModel {
        id: 1,
        name: "Current Isolation Test".to_string(),
    })?;
    current_tx.commit()?;

    let v081_tx = v081_db.rw_transaction()?;
    v081_tx.insert(V081Model {
        id: 1,
        name: "V0.8.1 Isolation Test".to_string(),
    })?;
    v081_tx.commit()?;

    // Verify both databases are independent and functional
    let current_read_tx = current_db.r_transaction()?;
    let current_data: Option<CurrentModel> = current_read_tx.get().primary(1u32)?;
    assert!(current_data.is_some());
    assert_eq!(current_data.unwrap().name, "Current Isolation Test");

    let v081_read_tx = v081_db.r_transaction()?;
    let v081_data: Option<V081Model> = v081_read_tx.get().primary(1u32)?;
    assert!(v081_data.is_some());
    assert_eq!(v081_data.unwrap().name, "V0.8.1 Isolation Test");

    // Cleanup
    let _ = std::fs::remove_file(&current_db_path);
    let _ = std::fs::remove_file(&v081_db_path);

    Ok(())
}

// TODO: Add more comprehensive compatibility tests
// - Database migration scenarios
// - Schema evolution tests
// - Data format compatibility
// - Performance comparisons between versions
