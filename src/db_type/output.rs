use crate::db_type::{Input, Result};

#[derive(Clone, Debug)]
pub(crate) struct Output(pub(crate) Vec<u8>);

impl From<&[u8]> for Output {
    fn from(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

impl Output {
    pub fn inner<T: Input>(&self) -> Result<T> {
        T::native_db_bincode_decode_from_slice(&self.0)
    }
}

pub(crate) fn unwrap_item<T: Input>(
    item: Option<redb::AccessGuard<&'static [u8]>>,
) -> Option<Result<T>> {
    if let Some(item) = item {
        let item = item.value();
        let item = T::native_db_bincode_decode_from_slice(item);
        Some(item)
    } else {
        None
    }
}
