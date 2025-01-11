use itertools::Itertools;
use native_db::db_type::Result;
use native_db::*;
use native_model::{native_model, Model};
use paste::paste;
use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

macro_rules! create_test {
    ($struct_name:ident, $id_type:ty, $id_value:expr, $expected_id_type:ty, $expected_id_value:expr, $non_expected_id_type:ty, $non_expected_id_value:expr) => {
        #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        #[allow(non_camel_case_types)]
        struct $struct_name {
            #[primary_key]
            id: $id_type,
            #[secondary_key(unique)]
            sk: $id_type,
        }

        #[test]
        #[allow(non_camel_case_types)]
        fn $struct_name() {
            let item = $struct_name {
                id: $id_value,
                sk: $id_value,
            };

            let mut models = Models::new();
            models.define::<$struct_name>().unwrap();
            let db = Builder::new().create_in_memory(&models).unwrap();

            let rw = db.rw_transaction().unwrap();
            rw.insert(item.clone()).unwrap();
            rw.commit().unwrap();

            let r = db.r_transaction().unwrap();

            // Get primary key
            let expected_id: $expected_id_type = $expected_id_value;
            let result_item = r.get().primary(expected_id).unwrap().unwrap();
            assert_eq!(item, result_item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            let result_item: Result<Option<$struct_name>> = r.get().primary(non_expected_id);
            assert!(result_item.is_err());
            assert!(matches!(
                result_item.unwrap_err(),
                db_type::Error::MissmatchedKeyType { .. }
            ));

            // Get secondary key
            let expected_id: $expected_id_type = $expected_id_value;
            paste! {
                let result_item = r.get().secondary([<$struct_name Key>]::sk, expected_id).unwrap().unwrap();
            }
            assert_eq!(item, result_item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            paste! {
                let result_item: Result<Option<$struct_name>> = r.get().secondary([<$struct_name Key>]::sk, non_expected_id);
            }
            assert!(result_item.is_err());
            assert!(matches!(
                result_item.unwrap_err(),
                db_type::Error::MissmatchedKeyType { .. }
            ));

            // Scan primary key range
            let expected_id: $expected_id_type = $expected_id_value;
            paste! {
                let result_item: Vec<$struct_name> = r.scan().primary().unwrap().range(expected_id..).unwrap().try_collect().unwrap();
            }
            assert_eq!(result_item.len(), 1);
            assert_eq!(result_item[0], item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            paste! {
                if let Err(result_item) = r.scan().primary::<$struct_name>().unwrap().range(non_expected_id..) {
                    assert!(matches!(
                        result_item,
                        db_type::Error::MissmatchedKeyType { .. }
                    ));
                } else {
                    panic!("scan primary key range expected error");
                }
            }

            // Scan primary key start with
            let expected_id: $expected_id_type = $expected_id_value;
            paste! {
                let result_item: Vec<$struct_name> = r.scan().primary().unwrap().start_with(expected_id).unwrap().try_collect().unwrap();
            }
            assert_eq!(result_item.len(), 1);
            assert_eq!(result_item[0], item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            paste! {
                if let Err(result_item) = r.scan().primary::<$struct_name>().unwrap().start_with(non_expected_id) {
                    assert!(matches!(
                        result_item,
                        db_type::Error::MissmatchedKeyType { .. }
                    ));
                } else {
                    panic!("scan primary key start with expected error");
                }
            }

            // Scan secondary key range
            let expected_id: $expected_id_type = $expected_id_value;
            paste! {
                let result_item: Vec<$struct_name> = r.scan().secondary([<$struct_name Key>]::sk).unwrap().range(expected_id..).unwrap().try_collect().unwrap();
            }
            assert_eq!(result_item.len(), 1);
            assert_eq!(result_item[0], item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            paste! {
                if let Err(result_item) = r.scan().secondary::<$struct_name>([<$struct_name Key>]::sk).unwrap().range(non_expected_id..) {
                    assert!(matches!(
                        result_item,
                        db_type::Error::MissmatchedKeyType { .. }
                    ));
                } else {
                    panic!("scan secondary key range expected error");
                }
            }

            // Scan secondary key start with
            let expected_id: $expected_id_type = $expected_id_value;
            paste! {
                let result_item: Vec<$struct_name> = r.scan().secondary([<$struct_name Key>]::sk).unwrap().start_with(expected_id).unwrap().try_collect().unwrap();
            }
            assert_eq!(result_item.len(), 1);
            assert_eq!(result_item[0], item);
            let non_expected_id: $non_expected_id_type = $non_expected_id_value;
            paste! {
                if let Err(result_item) = r.scan().secondary::<$struct_name>([<$struct_name Key>]::sk).unwrap().start_with(non_expected_id) {
                    assert!(matches!(
                        result_item,
                        db_type::Error::MissmatchedKeyType { .. }
                    ));
                } else {
                    panic!("scan secondary key start with expected error");
                }
            }
        }
    };
}

create_test!(test_u8_u8, u8, 1u8, u8, 1u8, u16, 1u16);
create_test!(test_u16_u16, u16, 1u16, u16, 1u16, u32, 1u32);
create_test!(test_u32_u32, u32, 1u32, u32, 1u32, u128, 1u128);
create_test!(test_u64_u64, u64, 1u64, u64, 1u64, u128, 1u128);
create_test!(test_u128_u128, u128, 1u128, u128, 1u128, u64, 1u64);

create_test!(test_i8_i8, i8, 1i8, i8, 1i8, i16, 1i16);
create_test!(test_i16_i16, i16, 1i16, i16, 1i16, i32, 1i32);
create_test!(test_i32_i32, i32, 1i32, i32, 1i32, i64, 1i64);
create_test!(test_i64_i64, i64, 1i64, i64, 1i64, i128, 1i128);
create_test!(test_i128_i128, i128, 1i128, i128, 1i128, i64, 1i64);

create_test!(test_f32_f32, f32, 1.0f32, f32, 1.0f32, f64, 1.0f64);
create_test!(test_f64_f64, f64, 1.0f64, f64, 1.0f64, f32, 1.0f32);

create_test!(test_char_char, char, 'a', char, 'a', u8, 97u8);

// Tests for String Types

create_test!(
    test_string_string,
    String,
    "test".to_string(),
    String,
    "test".to_string(),
    u128,
    1u128
);

create_test!(
    test_string_str,
    String,
    "test".to_string(),
    &str,
    "test",
    u128,
    1u128
);

// Tests for Compound Types

create_test!(
    test_option_u32_option_u32,
    Option<u32>,
    Some(1u32),
    Option<u32>,
    Some(1u32),
    Option<u64>,
    Some(1u64)
);

create_test!(
    test_array_u32_u32,
    Vec<u32>,
    vec![1, 2, 3],
    &'static [u32],
    &[1, 2, 3],
    &'static [u64],
    &[1, 2, 3]
);

create_test!(
    test_vecu8_vecu8,
    Vec<u8>,
    vec![1, 2, 3],
    Vec<u8>,
    vec![1, 2, 3],
    u128,
    1u128
);

create_test!(
    test_tuple_u32_string,
    (u32, String),
    (1u32, "test".to_string()),
    (u32, String),
    (1u32, "test".to_string()),
    (u32, u32),
    (1u32, 2u32)
);

// Tests for Custom Types

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
struct MyUuid(uuid::Uuid);

impl ToKey for MyUuid {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }

    fn key_names() -> Vec<String> {
        vec!["MyUuid".to_string()]
    }
}

const UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

create_test!(
    test_myuuid_myuuid,
    MyUuid,
    MyUuid(UUID),
    MyUuid,
    MyUuid(UUID),
    u128,
    1u128
);

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
enum PkEnum {
    A,
    B,
    C,
}

impl ToKey for PkEnum {
    fn to_key(&self) -> Key {
        match self {
            PkEnum::A => Key::new(vec![0]),
            PkEnum::B => Key::new(vec![1]),
            PkEnum::C => Key::new(vec![2]),
        }
    }

    fn key_names() -> Vec<String> {
        vec!["PkEnum".to_string()]
    }
}

create_test!(
    test_pkenum_pkenum,
    PkEnum,
    PkEnum::A,
    PkEnum,
    PkEnum::A,
    u128,
    1u128
);

create_test!(test_bool_bool, bool, true, bool, true, u8, 1u8);
