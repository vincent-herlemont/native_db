use redb::{Key as RedbKey, TypeName, Value as RedbValue};
use serde::Serialize;
use std::fmt::Debug;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::u8;

use super::key_serializer::KeySerializer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(Vec<u8>);

impl Key {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub(crate) fn extend(&mut self, data: &Key) {
        self.0.extend(data.0.iter());
    }

    pub fn extend_with_delimiter(&mut self, delimiter: u8, data: &Key) {
        self.0.push(delimiter);
        self.0.extend(data.0.iter());
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Allow to use a type as a key in the database.
///
/// In the below example, we define a struct `City` and implement the `ToKey` trait for it.
/// It can be use for primary or/and secondary key, and any other type that require `City` as a key.
///
/// # Example
/// ```rust
/// use native_db::*;
/// use native_model::{native_model, Model};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize)]
/// struct City(String);
///
///
/// #[derive(Serialize, Deserialize)]
/// #[native_model(id=1, version=1)]
/// #[native_db]
/// struct Contry {
///     #[primary_key]
///     capital: City,
///     #[secondary_key(unique)]
///     bigest_city: City,
/// }
///
/// fn main() -> Result<(), db_type::Error> {
///     let mut models = Models::new();
///     models.define::<Contry>()?;
///     let db = Builder::new().create_in_memory(&models)?;
///     
///     // Open a read transaction
///     let r = db.r_transaction()?;
///     
///     // Get contry by the capital city (primary key)
///     let _us: Option<Contry> = r.get().primary(&City("Washington, D.C.".to_string()))?;
///
///     // Get contry by the bigest city (secondary key)
///     let _us: Option<Contry> = r.get().secondary(ContryKey::bigest_city,&City("New York".to_string()))?;
///     Ok(())
/// }
/// ```
pub trait ToKey: Debug {
    fn to_key(&self) -> Key;
}

impl ToKey for Key {
    // TODO: Bad because that cause a copy of the data when we pass a DatabaseInnerKeyValue to a function
    //       which has a impl InnerKeyValue parameter
    fn to_key(&self) -> Key {
        self.clone()
    }
}

impl<T: Serialize + Debug> ToKey for T {
    fn to_key(&self) -> Key {
        let mut serializer = KeySerializer::new();
        self.serialize(&mut serializer).unwrap();
        serializer.into()
    }
}

impl RedbValue for Key {
    type SelfType<'a> = Key;
    type AsBytes<'a> = &'a [u8] where Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        data.to_key()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.0.as_slice()
    }

    fn type_name() -> TypeName {
        TypeName::new("DatabaseInnerKeyValue")
    }
}

impl RedbKey for Key {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(&data2)
    }
}

pub enum KeyRange {
    Range(Range<Key>),
    RangeInclusive(RangeInclusive<Key>),
    RangeFrom(RangeFrom<Key>),
    RangeTo(RangeTo<Key>),
    RangeToInclusive(RangeToInclusive<Key>),
    RangeFull,
}

impl KeyRange {
    pub fn new<T>(bounds: impl RangeBounds<T>) -> KeyRange
    where
        T: ToKey,
    {
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Included(end)) => {
                let start = start.to_key();
                let mut end = end.to_key();
                // Add 255 to the end key to include the last key
                end.extend(&Key::new(vec![255]));
                KeyRange::RangeInclusive(start..=end)
            }
            (Bound::Included(start), Bound::Excluded(end)) => {
                KeyRange::Range(start.to_key()..end.to_key())
            }
            (Bound::Included(start), Bound::Unbounded) => KeyRange::RangeFrom(RangeFrom {
                start: start.to_key(),
            }),
            (Bound::Excluded(start), Bound::Included(end)) => {
                let start = start.to_key();
                let mut end = end.to_key();
                // Add 255 to the end key to include the last key
                end.extend(&Key::new(vec![255]));
                KeyRange::RangeInclusive(start..=end)
            }
            (Bound::Excluded(start), Bound::Excluded(end)) => {
                KeyRange::Range(start.to_key()..end.to_key())
            }
            (Bound::Excluded(start), Bound::Unbounded) => KeyRange::RangeFrom(RangeFrom {
                start: start.to_key(),
            }),
            (Bound::Unbounded, Bound::Included(end)) => KeyRange::RangeTo(RangeTo {
                end: { end.to_key() },
            }),
            (Bound::Unbounded, Bound::Excluded(end)) => {
                KeyRange::RangeTo(RangeTo { end: end.to_key() })
            }
            (Bound::Unbounded, Bound::Unbounded) => KeyRange::RangeFull,
        }
    }
}

impl RangeBounds<Key> for KeyRange {
    fn start_bound(&self) -> Bound<&Key> {
        match self {
            KeyRange::Range(range) => range.start_bound(),
            KeyRange::RangeInclusive(range) => range.start_bound(),
            KeyRange::RangeFrom(range) => range.start_bound(),
            KeyRange::RangeTo(range) => range.start_bound(),
            KeyRange::RangeToInclusive(range) => range.start_bound(),
            KeyRange::RangeFull => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&Key> {
        match self {
            KeyRange::Range(range) => range.end_bound(),
            KeyRange::RangeInclusive(range) => range.end_bound(),
            KeyRange::RangeFrom(range) => range.end_bound(),
            KeyRange::RangeTo(range) => range.end_bound(),
            KeyRange::RangeToInclusive(range) => range.end_bound(),
            KeyRange::RangeFull => Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::RangeBounds;

    fn range<T: ToKey, R: RangeBounds<T>>(range: R) -> KeyRange {
        let range = KeyRange::new(range);
        range
    }

    #[test]
    fn test_range() {
        use redb::TableDefinition;

        const TABLE: TableDefinition<Key, u64> = TableDefinition::new("my_data");

        let backend = redb::backends::InMemoryBackend::new();
        let db = redb::Database::builder()
            .create_with_backend(backend)
            .unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert(0u32.to_key(), &123).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(table.get(0u32.to_key()).unwrap().unwrap().value(), 123);

        let range = range(0..2);
        let iter = table.range::<Key>(range).unwrap();
        let result: Vec<_> = iter.collect();
        assert_eq!(result.len(), 1);
    }
}
