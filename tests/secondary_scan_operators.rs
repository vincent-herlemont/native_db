use itertools::Itertools;
use native_db::native_model::{native_model, Model};
use native_db::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Data {
    #[primary_key]
    id: u32,

    #[secondary_key]
    a: i32,
    #[secondary_key]
    b: i32,
}

/// Creates a temporary database for testing purposes.
macro_rules! seed_db {
    ( $( $val:expr ),+ $(,)? ) => {{
        let models = Box::leak(Box::new(Models::new()));
        models.define::<Data>()?;
        let db = Builder::new().create_in_memory(models)?;

        let rw = db.rw_transaction()?;
        for item in vec![$($val),+] {
            rw.insert(item)?;
        }

        rw.commit()?;
        db
    }};
}

/// Test basic `and` behavior.
mod core_and {
    use super::*;

    #[test]
    fn and_single_common_key() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 0 },
            Data { id: 2, a: 1, b: 1 },
            Data { id: 3, a: 2, b: 2 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .and(r.scan().secondary(DataKey::b)?.range(2..5)?)
            .try_collect()?;

        assert_eq!(values, vec![Data { id: 3, a: 2, b: 2 }],);
        Ok(())
    }

    #[test]
    fn and_no_common_key_returns_empty() -> Result<(), db_type::Error> {
        let db = seed_db!(Data { id: 1, a: 0, b: 0 }, Data { id: 2, a: 1, b: 1 });
        let r = db.r_transaction()?;

        assert_eq!(
            r.scan().secondary::<Data>(DataKey::a)?.range(0..1)?.count(),
            1
        );
        assert_eq!(
            r.scan().secondary::<Data>(DataKey::b)?.range(1..2)?.count(),
            1
        );

        assert_eq!(
            r.scan()
                .secondary::<Data>(DataKey::a)?
                .range(0..1)?
                .and(r.scan().secondary(DataKey::b)?.range(1..2)?)
                .count(),
            0
        );

        Ok(())
    }

    #[test]
    fn and_full_overlap() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 0 },
            Data { id: 2, a: 1, b: 1 },
            Data { id: 3, a: 2, b: 2 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .and(r.scan().secondary(DataKey::b)?.range(0..3)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 0 },
                Data { id: 2, a: 1, b: 1 },
                Data { id: 3, a: 2, b: 2 }
            ],
        );
        Ok(())
    }
}

/// Test basic `or` behavior.
mod core_or {
    use super::*;

    #[test]
    fn or_union_disjoint_sets() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 4 },
            Data { id: 3, a: 2, b: 3 },
            Data { id: 4, a: 3, b: 2 },
            Data { id: 5, a: 4, b: 1 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..4)?
            .or(r.scan().secondary(DataKey::b)?.range(1..3)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 5 },
                Data { id: 2, a: 1, b: 4 },
                Data { id: 3, a: 2, b: 3 },
                Data { id: 4, a: 3, b: 2 },
                Data { id: 5, a: 4, b: 1 },
            ]
        );
        Ok(())
    }

    #[test]
    fn or_union_with_overlap_deduplicates() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 4 },
            Data { id: 3, a: 2, b: 3 },
            Data { id: 4, a: 3, b: 2 },
            Data { id: 5, a: 4, b: 1 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .or(r.scan().secondary(DataKey::b)?.range(1..4)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 5 },
                Data { id: 2, a: 1, b: 4 },
                Data { id: 3, a: 2, b: 3 },
                Data { id: 5, a: 4, b: 1 },
                Data { id: 4, a: 3, b: 2 },
            ]
        );
        Ok(())
    }

    #[test]
    fn or_preserves_first_iterator_order() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 4 },
            Data { id: 3, a: 2, b: 3 },
            Data { id: 4, a: 3, b: 2 },
            Data { id: 5, a: 4, b: 1 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..5)?
            .or(r.scan().secondary(DataKey::b)?.range(2..4)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 5 },
                Data { id: 2, a: 1, b: 4 },
                Data { id: 3, a: 2, b: 3 },
                Data { id: 4, a: 3, b: 2 },
                Data { id: 5, a: 4, b: 1 },
            ]
        );
        Ok(())
    }
}

/// Test `and` and `or` edge cases.
mod edge_cases {
    use super::*;

    #[test]
    fn and_with_empty_iterator() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 0 },
            Data { id: 2, a: 1, b: 1 },
            Data { id: 3, a: 2, b: 2 },
        );
        let r = db.r_transaction()?;

        assert_eq!(
            r.scan().secondary::<Data>(DataKey::a)?.range(0..3)?.count(),
            3
        );
        assert_eq!(
            r.scan().secondary::<Data>(DataKey::b)?.range(3..5)?.count(),
            0
        );
        assert_eq!(
            r.scan()
                .secondary::<Data>(DataKey::a)?
                .range(0..3)?
                .and(r.scan().secondary(DataKey::b)?.range(3..5)?)
                .count(),
            0
        );
        Ok(())
    }

    #[test]
    fn or_with_empty_iterator() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 0 },
            Data { id: 2, a: 1, b: 1 },
            Data { id: 3, a: 2, b: 2 },
            Data { id: 4, a: 3, b: 3 },
            Data { id: 5, a: 4, b: 4 },
        );
        let r = db.r_transaction()?;

        assert_eq!(
            r.scan().secondary::<Data>(DataKey::a)?.range(0..3)?.count(),
            3
        );
        assert_eq!(
            r.scan().secondary::<Data>(DataKey::b)?.range(5..7)?.count(),
            0
        );

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .or(r.scan().secondary(DataKey::b)?.range(5..7)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 0 },
                Data { id: 2, a: 1, b: 1 },
                Data { id: 3, a: 2, b: 2 },
            ]
        );
        Ok(())
    }

    // #[test]
    fn error_propagation_invalid_secondary_key() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 6 },
            Data { id: 3, a: 2, b: 7 },
            Data { id: 4, a: 3, b: 8 },
            Data { id: 5, a: 4, b: 9 },
        );
        let r = db.r_transaction()?;

        // TODO how to trigger this?

        // let result: Result<Vec<Data>, db_type::Error> = r
        //     .scan()
        //     .secondary::<Data>(DataKey::a)?
        //     .start_with("a")?
        //     .try_collect();

        // assert!(result.is_err());
        Ok(())
    }
}

/// Test mixed usages of `and` and `or`.
mod mixed {
    use super::*;

    #[test]
    fn mix_and_then_or() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 6 },
            Data { id: 3, a: 2, b: 7 },
            Data { id: 4, a: 3, b: 8 },
            Data { id: 5, a: 4, b: 9 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .and(r.scan().secondary(DataKey::b)?.range(5..7)?)
            .or(r.scan().secondary(DataKey::b)?.range(9..10)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![
                Data { id: 1, a: 0, b: 5 },
                Data { id: 2, a: 1, b: 6 },
                Data { id: 5, a: 4, b: 9 },
            ]
        );
        Ok(())
    }

    #[test]
    fn mix_or_then_and() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 6 },
            Data { id: 3, a: 2, b: 7 },
            Data { id: 4, a: 3, b: 8 },
            Data { id: 5, a: 4, b: 9 },
        );
        let r = db.r_transaction()?;

        let values: Vec<Data> = r
            .scan()
            .secondary(DataKey::a)?
            .range(0..3)?
            .or(r.scan().secondary(DataKey::b)?.range(9..10)?)
            .and(r.scan().secondary(DataKey::b)?.range(5..7)?)
            .try_collect()?;

        assert_eq!(
            values,
            vec![Data { id: 1, a: 0, b: 5 }, Data { id: 2, a: 1, b: 6 },]
        );
        Ok(())
    }

    #[test]
    fn iterator_reuse_after_and_or() -> Result<(), db_type::Error> {
        let db = seed_db!(
            Data { id: 1, a: 0, b: 5 },
            Data { id: 2, a: 1, b: 6 },
            Data { id: 3, a: 2, b: 7 },
            Data { id: 4, a: 3, b: 8 },
            Data { id: 5, a: 4, b: 9 },
        );
        let r = db.r_transaction()?;

        let scan_b = r.scan().secondary::<Data>(DataKey::b)?;
        let scan_a = r.scan().secondary::<Data>(DataKey::a)?;
        let mut iterator = scan_a.range(0..3)?.and(scan_b.range(5..7)?);

        assert_eq!(
            iterator.next().unwrap().unwrap(),
            Data { id: 1, a: 0, b: 5 },
        );
        assert_eq!(
            iterator.next().unwrap().unwrap(),
            Data { id: 2, a: 1, b: 6 },
        );
        assert!(iterator.next().is_none());

        let scan_b = r.scan().secondary::<Data>(DataKey::b)?;
        let mut iterator = iterator.or(scan_b.range(9..10)?);
        assert_eq!(
            iterator.next().unwrap().unwrap(),
            Data { id: 5, a: 4, b: 9 },
        );

        let scan_b = r.scan().secondary::<Data>(DataKey::b)?;
        let mut iterator = iterator.and(scan_b.range(9..10)?);
        assert!(iterator.next().is_none());

        Ok(())
    }
}
