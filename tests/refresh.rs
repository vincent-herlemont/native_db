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
fn test_refresh() {
    use std::path::PathBuf;
    println!("test_refresh");
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
    // TODO: during open, the database must be upgraded to the latest version.

    tmp.display_dir_entries();
    let stats = db.redb_stats().unwrap();
    dbg!(&stats);
    assert_eq!(stats.primary_tables.len(), 2);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.primary_tables[1].name, "2_1_id");
    assert_eq!(stats.primary_tables[1].n_entries, Some(1000));
    assert_eq!(stats.secondary_tables.len(), 3);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables[1].name, "2_1_id2");
    assert_eq!(stats.secondary_tables[1].n_entries, Some(500));
    assert_eq!(stats.secondary_tables[2].name, "2_1_name");
    assert_eq!(stats.secondary_tables[2].n_entries, Some(1000));

    // Is not usefull but it's generaly used with the method upgrading_from_version.
    if db.upgrading_from_version("<0.8.0").unwrap() {
        // Refresh the database
        let rw = db.rw_transaction().unwrap();
        rw.refresh::<Item1>().unwrap();
        rw.refresh::<Item2>().unwrap();
        rw.commit().unwrap();
    } else {
        assert!(false);
    }

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 2);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.primary_tables[1].name, "2_1_id");
    assert_eq!(stats.primary_tables[1].n_entries, Some(1000));
    assert_eq!(stats.secondary_tables.len(), 3);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables[1].name, "2_1_id2");
    assert_eq!(stats.secondary_tables[1].n_entries, Some(500));
    assert_eq!(stats.secondary_tables[2].name, "2_1_name");
    assert_eq!(stats.secondary_tables[2].n_entries, Some(1000));
}
