pub fn bincode_encode_to_vec<T>(value: &T) -> Option<Vec<u8>>
where
    T: serde::Serialize + native_model::Model,
{
    native_model::encode(value).ok()
}

pub fn bincode_decode_from_slice<T>(slice: &[u8]) -> Option<(T, usize)>
where
    T: serde::de::DeserializeOwned + native_model::Model,
{
    let (data, _) = native_model::decode(slice.to_vec()).ok()?;
    Some((data, 0))
}
