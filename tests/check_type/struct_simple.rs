use std::vec;

use itertools::Itertools;
use native_db::db_type::Result;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemCustomPk {
    #[primary_key]
    id: Vec<u32>,
    #[secondary_key]
    sk: Vec<std::string::String>,
    #[secondary_key(unique)]
    sk_u: Vec<std::string::String>,
    #[secondary_key(optional)]
    sk_o: Option<Vec<std::string::String>>,
    #[secondary_key(unique, optional)]
    sk_u_o: Option<Vec<std::string::String>>,
}

#[test]
fn test_get_primary_key_custom_pk() {
    let item = ItemCustomPk {
        id: vec![1],
        sk: vec!["test".to_string()],
        sk_u: vec!["test".to_string()],
        sk_o: Some(vec!["test".to_string()]),
        sk_u_o: Some(vec!["test".to_string()]),
    };

    let mut models = Models::new();
    models.define::<ItemCustomPk>().unwrap();
    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    // Get primary key for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item: ItemCustomPk = r.get().primary(expected_id).unwrap().unwrap();
    assert_eq!(item, result_item);
    let non_expected_id = vec![3];
    let result_item: Result<Option<ItemCustomPk>> = r.get().primary(non_expected_id);
    assert!(matches!(
        result_item.unwrap_err(),
        db_type::Error::MissmatchedKeyType { .. }
    ));
    drop(r);

    // Get primary key for write transaction
    let rw = db.rw_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item = rw.get().primary(expected_id).unwrap().unwrap();
    assert_eq!(item, result_item);
    let non_expected_id = vec![3];
    let result_item: Result<Option<ItemCustomPk>> = rw.get().primary(non_expected_id);
    assert!(matches!(
        result_item.unwrap_err(),
        db_type::Error::MissmatchedKeyType { .. }
    ));
    drop(rw);

    // Get secondary key for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result_item = r
        .get()
        .secondary(ItemCustomPkKey::sk_u, expected_id)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
    let non_expected_id = vec![3];
    let result_item: Result<Option<ItemCustomPk>> =
        r.get().secondary(ItemCustomPkKey::sk_u, non_expected_id);
    assert!(matches!(
        result_item.unwrap_err(),
        db_type::Error::MissmatchedKeyType { .. }
    ));
    drop(r);

    // Get secondary key for write transaction
    let rw = db.rw_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result_item = rw
        .get()
        .secondary(ItemCustomPkKey::sk_u, expected_id)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
    let non_expected_id = vec![3];
    let result_item: Result<Option<ItemCustomPk>> =
        rw.get().secondary(ItemCustomPkKey::sk_u, non_expected_id);
    assert!(matches!(
        result_item.unwrap_err(),
        db_type::Error::MissmatchedKeyType { .. }
    ));
    drop(rw);

    // Scan primary key range for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item: Vec<ItemCustomPk> = r
        .scan()
        .primary()
        .unwrap()
        .range(expected_id..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = r
        .scan()
        .primary::<ItemCustomPk>()
        .unwrap()
        .range(non_expected_id..)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Scan primary key range for write transaction
    let rw = db.rw_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item: Vec<ItemCustomPk> = rw
        .scan()
        .primary()
        .unwrap()
        .range(expected_id..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = rw
        .scan()
        .primary::<ItemCustomPk>()
        .unwrap()
        .range(non_expected_id..)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(rw);

    // Scan primary key start with for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item: Vec<ItemCustomPk> = r
        .scan()
        .primary()
        .unwrap()
        .start_with(expected_id)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = r
        .scan()
        .primary::<ItemCustomPk>()
        .unwrap()
        .start_with(non_expected_id)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Scan primary key start with for write transaction
    let rw = db.rw_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result_item: Vec<ItemCustomPk> = rw
        .scan()
        .primary()
        .unwrap()
        .start_with(expected_id)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = rw
        .scan()
        .primary::<ItemCustomPk>()
        .unwrap()
        .start_with(non_expected_id)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(rw);

    // Scan secondary key range for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result_item: Vec<ItemCustomPk> = r
        .scan()
        .secondary(ItemCustomPkKey::sk_u)
        .unwrap()
        .range(expected_id..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = r
        .scan()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u)
        .unwrap()
        .range(non_expected_id..)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Scan secondary key range for write transaction
    let rw = db.rw_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result_item: Vec<ItemCustomPk> = rw
        .scan()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u)
        .unwrap()
        .range(expected_id..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = rw
        .scan()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u)
        .unwrap()
        .range(non_expected_id..)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(rw);

    // Scan secondary key start with for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result_item: Vec<ItemCustomPk> = r
        .scan()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u)
        .unwrap()
        .start_with(expected_id)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0], item);
    let non_expected_id = vec![3];
    if let Err(result_item) = r
        .scan()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u)
        .unwrap()
        .start_with(non_expected_id)
    {
        assert!(matches!(
            result_item,
            db_type::Error::MissmatchedKeyType { .. }
        ));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Watch get primary key for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result = db.watch().get().primary::<ItemCustomPk>(expected_id);
    assert!(result.is_ok());
    let non_expected_id = vec![3];
    if let Err(result) = db.watch().get().primary::<ItemCustomPk>(non_expected_id) {
        assert!(matches!(result, db_type::Error::MissmatchedKeyType { .. }));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Watch get secondary key for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result = db
        .watch()
        .get()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u, expected_id);
    assert!(result.is_ok());
    let non_expected_id = vec![3];
    if let Err(result) = db
        .watch()
        .get()
        .secondary::<ItemCustomPk>(ItemCustomPkKey::sk_u, non_expected_id)
    {
        assert!(matches!(result, db_type::Error::MissmatchedKeyType { .. }));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Watch scan primary key start with for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id: Vec<u32> = vec![1];
    let result = db
        .watch()
        .scan()
        .primary()
        .start_with::<ItemCustomPk>(expected_id);
    assert!(result.is_ok());
    let non_expected_id = vec![3];
    if let Err(result) = db
        .watch()
        .scan()
        .primary()
        .start_with::<ItemCustomPk>(non_expected_id)
    {
        assert!(matches!(result, db_type::Error::MissmatchedKeyType { .. }));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // Watch scan secondary key start with for read transaction
    let r = db.r_transaction().unwrap();
    let expected_id = vec!["test".to_string()];
    let result = db
        .watch()
        .scan()
        .secondary(ItemCustomPkKey::sk_u)
        .start_with::<ItemCustomPk>(expected_id);
    assert!(result.is_ok());
    let non_expected_id = vec![3];
    if let Err(result) = db
        .watch()
        .scan()
        .secondary(ItemCustomPkKey::sk_u)
        .start_with::<ItemCustomPk>(non_expected_id)
    {
        assert!(matches!(result, db_type::Error::MissmatchedKeyType { .. }));
    } else {
        panic!("Expected error");
    }
    drop(r);

    // TODO: watch scan primary key range, because it's not implemented
    // TODO: watch scan secondary key range, because it's not implemented
}
