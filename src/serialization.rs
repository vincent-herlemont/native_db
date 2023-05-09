pub fn bincode_encode_to_vec<T>(value: &T) -> Option<Vec<u8>>
where
    T: serde::Serialize,
{
    bincode::serde::encode_to_vec(value, bincode::config::standard()).ok()
}

pub fn bincode_decode_from_slice<T>(slice: &[u8]) -> Option<(T, usize)>
where
    T: serde::de::DeserializeOwned,
{
    bincode::serde::decode_from_slice(slice, bincode::config::standard()).ok()
}
