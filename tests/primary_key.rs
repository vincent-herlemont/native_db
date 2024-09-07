use native_db::db_type::Result;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

macro_rules! create_test {
    ($struct_name:ident, $id_type:ty, $id_value:expr, $expected_id_type:ty, $expected_id_value:expr, $non_expected_id_type:ty, $non_expected_id_value:expr) => {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        #[allow(non_camel_case_types)]
        struct $struct_name {
            #[primary_key]
            id: $id_type,
            name: String,
        }

        #[test]
        fn $struct_name() {
            let item = $struct_name {
                id: $id_value,
                name: "test".to_string(),
            };

            let mut models = Models::new();
            models.define::<$struct_name>().unwrap();
            let db = Builder::new().create_in_memory(&models).unwrap();

            let rw = db.rw_transaction().unwrap();
            rw.insert(item.clone()).unwrap();
            rw.commit().unwrap();

            let r = db.r_transaction().unwrap();
            let id: $expected_id_type = $expected_id_value;
            let result_item = r.get().primary(id).unwrap().unwrap();
            assert_eq!(item, result_item);

            let id: $non_expected_id_type = $non_expected_id_value;
            let result_item: Result<Option<$struct_name>> = r.get().primary(id);
            assert!(result_item.is_err());
            assert!(matches!(
                result_item.unwrap_err(),
                db_type::Error::MissmatchedKeyType { .. }
            ));
        }
    };
}

create_test!(
    test_array_u32_u32,
    Vec::<u32>,
    vec![1, 2, 3],
    &'static [u32],
    &[1, 2, 3],
    &'static [u64],
    &[1, 2, 3]
);

create_test!(test_u32_u32, u32, 1u32, u32, 1u32, u128, 1u128);

create_test!(
    test_string_string,
    String,
    "1".to_string(),
    String,
    "1".to_string(),
    u128,
    1u128
);

create_test!(
    test_string_str,
    String,
    "1".to_string(),
    &str,
    "1",
    u128,
    1u128
);

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

create_test!(
    test_vecu8_vecu8,
    Vec<u8>,
    vec![1, 2, 3],
    Vec<u8>,
    vec![1, 2, 3],
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
