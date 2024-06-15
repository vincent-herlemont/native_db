use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_snapshot() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();

    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item {
        id: 1,
        name: "test".to_string(),
    })
    .unwrap();
    rw.commit().unwrap();

    let db_snapshot = db
        .snapshot(&models, tf.path("snapshot.db").as_std_path())
        .unwrap();

    let r = db_snapshot.r_transaction().unwrap();
    let result_item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(
        Item {
            id: 1,
            name: "test".to_string()
        },
        result_item
    );

    tf.display_dir_entries();
}
