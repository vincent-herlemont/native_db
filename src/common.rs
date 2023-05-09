use crate::SDBItem;

pub(crate) fn get<T: SDBItem>(value: Option<redb::AccessGuard<&'static [u8]>>) -> Option<T> {
    if let Some(value) = value {
        let value = value.value();
        let value = T::struct_db_bincode_decode_from_slice(value);
        Some(value)
    } else {
        None
    }
}
