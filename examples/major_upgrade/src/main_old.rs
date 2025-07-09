use crate::models::v08x::V08xModel;
use native_db_v0_8_x::{Builder, Models};
use std::path::Path;

pub fn main_old<P: AsRef<Path>>(db_path: P) -> Result<(), Box<native_db_v0_8_x::db_type::Error>> {
    let db_path = db_path.as_ref();

    // Clean up if exists
    if db_path.exists() {
        std::fs::remove_file(db_path).ok();
    }

    // Define the model
    let mut models = Models::new();
    models.define::<V08xModel>().map_err(Box::new)?;

    // Create database with v0.8.x - path is a file path
    let db = Builder::new().create(&models, db_path).map_err(Box::new)?;

    // Insert some test data
    let rw = db.rw_transaction().map_err(Box::new)?;
    rw.insert(V08xModel {
        id: 1,
        name: "Test Item v0.8.x".to_string(),
    })
    .map_err(Box::new)?;
    rw.insert(V08xModel {
        id: 2,
        name: "Another Item v0.8.x".to_string(),
    })
    .map_err(Box::new)?;
    rw.commit().map_err(Box::new)?;

    // Verify data
    let r = db.r_transaction().map_err(Box::new)?;
    let item: V08xModel = r.get().primary(1u32).map_err(Box::new)?.unwrap();
    assert_eq!(item.id, 1);
    assert_eq!(item.name, "Test Item v0.8.x");

    println!("Successfully created v0.8.x database at: {db_path:?}");

    Ok(())
}
