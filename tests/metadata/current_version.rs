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
#[cfg(feature = "upgrade_0_7_x")]
fn test_current_version() {
    use std::path::PathBuf;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let database_path = { dinghy_test::test_project_path().join("tests/data/db_0_7_1") };

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let database_path = {
        let root_project_path = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!("{}/tests/data/db_0_7_1", root_project_path))
    };

    use shortcut_assert_fs::TmpFs;
    let tmp = TmpFs::new().unwrap();

    // Copy the legacy database to the temporary directory.
    let tmp_database_path = tmp.path("db_0_7_1");
    std::fs::copy(&database_path, &tmp_database_path).unwrap();

    // Open the legacy database with the upgrade feature. This must succeed.
    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    models.define::<Item2>().unwrap();
    let db = Builder::new().open(&models, &tmp_database_path).unwrap();

    let metadata = db.metadata();
    assert_eq!(metadata.current_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(metadata.current_native_model_version(), "0.4.19");

    // During open, the database add the metadata table
    assert_eq!(metadata.previous_version(), None);
    assert_eq!(metadata.previous_native_model_version(), None);

    assert!(db.upgrading_from_version("<0.8.0").unwrap());

    drop(db);

    let db = Builder::new().open(&models, &tmp_database_path).unwrap();

    let metadata = db.metadata();
    assert_eq!(metadata.current_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(metadata.current_native_model_version(), "0.4.19");

    // During open, the database add the metadata table
    assert_eq!(metadata.previous_version(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(metadata.previous_native_model_version(), Some("0.4.19"));

    assert!(!db.upgrading_from_version("<0.8.0").unwrap());
}

// TODO: add test for version <=0.8.0
