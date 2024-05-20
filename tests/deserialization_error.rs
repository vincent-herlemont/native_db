use native_db::*;
use native_db::db_type::Result;
use native_db::db_type::Error;

use native_model::{native_model, Model, Error as ModelError};
use serde::{Deserialize, Serialize};
use itertools::Itertools;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item1 {
    // The type of the primary key must be u32, see generation of the test "create_local_database_for_tests".
    // We change the type of the primary key to String to generate a deserialization error.
    #[primary_key]
    id: String, 
    #[secondary_key(unique)]
    name: String,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[test]
fn create_local_database_for_tests() {
    let root_project_path = env!("CARGO_MANIFEST_DIR");
    let database_path: String = format!("{}/tests/data/db_0_6_0", root_project_path);

    println!("database_path: {}", database_path);
    let mut builder = DatabaseBuilder::new();
    builder.define::<Item1>().unwrap();
    let db = builder.open(&database_path).unwrap();
    let r = db.r_transaction().unwrap();
    let result: Result<Vec<Item1>> = r.scan().primary().unwrap().all().try_collect();
    assert!(matches!(result, Err(Error::ModelError(ModelError::DecodeBodyError(_)))));
}
