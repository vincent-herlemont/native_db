use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 941, version = 1)]
#[native_db]
struct Reading {
    #[primary_key]
    id: u32,
    #[secondary_key]
    station: String,
    #[secondary_key(unique)]
    serial: String,
    #[expire_after(60)]
    recorded_at: u64,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 942, version = 1)]
#[native_db]
struct Permanent {
    #[primary_key]
    id: u32,
    #[secondary_key]
    station: String,
    recorded_at: u64,
}

fn models() -> Models {
    let mut models = Models::new();
    models.define::<Reading>().unwrap();
    models.define::<Permanent>().unwrap();
    models
}

fn seed_readings(db: &Database) {
    let rw = db.rw_transaction().unwrap();
    rw.insert(Reading {
        id: 1,
        station: "north".into(),
        serial: "s1".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.insert(Reading {
        id: 2,
        station: "north".into(),
        serial: "s2".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.insert(Reading {
        id: 3,
        station: "north".into(),
        serial: "s3".into(),
        recorded_at: 2000,
    })
    .unwrap();
    rw.commit().unwrap();
}

fn ids(mut values: Vec<Reading>) -> Vec<u32> {
    values.sort_by_key(|v| v.id);
    values.into_iter().map(|v| v.id).collect()
}

#[test]
fn expired_value_is_hidden_from_primary_get() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();
    let expired: Option<Reading> = r.get().primary(1u32).unwrap();
    let live: Option<Reading> = r.get().primary(3u32).unwrap();

    assert!(expired.is_none());
    assert_eq!(live.map(|v| v.id), Some(3));
}

#[test]
fn expired_value_is_hidden_from_secondary_get() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(Reading {
        id: 1,
        station: "north".into(),
        serial: "s1".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.insert(Reading {
        id: 3,
        station: "south".into(),
        serial: "s3".into(),
        recorded_at: 2000,
    })
    .unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction_observed_at(1070).unwrap();
    let expired: Option<Reading> = r.get().secondary(ReadingKey::serial, "s1").unwrap();
    let live: Option<Reading> = r.get().secondary(ReadingKey::serial, "s3").unwrap();

    assert!(expired.is_none());
    assert_eq!(live.map(|v| v.id), Some(3));
}

#[test]
fn primary_scan_skips_expired_values() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();
    let seen: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();

    assert_eq!(ids(seen), vec![3]);
}

#[test]
fn secondary_scan_skips_expired_values() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();
    let seen: Vec<Reading> = r
        .scan()
        .secondary(ReadingKey::station)
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();

    assert_eq!(ids(seen), vec![3]);
}

#[test]
fn count_agrees_with_what_the_scan_returns() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();
    let scanned: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    let counted = r.len().primary::<Reading>().unwrap();

    assert_eq!(counted, 1);
    assert_eq!(counted as usize, scanned.len());
}

#[test]
fn secondary_count_agrees_with_secondary_scan() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();
    let scanned: Vec<Reading> = r
        .scan()
        .secondary(ReadingKey::station)
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    let counted = r.len().secondary::<Reading>(ReadingKey::station).unwrap();

    assert_eq!(counted as usize, scanned.len());
}

#[test]
fn value_expires_exactly_when_its_lifetime_elapses() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(Reading {
        id: 1,
        station: "north".into(),
        serial: "s1".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.commit().unwrap();

    let just_before = db.r_transaction_observed_at(1059).unwrap();
    let at_boundary = db.r_transaction_observed_at(1060).unwrap();

    let before: Option<Reading> = just_before.get().primary(1u32).unwrap();
    let at: Option<Reading> = at_boundary.get().primary(1u32).unwrap();

    assert_eq!(before.map(|v| v.id), Some(1));
    assert!(at.is_none());
}

#[test]
fn value_stays_visible_for_its_whole_lifetime() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1030).unwrap();
    let seen: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    let counted = r.len().primary::<Reading>().unwrap();

    assert_eq!(ids(seen), vec![1, 2, 3]);
    assert_eq!(counted, 3);
}

#[test]
fn read_without_an_observed_moment_sees_everything() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction().unwrap();
    let seen: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    let counted = r.len().primary::<Reading>().unwrap();
    let oldest: Option<Reading> = r.get().primary(1u32).unwrap();

    assert_eq!(ids(seen), vec![1, 2, 3]);
    assert_eq!(counted, 3);
    assert!(oldest.is_some());
}

#[test]
fn type_without_a_lifetime_is_never_hidden() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(Permanent {
        id: 1,
        station: "north".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.insert(Permanent {
        id: 2,
        station: "north".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction_observed_at(999_999).unwrap();
    let seen: Vec<Permanent> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    let counted = r.len().primary::<Permanent>().unwrap();
    let first: Option<Permanent> = r.get().primary(1u32).unwrap();

    assert_eq!(seen.len(), 2);
    assert_eq!(counted, 2);
    assert!(first.is_some());
}

#[test]
fn write_transaction_honours_expiry_too() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let rw = db.rw_transaction_observed_at(1070).unwrap();
    let expired: Option<Reading> = rw.get().primary(1u32).unwrap();
    let live: Option<Reading> = rw.get().primary(3u32).unwrap();
    let counted = rw.len().primary::<Reading>().unwrap();

    assert!(expired.is_none());
    assert_eq!(live.map(|v| v.id), Some(3));
    assert_eq!(counted, 1);
}

#[test]
fn the_boundary_is_the_same_through_every_read_path() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(Reading {
        id: 1,
        station: "north".into(),
        serial: "s1".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.commit().unwrap();

    let just_before = db.r_transaction_observed_at(1059).unwrap();
    assert!(just_before
        .get()
        .primary::<Reading>(1u32)
        .unwrap()
        .is_some());
    assert!(just_before
        .get()
        .secondary::<Reading>(ReadingKey::serial, "s1")
        .unwrap()
        .is_some());
    assert_eq!(
        just_before
            .scan()
            .primary::<Reading>()
            .unwrap()
            .all()
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        just_before
            .scan()
            .secondary::<Reading>(ReadingKey::station)
            .unwrap()
            .all()
            .unwrap()
            .count(),
        1
    );
    assert_eq!(just_before.len().primary::<Reading>().unwrap(), 1);
    assert_eq!(
        just_before
            .len()
            .secondary::<Reading>(ReadingKey::station)
            .unwrap(),
        1
    );

    let at_boundary = db.r_transaction_observed_at(1060).unwrap();
    assert!(at_boundary
        .get()
        .primary::<Reading>(1u32)
        .unwrap()
        .is_none());
    assert!(at_boundary
        .get()
        .secondary::<Reading>(ReadingKey::serial, "s1")
        .unwrap()
        .is_none());
    assert_eq!(
        at_boundary
            .scan()
            .primary::<Reading>()
            .unwrap()
            .all()
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        at_boundary
            .scan()
            .secondary::<Reading>(ReadingKey::station)
            .unwrap()
            .all()
            .unwrap()
            .count(),
        0
    );
    assert_eq!(at_boundary.len().primary::<Reading>().unwrap(), 0);
    assert_eq!(
        at_boundary
            .len()
            .secondary::<Reading>(ReadingKey::station)
            .unwrap(),
        0
    );
}

#[test]
fn an_expired_value_can_still_be_removed() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(Reading {
        id: 1,
        station: "north".into(),
        serial: "s1".into(),
        recorded_at: 1000,
    })
    .unwrap();
    rw.commit().unwrap();

    let expired_view = db.r_transaction_observed_at(9000).unwrap();
    assert!(expired_view
        .get()
        .primary::<Reading>(1u32)
        .unwrap()
        .is_none());
    assert_eq!(expired_view.len().primary::<Reading>().unwrap(), 0);

    let rw = db.rw_transaction().unwrap();
    let stored: Reading = rw.get().primary(1u32).unwrap().unwrap();
    rw.remove(stored).unwrap();
    rw.commit().unwrap();

    let after = db.r_transaction().unwrap();
    assert_eq!(after.len().primary::<Reading>().unwrap(), 0);
    assert!(after.get().primary::<Reading>(1u32).unwrap().is_none());
}

#[test]
fn range_and_start_with_scans_skip_expired_values() {
    let models = models();
    let db = Builder::new().create_in_memory(&models).unwrap();
    seed_readings(&db);

    let r = db.r_transaction_observed_at(1070).unwrap();

    let ranged: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .range(1u32..=3u32)
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    assert_eq!(ids(ranged), vec![3]);

    let reversed: Vec<Reading> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .rev()
        .map(|v| v.unwrap())
        .collect();
    assert_eq!(ids(reversed), vec![3]);

    let by_prefix: Vec<Reading> = r
        .scan()
        .secondary(ReadingKey::serial)
        .unwrap()
        .start_with("s")
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    assert_eq!(ids(by_prefix), vec![3]);
}
