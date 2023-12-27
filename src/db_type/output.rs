use crate::db_type::Input;

#[derive(Clone, Debug)]
pub struct DatabaseOutputValue(pub(crate) Vec<u8>);

impl From<&[u8]> for DatabaseOutputValue {
    fn from(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

impl DatabaseOutputValue {
    pub fn inner<T: Input>(&self) -> T {
        T::native_db_bincode_decode_from_slice(&self.0)
    }
}

pub fn unwrap_item<T: Input>(item: Option<redb::AccessGuard<'_, &'static [u8]>>) -> Option<T> {
    item.map_or_else(
        || None,
        |item| {
            let item = item.value();
            let item = T::native_db_bincode_decode_from_slice(item);
            Some(item)
        },
    )
}
