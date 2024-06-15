use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item1 {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct Item2 {
    #[primary_key]
    id: u32,
    #[secondary_key(optional)]
    id2: Option<u32>,
    #[secondary_key]
    name: String,
}

#[test]
#[cfg(not(feature = "upgrade_0_5_x"))]
fn try_to_open_legacy_database_without_upgrade_feature() {
    let root_project_path = env!("CARGO_MANIFEST_DIR");
    let database_path = format!("{}/tests/data/db_0_5_x", root_project_path);

    // Try to open the legacy database. This must fail with an UpgradeRequired error.
    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    let db_error: Result<Database<'_>, db_type::Error> = Builder::new().open(&models,&database_path);
    assert!(db_error.is_err());
    assert!(matches!(
        db_error,
        Result::Err(db_type::Error::RedbDatabaseError(
            redb::DatabaseError::UpgradeRequired(1)
        ))
    ));
}

#[test]
#[cfg(feature = "upgrade_0_5_x")]
fn try_to_open_legacy_database_with_upgrade_feature() {
    use std::path::PathBuf;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let database_path = { dinghy_test::test_project_path().join("tests/data/db_0_5_x") };

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let database_path = {
        let root_project_path = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!("{}/tests/data/db_0_5_x", root_project_path))
    };

    use shortcut_assert_fs::TmpFs;
    let tmp = TmpFs::new().unwrap();

    // Copy the legacy database to the temporary directory.
    let tmp_database_path = tmp.path("db_0_5_x");
    std::fs::copy(&database_path, &tmp_database_path).unwrap();

    // Open the legacy database with the upgrade feature. This must succeed.
    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    models.define::<Item2>().unwrap();
    let db = Builder::new().open(&models, &tmp_database_path).unwrap();
    // TODO: during open, the database must be upgraded to the latest version.

    tmp.display_dir_entries();

    // Check the content of the database.
    let r_txn = db.r_transaction().unwrap();
    let len = r_txn.len().primary::<Item1>().unwrap();
    assert_eq!(len, 1);

    let r_txn = db.r_transaction().unwrap();
    let len = r_txn.len().primary::<Item2>().unwrap();
    assert_eq!(len, 1000);
}
