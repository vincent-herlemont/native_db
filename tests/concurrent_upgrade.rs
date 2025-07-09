use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemV1 {
    #[primary_key]
    id: u32,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 1, version = 2, from = ItemV1)]
#[native_db]
struct ItemV2 {
    #[primary_key]
    id: u32,
    value: String,
    new_field: i32,
}

impl From<ItemV1> for ItemV2 {
    fn from(item: ItemV1) -> Self {
        Self {
            id: item.id,
            value: item.value,
            new_field: 0,
        }
    }
}

impl From<ItemV2> for ItemV1 {
    fn from(item: ItemV2) -> Self {
        Self {
            id: item.id,
            value: item.value,
        }
    }
}

#[test]
fn test_concurrent_upgrade_protection() {
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test.db");

    // Create initial database with V1 model
    {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        let db = Builder::new().create(&models, &db_path).unwrap();

        // Insert some data
        let txn = db.rw_transaction().unwrap();
        txn.insert(ItemV1 {
            id: 1,
            value: "test".to_string(),
        })
        .unwrap();
        txn.commit().unwrap();
    }

    // Now try to upgrade from two threads concurrently
    let db_path_clone = db_path.clone();

    let handle1 = thread::spawn(move || {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        models.define::<ItemV2>().unwrap();

        Builder::new()
            .upgrade(&models, &db_path_clone, |_txn| {
                // Simulate some work during upgrade
                thread::sleep(Duration::from_millis(100));

                // The upgrade creates a new empty database, so we just
                // simulate some migration work here
                Ok(())
            })
            .map(|_| ()) // Convert to Result<(), Error>
    });

    let db_path_clone2 = db_path.clone();

    let handle2 = thread::spawn(move || {
        // Give the first thread a small head start
        thread::sleep(Duration::from_millis(10));

        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        models.define::<ItemV2>().unwrap();

        Builder::new()
            .upgrade(&models, &db_path_clone2, |_txn| {
                // This should fail with AlreadyExists error
                Ok(())
            })
            .map(|_| ()) // Convert to Result<(), Error>
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();

    // One should succeed, one should fail
    match (result1, result2) {
        (Ok(_), Err(e)) => {
            // First succeeded, second failed - expected
            match e {
                db_type::Error::Io(io_err) => {
                    assert_eq!(io_err.kind(), std::io::ErrorKind::AlreadyExists);
                    assert!(io_err.to_string().contains("Upgrade already in progress"));
                    assert!(io_err.to_string().to_lowercase().contains("lock file"));
                }
                _ => panic!("Expected IO error with AlreadyExists, got: {:?}", e),
            }
        }
        (Err(e), Ok(_)) => {
            // Second succeeded, first failed - also valid
            match e {
                db_type::Error::Io(io_err) => {
                    assert_eq!(io_err.kind(), std::io::ErrorKind::AlreadyExists);
                    assert!(io_err.to_string().contains("Upgrade already in progress"));
                    assert!(io_err.to_string().to_lowercase().contains("lock file"));
                }
                _ => panic!("Expected IO error with AlreadyExists, got: {:?}", e),
            }
        }
        (Ok(_), Ok(_)) => panic!("Both upgrades succeeded - race condition not prevented!"),
        (Err(e1), Err(e2)) => panic!("Both upgrades failed: {:?}, {:?}", e1, e2),
    }

    // Verify the upgrade succeeded by opening with V2 model
    let mut models = Models::new();
    models.define::<ItemV2>().unwrap();
    let _db = Builder::new().open(&models, &db_path).unwrap();
    // Database should open successfully with the new model
}

#[test]
fn test_lock_file_cleanup_on_success() {
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test.db");
    let lock_path = tf.path("test.db.upgrade.lock");

    // Create initial database with V1 model
    {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        let _db = Builder::new().create(&models, &db_path).unwrap();
    }

    // Perform upgrade
    {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        models.define::<ItemV2>().unwrap();

        let _db = Builder::new()
            .upgrade(&models, &db_path, |_txn| Ok(()))
            .unwrap();
    }

    // Lock file should be cleaned up
    assert!(
        !lock_path.exists(),
        "Lock file was not cleaned up after successful upgrade"
    );
}

#[test]
fn test_lock_file_cleanup_on_error() {
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test.db");
    let lock_path = tf.path("test.db.upgrade.lock");

    // Create initial database with V1 model
    {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        let _db = Builder::new().create(&models, &db_path).unwrap();
    }

    // Perform upgrade that fails
    {
        let mut models = Models::new();
        models.define::<ItemV1>().unwrap();
        models.define::<ItemV2>().unwrap();

        let result = Builder::new().upgrade(&models, &db_path, |_txn| {
            // Force an error during migration
            Err(db_type::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Test error",
            )))
        });

        assert!(result.is_err());
    }

    // Lock file should still be cleaned up
    assert!(
        !lock_path.exists(),
        "Lock file was not cleaned up after failed upgrade"
    );
}
