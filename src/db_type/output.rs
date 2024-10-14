use crate::db_type::{Result, ToInput};

use super::Input;

#[derive(Clone, Debug)]
pub(crate) struct Output(pub(crate) Vec<u8>);

impl From<Input> for Output {
    fn from(input: Input) -> Self {
        Self(input.value)
    }
}

impl From<&[u8]> for Output {
    fn from(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

impl Output {
    pub fn inner<T: ToInput>(&self) -> Result<T> {
        T::native_db_bincode_decode_from_slice(&self.0)
    }
}

pub(crate) fn unwrap_item<T: ToInput>(
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
