use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod current_version_tests {
    use super::*;

    // Import current version as native_db for macro expansion
    use native_db_current as native_db;
    use native_db_current::{Builder, Models, ToKey};

    // Import native_model macro version matched with the current version native_db for macro expansion.
    use native_model_current as native_model;
    use native_model_current::{native_model, Model};

    // Model for current version
    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    // We no need to add the `from` attribute here, we manually implement
    // conversion between the two models using `.into()` method.
    // Maybe we could  reset the version number too. And set it to 1.
    #[native_model(id = 1, version = 1)]
    #[native_db_current::native_db]
    pub struct CurrentModel {
        #[primary_key]
        pub id: u32,
        pub name: String,
    }

    // Upgrade from v0.8.1 to current version
    impl From<crate::v081_tests::V081Model> for CurrentModel {
        fn from(v081_model: crate::v081_tests::V081Model) -> Self {
            Self {
                id: v081_model.id,
                name: v081_model.name,
            }
        }
    }

    // Downgrade from current version to v0.8.1
    impl From<CurrentModel> for crate::v081_tests::V081Model {
        fn from(current_model: CurrentModel) -> Self {
            Self {
                id: current_model.id,
                name: current_model.name,
            }
        }
    }

    #[test]
    pub fn test_current_version_operations() -> Result<(), Box<dyn std::error::Error>> {
        let db_path = PathBuf::from("test_current_multi.db");
        let _ = std::fs::remove_file(&db_path); // Cleanup any previous test db

        // Initialize database with current version
        let mut models = Models::new();
        models.define::<CurrentModel>()?;
        let db = Builder::new().create(&models, &db_path)?;

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
}

mod v081_tests {
    use super::*;

    // Import v0.8.1 version as native_db for macro expansion
    use native_db_v0_8_x as native_db;
    use native_db_v0_8_x::{Builder, Models, ToKey};

    // Import native_model macro version matched with the v0.8.1 version native_db for macro expansion.
    use native_model_v0_4_x as native_model;
    use native_model_v0_4_x::{native_model, Model};

    // Model for v0.8.1
    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[native_model(id = 1, version = 1)]
    #[native_db_v0_8_x::native_db]
    pub struct V081Model {
        #[primary_key]
        pub id: u32,
        pub name: String,
    }

    #[test]
    pub fn test_v081_operations() -> Result<(), Box<dyn std::error::Error>> {
        let db_path = PathBuf::from("test_v081_multi.db");
        let _ = std::fs::remove_file(&db_path); // Cleanup any previous test db

        // Initialize database with v0.8.1
        let mut models = Models::new();
        models.define::<V081Model>()?;
        let db = Builder::new().create(&models, &db_path)?;

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
}

#[test]
fn test_migration_with_native_model_only() -> Result<(), Box<dyn std::error::Error>> {
    // Encore with old version
    let binary_old = {
        // Create old model
        use crate::v081_tests::V081Model;
        let old_model = V081Model {
            id: 1,
            name: "Old Model".to_string(),
        };

        // Encode using only native_model old version
        use native_model_v0_4_x as native_model;
        let encoded = native_model::encode(&old_model)?;
        println!("Encoded: {:?}", encoded);
        encoded
    };

    // Use the old model to decode binary_old
    let old_model = {
        use crate::v081_tests::V081Model;
        use native_model_v0_4_x as native_model;
        let old_model: (V081Model, _) = native_model::decode(binary_old)?;
        old_model.0
    };

    // Transform old model to current model
    let current_model = {
        use crate::current_version_tests::CurrentModel;
        let current_model: CurrentModel = old_model.into();
        current_model
    };

    // Verify the transformation worked
    assert_eq!(current_model.name, "Old Model");
    println!("Successfully transformed: {:?}", current_model);

    Ok(())
}

#[test]
fn test_migration_with_native_model_and_native_db() -> Result<(), Box<dyn std::error::Error>> {
    let old_db_path = PathBuf::from("test_migration_old.db");
    let new_db_path = PathBuf::from("test_migration_new.db");

    // Cleanup any previous test dbs
    let _ = std::fs::remove_file(&old_db_path);
    let _ = std::fs::remove_file(&new_db_path);

    // Step 1: Create and populate database with old version (v0.8.1)
    let old_model = {
        use crate::v081_tests::V081Model;
        use native_db_v0_8_x::{Builder, Models};

        // Initialize database with v0.8.1
        let mut models = Models::new();
        models.define::<V081Model>()?;
        let db = Builder::new().create(&models, &old_db_path)?;

        // Store old model in database
        let old_model = V081Model {
            id: 1,
            name: "Migration Test Data".to_string(),
        };

        let tx = db.rw_transaction()?;
        tx.insert(old_model.clone())?;
        tx.commit()?;

        // Read back from old database to verify storage
        let tx = db.r_transaction()?;
        let retrieved: Option<V081Model> = tx.get().primary(1u32)?;
        assert!(retrieved.is_some());

        println!("Old model from database: {:?}", retrieved);
        retrieved.unwrap()
    };

    // Step 2: Transform old model to current model
    let current_model = {
        use crate::current_version_tests::CurrentModel;
        let current_model: CurrentModel = old_model.into();
        println!("Transformed to current model: {:?}", current_model);
        current_model
    };

    // Step 3: Store transformed model in new database with current version
    {
        use crate::current_version_tests::CurrentModel;
        use native_db_current::{Builder, Models};

        // Initialize database with current version
        let mut models = Models::new();
        models.define::<CurrentModel>()?;
        let db = Builder::new().create(&models, &new_db_path)?;

        // Store current model in new database
        let tx = db.rw_transaction()?;
        tx.insert(current_model)?;
        tx.commit()?;

        // Verify the migrated data in new database
        let tx = db.r_transaction()?;
        let retrieved: Option<CurrentModel> = tx.get().primary(1u32)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.clone().unwrap().name, "Migration Test Data");

        println!("Current model from new database: {:?}", retrieved);
    }

    // Cleanup
    let _ = std::fs::remove_file(&old_db_path);
    let _ = std::fs::remove_file(&new_db_path);

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
    {
        use current_version_tests::CurrentModel;
        // use native_db_current as native_db;
        use native_db_current::{Builder, Models};

        let mut current_models = Models::new();
        current_models.define::<CurrentModel>()?;
        let current_db = Builder::new().create(&current_models, &current_db_path)?;

        // Perform operations with current version
        let current_tx = current_db.rw_transaction()?;
        current_tx.insert(CurrentModel {
            id: 1,
            name: "Current Isolation Test".to_string(),
        })?;
        current_tx.commit()?;

        // Verify current version data
        let current_read_tx = current_db.r_transaction()?;
        let current_data: Option<CurrentModel> = current_read_tx.get().primary(1u32)?;
        assert!(current_data.is_some());
        assert_eq!(current_data.unwrap().name, "Current Isolation Test");
    }

    // Set up v0.8.1 database
    {
        use native_db_v0_8_x::{Builder, Models};
        use v081_tests::V081Model;

        let mut v081_models = Models::new();
        v081_models.define::<V081Model>()?;
        let v081_db = Builder::new().create(&v081_models, &v081_db_path)?;

        // Perform operations with v0.8.1
        let v081_tx = v081_db.rw_transaction()?;
        v081_tx.insert(V081Model {
            id: 1,
            name: "V0.8.1 Isolation Test".to_string(),
        })?;
        v081_tx.commit()?;

        // Verify v0.8.1 data
        let v081_read_tx = v081_db.r_transaction()?;
        let v081_data: Option<V081Model> = v081_read_tx.get().primary(1u32)?;
        assert!(v081_data.is_some());
        assert_eq!(v081_data.unwrap().name, "V0.8.1 Isolation Test");
    }

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
