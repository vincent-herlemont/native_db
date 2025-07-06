use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct TestI8 {
    #[primary_key]
    id: u32,
    #[secondary_key]
    value: i8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct TestI16 {
    #[primary_key]
    id: u32,
    #[secondary_key]
    value: i16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
struct TestI32 {
    #[primary_key]
    id: u32,
    #[secondary_key]
    value: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
struct TestI64 {
    #[primary_key]
    id: u32,
    #[secondary_key]
    value: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[native_model(id = 5, version = 1)]
#[native_db]
struct TestI128 {
    #[primary_key]
    id: u32,
    #[secondary_key]
    value: i128,
}

#[test]
fn test_i8_ordering() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI8>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;
    rw.insert(TestI8 {
        id: 1,
        value: i8::MIN,
    })?;
    rw.insert(TestI8 { id: 2, value: -100 })?;
    rw.insert(TestI8 { id: 3, value: -1 })?;
    rw.insert(TestI8 { id: 4, value: 0 })?;
    rw.insert(TestI8 { id: 5, value: 1 })?;
    rw.insert(TestI8 { id: 6, value: 100 })?;
    rw.insert(TestI8 {
        id: 7,
        value: i8::MAX,
    })?;
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test full scan ordering
    let all: Vec<TestI8> = r
        .scan()
        .secondary(TestI8Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i8> = all.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![i8::MIN, -100, -1, 0, 1, 100, i8::MAX]);

    // Test negative range
    let range: Vec<TestI8> = r
        .scan()
        .secondary(TestI8Key::value)?
        .range(-100i8..=100i8)?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i8> = range.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![-100, -1, 0, 1, 100]);

    Ok(())
}

#[test]
fn test_i16_ordering() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI16>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;
    rw.insert(TestI16 {
        id: 1,
        value: i16::MIN,
    })?;
    rw.insert(TestI16 {
        id: 2,
        value: -1000,
    })?;
    rw.insert(TestI16 { id: 3, value: -1 })?;
    rw.insert(TestI16 { id: 4, value: 0 })?;
    rw.insert(TestI16 { id: 5, value: 1 })?;
    rw.insert(TestI16 { id: 6, value: 1000 })?;
    rw.insert(TestI16 {
        id: 7,
        value: i16::MAX,
    })?;
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test full scan ordering
    let all: Vec<TestI16> = r
        .scan()
        .secondary(TestI16Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i16> = all.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![i16::MIN, -1000, -1, 0, 1, 1000, i16::MAX]);

    // Test negative range
    let range: Vec<TestI16> = r
        .scan()
        .secondary(TestI16Key::value)?
        .range(-1000i16..=1000i16)?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i16> = range.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![-1000, -1, 0, 1, 1000]);

    Ok(())
}

#[test]
fn test_i32_ordering() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI32>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;
    rw.insert(TestI32 {
        id: 1,
        value: i32::MIN,
    })?;
    rw.insert(TestI32 {
        id: 2,
        value: -1000000,
    })?;
    rw.insert(TestI32 { id: 3, value: -1 })?;
    rw.insert(TestI32 { id: 4, value: 0 })?;
    rw.insert(TestI32 { id: 5, value: 1 })?;
    rw.insert(TestI32 {
        id: 6,
        value: 1000000,
    })?;
    rw.insert(TestI32 {
        id: 7,
        value: i32::MAX,
    })?;
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test full scan ordering
    let all: Vec<TestI32> = r
        .scan()
        .secondary(TestI32Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i32> = all.iter().map(|x| x.value).collect();
    assert_eq!(
        values,
        vec![i32::MIN, -1000000, -1, 0, 1, 1000000, i32::MAX]
    );

    // Test negative range
    let range: Vec<TestI32> = r
        .scan()
        .secondary(TestI32Key::value)?
        .range(-1000000..=1000000)?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i32> = range.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![-1000000, -1, 0, 1, 1000000]);

    Ok(())
}

#[test]
fn test_i64_ordering() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI64>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;
    rw.insert(TestI64 {
        id: 1,
        value: i64::MIN,
    })?;
    rw.insert(TestI64 {
        id: 2,
        value: -1000000000000,
    })?;
    rw.insert(TestI64 { id: 3, value: -1 })?;
    rw.insert(TestI64 { id: 4, value: 0 })?;
    rw.insert(TestI64 { id: 5, value: 1 })?;
    rw.insert(TestI64 {
        id: 6,
        value: 1000000000000,
    })?;
    rw.insert(TestI64 {
        id: 7,
        value: i64::MAX,
    })?;
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test full scan ordering
    let all: Vec<TestI64> = r
        .scan()
        .secondary(TestI64Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i64> = all.iter().map(|x| x.value).collect();
    assert_eq!(
        values,
        vec![i64::MIN, -1000000000000, -1, 0, 1, 1000000000000, i64::MAX]
    );

    // Test negative range
    let range: Vec<TestI64> = r
        .scan()
        .secondary(TestI64Key::value)?
        .range(-1000000000000i64..=1000000000000i64)?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i64> = range.iter().map(|x| x.value).collect();
    assert_eq!(values, vec![-1000000000000, -1, 0, 1, 1000000000000]);

    Ok(())
}

#[test]
fn test_i128_ordering() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI128>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;
    rw.insert(TestI128 {
        id: 1,
        value: i128::MIN,
    })?;
    rw.insert(TestI128 {
        id: 2,
        value: -1000000000000000000000000,
    })?;
    rw.insert(TestI128 { id: 3, value: -1 })?;
    rw.insert(TestI128 { id: 4, value: 0 })?;
    rw.insert(TestI128 { id: 5, value: 1 })?;
    rw.insert(TestI128 {
        id: 6,
        value: 1000000000000000000000000,
    })?;
    rw.insert(TestI128 {
        id: 7,
        value: i128::MAX,
    })?;
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test full scan ordering
    let all: Vec<TestI128> = r
        .scan()
        .secondary(TestI128Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i128> = all.iter().map(|x| x.value).collect();
    assert_eq!(
        values,
        vec![
            i128::MIN,
            -1000000000000000000000000,
            -1,
            0,
            1,
            1000000000000000000000000,
            i128::MAX
        ]
    );

    // Test negative range
    let range: Vec<TestI128> = r
        .scan()
        .secondary(TestI128Key::value)?
        .range(-1000000000000000000000000i128..=1000000000000000000000000i128)?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i128> = range.iter().map(|x| x.value).collect();
    assert_eq!(
        values,
        vec![
            -1000000000000000000000000,
            -1,
            0,
            1,
            1000000000000000000000000
        ]
    );

    Ok(())
}

#[test]
fn test_mixed_sign_comparisons() -> Result<(), db_type::Error> {
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestI32>()?;
    let db = Builder::new().create_in_memory(models)?;

    let rw = db.rw_transaction()?;

    // Insert values that would be problematic without order-preserving encoding
    rw.insert(TestI32 {
        id: 1,
        value: -2147483648,
    })?; // i32::MIN
    rw.insert(TestI32 { id: 2, value: -1 })?;
    rw.insert(TestI32 { id: 3, value: 0 })?;
    rw.insert(TestI32 { id: 4, value: 1 })?;
    rw.insert(TestI32 {
        id: 5,
        value: 2147483647,
    })?; // i32::MAX
    rw.commit()?;

    let r = db.r_transaction()?;

    // Test that negative values come before positive
    let all: Vec<TestI32> = r
        .scan()
        .secondary(TestI32Key::value)?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;
    let values: Vec<i32> = all.iter().map(|x| x.value).collect();

    // Verify correct ordering
    for i in 0..values.len() - 1 {
        assert!(
            values[i] < values[i + 1],
            "Values should be in ascending order: {} < {}",
            values[i],
            values[i + 1]
        );
    }

    Ok(())
}
