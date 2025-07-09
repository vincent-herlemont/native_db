use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::fs;

// Version 1 of our model
#[derive(Serialize, Deserialize, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct PersonV1 {
    #[primary_key]
    id: u32,
    name: String,
}

// Version 2 of our model - adds an age field
#[derive(Serialize, Deserialize, Debug)]
#[native_model(id = 1, version = 2)]
#[native_db]
struct PersonV2 {
    #[primary_key]
    id: u32,
    name: String,
    age: u32,
}

// Implement conversion from V1 to V2
impl From<PersonV1> for PersonV2 {
    fn from(v1: PersonV1) -> Self {
        PersonV2 {
            id: v1.id,
            name: v1.name,
            age: 0, // Default age for migrated records
        }
    }
}

fn main() -> Result<(), db_type::Error> {
    let db_path = "example_upgrade.db";

    // Clean up any existing files
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(format!("{}.upgrading", db_path));
    let _ = fs::remove_file(format!("{}.old", db_path));

    // Step 1: Create a database with V1 model
    println!("Creating database with PersonV1 model...");
    {
        let mut models = Models::new();
        models.define::<PersonV1>()?;
        let db = Builder::new().create(&models, db_path)?;

        let txn = db.rw_transaction()?;
        txn.insert(PersonV1 {
            id: 1,
            name: "Alice".to_string(),
        })?;
        txn.insert(PersonV1 {
            id: 2,
            name: "Bob".to_string(),
        })?;
        txn.commit()?;

        println!("Inserted 2 PersonV1 records");
    }

    // Step 2: Upgrade the database to V2 model
    println!("\nUpgrading database to PersonV2 model...");
    let mut new_models = Models::new();
    new_models.define::<PersonV2>()?;

    let upgraded_db = Builder::new().upgrade(&new_models, db_path, |new_txn| {
        println!("Migration closure started...");

        // Open the old database
        let mut old_models = Models::new();
        old_models.define::<PersonV1>()?;
        let old_db = Builder::new().open(&old_models, db_path)?;

        // Read all V1 records
        let old_txn = old_db.r_transaction()?;
        let mut count = 0;

        for result in old_txn.scan().primary()?.all()? {
            let person_v1: PersonV1 = result?;
            println!("  Migrating: {:?}", person_v1);

            // Convert V1 to V2
            let person_v2: PersonV2 = person_v1.into();

            // Insert into new database
            new_txn.insert(person_v2)?;
            count += 1;
        }

        println!("Migration completed: {} records migrated", count);
        Ok(())
    })?;

    // Step 3: Verify the upgraded database
    println!("\nVerifying upgraded database...");
    let read_txn = upgraded_db.r_transaction()?;

    for result in read_txn.scan().primary()?.all()? {
        let person: PersonV2 = result?;
        println!(
            "  PersonV2: id={}, name={}, age={}",
            person.id, person.name, person.age
        );
    }

    // Check that backup was created
    if fs::metadata(format!("{}.old", db_path)).is_ok() {
        println!("\nBackup database created: {}.old", db_path);
    }

    // Clean up
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(format!("{}.old", db_path));

    Ok(())
}
