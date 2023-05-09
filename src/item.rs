pub trait SDBItem: Sized {
    fn struct_db_schema() -> crate::Schema;
    fn struct_db_primary_key(&self) -> Vec<u8>;
    fn struct_db_keys(&self) -> std::collections::HashMap<&'static str, Vec<u8>>;
    fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8>;
    fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self;
}

pub trait KeyDefinition: Sized {
    fn secondary_table_name(&self) -> &'static str;
}

#[derive(Clone)]
pub(crate) struct BinaryValue(pub(crate) Vec<u8>);

impl BinaryValue {
    pub fn inner<T: SDBItem>(&self) -> T {
        T::struct_db_bincode_decode_from_slice(&self.0)
    }
}
