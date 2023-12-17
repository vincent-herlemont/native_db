use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

type Item = ItemV1;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemV0 {
    #[primary_key]
    pub id: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct ItemV1 {
    #[primary_key]
    pub id: String,
}

impl From<ItemV0> for ItemV1 {
    fn from(item: ItemV0) -> Self {
        ItemV1 {
            id: item.id.to_string(),
        }
    }
}

#[test]
fn convert_all() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemV0>().unwrap();
    builder.define::<ItemV1>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a = ItemV0 { id: 42 };

    let rw_txn = db.rw_transaction().unwrap();
    rw_txn.insert(a.clone()).unwrap();
    rw_txn.commit().unwrap();

    // Check if a is saved
    let txn = db.rw_transaction().unwrap();
    let a1 = txn.get().primary(a.id).unwrap().unwrap();
    assert_eq!(a, a1);
    txn.commit().unwrap();

    #[allow(unused_mut)]
    #[cfg(not(feature = "tokio"))]
    let (mut recv_av1, _id) = db.watch().scan().primary().all::<ItemV0>().unwrap();
    #[allow(unused_mut)]
    #[cfg(not(feature = "tokio"))]
    let (mut recv_av2, _id) = db.watch().scan().primary().all::<ItemV1>().unwrap();

    #[cfg(feature = "tokio")]
    let (mut recv_av1, _id) = db.watch().scan().primary().all::<ItemV0>().unwrap();
    #[cfg(feature = "tokio")]
    let (mut recv_av2, _id) = db.watch().scan().primary().all::<ItemV1>().unwrap();

    // Migrate
    let rw_txn = db.rw_transaction().unwrap();
    rw_txn.convert_all::<ItemV0, ItemV1>().unwrap();
    rw_txn.commit().unwrap();

    // Check is there is no event from AV1
    assert!(recv_av1.try_recv().is_err());
    // Check is there is no event from AV2
    assert!(recv_av2.try_recv().is_err());

    // Check migration
    let r_txn = db.r_transaction().unwrap();
    let len_av1 = r_txn.len().primary::<ItemV0>().unwrap();
    assert_eq!(len_av1, 0);
    let len_av2 = r_txn.len().primary::<ItemV1>().unwrap();
    assert_eq!(len_av2, 1);

    let a2: Item = r_txn.get().primary("42").unwrap().unwrap();
    assert_eq!(
        a2,
        Item {
            id: "42".to_string()
        }
    );
}
