use crate::SDBItem;

pub(crate) fn unwrap_item<T: SDBItem>(item: Option<redb::AccessGuard<&'static [u8]>>) -> Option<T> {
    if let Some(item) = item {
        let item = item.value();
        let item = T::struct_db_bincode_decode_from_slice(item);
        Some(item)
    } else {
        None
    }
}
