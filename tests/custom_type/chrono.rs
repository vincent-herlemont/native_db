use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct DateTimeItem {
    #[primary_key]
    id: u64,
    #[secondary_key(unique)]
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct NaiveDateTimeItem {
    #[primary_key]
    id: u64,
    #[secondary_key(unique)]
    timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 3, version = 1)]
#[native_db]
struct NaiveTimeItem {
    #[primary_key]
    id: u64,
    #[secondary_key(unique)]
    time: NaiveTime,
}

#[test]
fn insert_get_datetime_utc() {
    let item = DateTimeItem {
        id: 1,
        timestamp: Utc::now(),
    };
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<DateTimeItem>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test_datetime").as_std_path())
        .unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();
    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(DateTimeItemKey::timestamp, &item.timestamp)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}

#[test]
fn insert_get_naive_datetime() {
    let item = NaiveDateTimeItem {
        id: 1,
        timestamp: Utc
            .with_ymd_and_hms(1970, 1, 1, 0, 1, 1)
            .unwrap()
            .naive_utc(),
    };
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<NaiveDateTimeItem>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test_naive_datetime").as_std_path())
        .unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();
    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(NaiveDateTimeItemKey::timestamp, &item.timestamp)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}

#[test]
fn insert_get_naive_time() {
    let item = NaiveTimeItem {
        id: 1,
        time: NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
    };
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<NaiveTimeItem>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test_naive_time").as_std_path())
        .unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();
    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(NaiveTimeItemKey::time, &item.time)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}
