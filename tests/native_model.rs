use bincode::{config, Decode, Encode};
use native_db::*;
use native_db_macro::native_db;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

pub struct Bincode;
impl<T: bincode::Encode> native_model::Encode<T> for Bincode {
    type Error = bincode::error::EncodeError;
    fn encode(obj: &T) -> std::result::Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(obj, config::standard())
    }
}

impl<T: bincode::Decode> native_model::Decode<T> for Bincode {
    type Error = bincode::error::DecodeError;
    fn decode(data: Vec<u8>) -> std::result::Result<T, bincode::error::DecodeError> {
        bincode::decode_from_slice(&data, config::standard()).map(|(result, _)| result)
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1, with = Bincode)]
#[native_db(primary_key(compute_primary_key))]
struct ItemV1 {
    id: u32,
    name: String,
}

impl ItemV1 {
    #[allow(dead_code)]
    pub fn compute_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn test_native_encode() {
    let my_item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let my_item_packed = native_model::encode(&my_item).unwrap();
    let (my_item_unpacked, _) = native_model::decode::<ItemV1>(my_item_packed).unwrap();
    assert_eq!(my_item, my_item_unpacked);
}
