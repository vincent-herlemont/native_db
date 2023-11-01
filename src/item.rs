#[cfg(not(feature = "use_native_model"))]
pub trait SDBItem: Sized {
    fn struct_db_schema() -> crate::Schema;
    fn struct_db_primary_key(&self) -> Vec<u8>;

    // Return map of secondary table name and the value of the key
    fn struct_db_keys(&self) -> std::collections::HashMap<&'static str, Vec<u8>>;
    fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8>;
    fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self;

    fn to_item(&self) -> Item {
        Item {
            primary_key: self.struct_db_primary_key(),
            secondary_keys: self.struct_db_keys(),
            value: self.struct_db_bincode_encode_to_vec(),
        }
    }
}

#[cfg(feature = "use_native_model")]
pub trait SDBItem: Sized + native_model::Model {
    fn struct_db_schema() -> crate::Schema;
    fn struct_db_primary_key(&self) -> Vec<u8>;
    fn struct_db_keys(&self) -> std::collections::HashMap<&'static str, Vec<u8>>;
    fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8>;
    fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self;

    fn to_item(&self) -> Item {
        Item {
            primary_key: self.struct_db_primary_key(),
            secondary_keys: self.struct_db_keys(),
            value: self.struct_db_bincode_encode_to_vec(),
        }
    }
}

pub trait KeyDefinition: Sized {
    fn secondary_table_name(&self) -> &'static str;
}

#[derive(Clone, Debug)]
pub(crate) struct BinaryValue(pub(crate) Vec<u8>);

impl BinaryValue {
    pub fn inner<T: SDBItem>(&self) -> T {
        T::struct_db_bincode_decode_from_slice(&self.0)
    }
}

#[derive(Debug)]
pub struct Item {
    pub(crate) primary_key: Vec<u8>,
    pub(crate) secondary_keys: std::collections::HashMap<&'static str, Vec<u8>>,
    pub(crate) value: Vec<u8>,
}
