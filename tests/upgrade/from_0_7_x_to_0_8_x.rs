use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const OLD_TABLE: redb::TableDefinition<Key, Key> = redb::TableDefinition::new("2_1_name");
const NEW_TABLE: redb::MultimapTableDefinition<Key, Key> =
    redb::MultimapTableDefinition::new("2_1_name");

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
fn upgrade_from_0_7_x_to_0_8_x() {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let database_path = { dinghy_test::test_project_path().join("tests/data/db_0_7_1") };

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let database_path = {
        let root_project_path = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!("{}/tests/data/db_0_7_1", root_project_path))
    };

    use redb::ReadableMultimapTable;
    use shortcut_assert_fs::TmpFs;
    let tmp = TmpFs::new().unwrap();

    // Copy the legacy database to the temporary directory.
    let tmp_database_path = tmp.path("db_0_7_1");
    std::fs::copy(&database_path, &tmp_database_path).unwrap();

    // Check before refresh the number of bytes of the secondary table.
    // We check that the delimiter is not included in the secondary table.
    {
        let db = redb::Database::open(&tmp_database_path).unwrap();
        let rx = db.begin_read().unwrap();
        let table = rx.open_table(OLD_TABLE).unwrap();
        let (key, _) = table.first().unwrap().unwrap();
        assert_eq!(
            key.value(),
            Key::new(vec![105, 116, 101, 109, 50, 95, 48, 0, 0, 0, 0])
        );
    }

    // Refresh the database
    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    models.define::<Item2>().unwrap();
    let db = Builder::new().open(&models, &tmp_database_path).unwrap();
    drop(db);

    {
        // Check after refresh the number of bytes of the secondary table.
        // We check that the delimiter is not included in the secondary table.
        let db = redb::Database::open(&tmp_database_path).unwrap();
        let rx = db.begin_read().unwrap();
        let table = rx.open_multimap_table(NEW_TABLE).unwrap();
        let mut primary_key = None;
        let mut secondary_key = None;
        for result in table.iter().unwrap() {
            let result = result.unwrap();
            secondary_key = Some(result.0);
            for l_primary_key in result.1 {
                let l_primary_key = l_primary_key.unwrap();
                primary_key = Some(l_primary_key);
            }
            break;
        }

        assert_eq!(
            secondary_key.unwrap().value(),
            Key::new(vec![105, 116, 101, 109, 50, 95, 48])
        );
        assert_eq!(primary_key.unwrap().value(), Key::new(vec![0, 0, 0, 0]));
        drop(rx);
        drop(db);
    }

    // More tests on database integrity
    let mut models = Models::new();
    models.define::<Item1>().unwrap();
    models.define::<Item2>().unwrap();
    let db = Builder::new().open(&models, &tmp_database_path).unwrap();

    let r = db.r_transaction().unwrap();
    let len = r.len().primary::<Item1>().unwrap();
    assert_eq!(len, 1);

    let len = r.len().primary::<Item2>().unwrap();
    assert_eq!(len, 1000);

    let item5: Item2 = r.get().primary(5_u32).unwrap().unwrap();
    assert_eq!(item5.id, 5);
    assert_eq!(item5.id2, None);
    assert_eq!(item5.name, "item2_5");

    let items: Vec<Item2> = r
        .scan()
        .secondary(Item2Key::name)
        .unwrap()
        .range("item2_5".."item2_599")
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(items.len(), 110);

    let items: Vec<Item2> = r
        .scan()
        .secondary(Item2Key::name)
        .unwrap()
        .range("item2_5"..="item2_599")
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(items.len(), 111);
}
