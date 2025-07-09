use crate::models::current_version::CurrentModel;
use crate::models::v08x::V08xModel;
use native_db_current::{
    upgrade::UpgradeResultExt, Builder as CurrentBuilder, Models as CurrentModels,
};
use native_db_v0_8_x::{Builder as V08xBuilder, Models as V08xModels};
use std::path::Path;

pub fn main<P: AsRef<Path>>(db_path: P) -> Result<(), native_db_current::db_type::Error> {
    let db_path = db_path.as_ref();

    // Define the models for current version
    let mut current_models = CurrentModels::new();
    current_models.define::<CurrentModel>()?;

    // Try to open the database with current version
    match CurrentBuilder::new().open(&current_models, db_path) {
        Ok(db) => {
            println!("Successfully opened database with current version");

            // Verify we can read data
            let r = db.r_transaction()?;
            let count = r.len().primary::<CurrentModel>()?;
            println!("Database contains {count} items");

            Ok(())
        }
        Err(native_db_current::db_type::Error::UpgradeRequired(_)) => {
            println!("Database requires upgrade from v0.8.x to current version");

            // Use the database upgrade method
            let upgraded_db =
                CurrentBuilder::new().upgrade(&current_models, db_path, |new_txn| {
                    // Open the old database inside the closure
                    let mut old_models = V08xModels::new();
                    old_models
                        .define::<V08xModel>()
                        .upgrade_context("defining old model")?;

                    let old_db = V08xBuilder::new()
                        .open(&old_models, db_path)
                        .upgrade_context("opening old database")?;

                    // Read all data from old database
                    let old_txn = old_db
                        .r_transaction()
                        .upgrade_context("creating read transaction")?;
                    let mut count = 0;

                    // Use scan to iterate through all items
                    let primary_scan = old_txn
                        .scan()
                        .primary()
                        .upgrade_context("creating primary scan")?;

                    let scan_iter = primary_scan
                        .all()
                        .upgrade_context("creating scan iterator")?;

                    for item_result in scan_iter {
                        let old_item: V08xModel =
                            item_result.upgrade_context("reading item from old database")?;

                        // Convert from old model to new model
                        let new_item: CurrentModel = old_item.into();

                        // Insert into new database
                        new_txn.insert(new_item)?;
                        count += 1;
                    }

                    println!("Migrated {count} items");

                    // Old database automatically closes when it goes out of scope
                    Ok(())
                })?;

            println!("Successfully upgraded database to current version");

            // Verify data was migrated
            let r = upgraded_db.r_transaction()?;
            let count = r.len().primary::<CurrentModel>()?;
            println!("Database contains {count} items after migration");

            Ok(())
        }
        Err(e) => {
            // If database doesn't exist or other error, create new
            if !db_path.exists() {
                println!("Creating new database with current version");
                let db = CurrentBuilder::new().create(&current_models, db_path)?;

                // Insert some test data
                let rw = db.rw_transaction()?;
                rw.insert(CurrentModel {
                    id: 1,
                    name: "Test Item Current".to_string(),
                })?;
                rw.insert(CurrentModel {
                    id: 2,
                    name: "Another Item Current".to_string(),
                })?;
                rw.commit()?;

                println!("Created new database with test data");
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}
