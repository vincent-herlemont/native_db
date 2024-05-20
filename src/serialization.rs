pub fn bincode_encode_to_vec<T>(value: &T) -> crate::db_type::Result<Vec<u8>>
where
    T: serde::Serialize + native_model::Model,
{
    native_model::encode(value).map_err(|e| e.into())
}

pub fn bincode_decode_from_slice<T>(slice: &[u8]) -> crate::db_type::Result<(T, usize)>
where
    T: serde::de::DeserializeOwned + native_model::Model,
{
    let (data, _) = native_model::decode(slice.to_vec())?;
    Ok((data, 0))
}
