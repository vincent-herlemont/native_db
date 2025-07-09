use native_db::db_type::Error;
use native_db::db_type::Result;
use native_db::*;

use itertools::Itertools;
use native_model::{native_model, Error as ModelError, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
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

use include_dir::{include_dir, Dir};
static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/data");

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[test]
#[ignore = "TODO: Update test to handle version upgrade errors. This test uses old database files (0.7.1) that now trigger upgrade errors."]
fn create_local_database_for_tests() {
    let tmp = TmpFs::new().unwrap();
    tmp.copy_assets(&PROJECT_DIR).unwrap();
    tmp.display_dir_entries();
    let database_path = tmp.path("db_0_8-pre-0").to_path_buf();

    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    let db = Builder::new().open(&models, &database_path).unwrap();
    let r = db.r_transaction().unwrap();
    let result: Result<Vec<Item1>> = r.scan().primary().unwrap().all().unwrap().try_collect();
    assert!(matches!(
        result,
        Err(Error::ModelError(boxed_error))
            if matches!(*boxed_error, ModelError::DecodeBodyError(_))
    ));
}
