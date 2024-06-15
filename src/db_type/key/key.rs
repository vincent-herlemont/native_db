use redb::{Key as RedbKey, TypeName, Value as RedbValue};
use std::fmt::Debug;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(Vec<u8>);

impl Key {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub(crate) fn extend(&mut self, data: &Key) {
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
/// impl ToKey for &City {
///    fn to_key(&self) -> Key {
///       Key::new(self.0.as_bytes().to_vec())
///   }
/// }
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
///     let _US: Option<Contry> = r.get().primary(&City("Washington, D.C.".to_string()))?;
///
///     // Get contry by the bigest city (secondary key)
///     let _US: Option<Contry> = r.get().secondary(ContryKey::bigest_city,&City("New York".to_string()))?;
///     Ok(())
/// }
/// ```
pub trait ToKey: Debug {
    fn to_key(&self) -> Key;
}

// Implement for char
impl ToKey for char {
    fn to_key(&self) -> Key {
        Key::new(u32::from(*self).to_be_bytes().to_vec())
    }
}

// Implement for String
impl ToKey for &String {
    fn to_key(&self) -> Key {
        self.as_str().to_key()
    }
}

// Implement for &str
impl ToKey for &str {
    fn to_key(&self) -> Key {
        Key::new(self.as_bytes().to_vec())
    }
}

impl ToKey for Key {
    // TODO: Bad because that cause a copy of the data when we pass a DatabaseInnerKeyValue to a function
    //       which has a impl InnerKeyValue parameter
    fn to_key(&self) -> Key {
        self.clone()
    }
}

// Implement for Slice
impl<T> ToKey for &[T]
where
    T: ToKey,
{
    fn to_key(&self) -> Key {
        let mut data = Vec::new();
        for item in self.iter().as_slice() {
            data.extend(item.to_key().0);
        }
        Key::new(data)
    }
}

// Implement for tuples
impl ToKey for () {
    fn to_key(&self) -> Key {
        Key::new(Vec::new())
    }
}

// Macro for tuples
macro_rules! impl_inner_key_value_for_tuple {
    ( $($t:ident, $i:tt),+ | $t_last:ident, $i_last:tt ) => {
        impl<$($t: ToKey,)+ $t_last: ToKey> ToKey for ($($t,)+ $t_last) {
            fn to_key(&self) -> Key {
                let mut data = Vec::new();
                $(
                    data.extend(self.$i.to_key().0);
                )+
                data.extend(self.$i_last.to_key().0);
                Key::new(data)
            }
        }
    }
}

// Implementations for tuples of different sizes
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0 | 
    T1, 1
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1 | 
    T2, 2
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1, 
    T2, 2 | T3, 3
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1, 
    T2, 2, T3, 3 | 
    T4, 4
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1,
    T2, 2, T3, 3, 
    T4, 4 | T5, 5
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1,
    T2, 2, T3, 3,
    T4, 4, T5, 5 
    | T6, 6
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1,
    T2, 2, T3, 3, 
    T4, 4, T5, 5,
    T6, 6 | T7, 7
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1,
    T2, 2, T3, 3,
    T4, 4, T5, 5,
    T6, 6, T7, 7 | 
    T8, 8
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1, 
    T2, 2, T3, 3,
    T4, 4, T5, 5,
    T6, 6, T7, 7,
    T8, 8 | T9, 9
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1, 
    T2, 2, T3, 3,
    T4, 4, T5, 5,
    T6, 6, T7, 7,
    T8, 8, T9, 9 | 
    T10, 10
);
#[rustfmt::skip]
impl_inner_key_value_for_tuple!(
    T0, 0, T1, 1,
    T2, 2, T3, 3,
    T4, 4, T5, 5,
    T6, 6, T7, 7,
    T8, 8, T9, 9,
    T10, 10 | T11, 11
);

// Implement InnerKeyValue for Vec<T> where T: InnerKeyValue
impl<T> ToKey for Vec<T>
where
    T: ToKey,
{
    fn to_key(&self) -> Key {
        let mut data = Vec::new();
        for item in self {
            data.extend(item.to_key().0);
        }
        Key::new(data)
    }
}

// Implement InnerKeyValue for Option<T> where T: InnerKeyValue
impl<T> ToKey for Option<T>
where
    T: ToKey,
{
    fn to_key(&self) -> Key {
        match self {
            Some(value) => value.to_key(),
            None => Key::new(Vec::new()),
        }
    }
}

// Macro for implementing InnerKeyValue for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
macro_rules! impl_inner_key_value_for_primitive {
    ($type:ty) => {
        impl ToKey for $type {
            fn to_key(&self) -> Key {
                Key::new(self.to_be_bytes().to_vec())
            }
        }
    };
}

impl_inner_key_value_for_primitive!(u8);
impl_inner_key_value_for_primitive!(u16);
impl_inner_key_value_for_primitive!(u32);
impl_inner_key_value_for_primitive!(u64);
impl_inner_key_value_for_primitive!(u128);
impl_inner_key_value_for_primitive!(i8);
impl_inner_key_value_for_primitive!(i16);
impl_inner_key_value_for_primitive!(i32);
impl_inner_key_value_for_primitive!(i64);
impl_inner_key_value_for_primitive!(i128);
impl_inner_key_value_for_primitive!(f32);
impl_inner_key_value_for_primitive!(f64);

// Implement Uuid::uuid

#[cfg(feature = "uuid")]
impl ToKey for &uuid::Uuid {
    fn to_key(&self) -> Key {
        Key::new(self.as_bytes().to_vec())
    }
}

// Implement chrono::DateTime<TZ>

#[cfg(feature = "chrono")]
impl<TZ> ToKey for &chrono::DateTime<TZ>
where
    TZ: chrono::TimeZone,
{
    fn to_key(&self) -> Key {
        Key::new(self.timestamp().to_be_bytes().to_vec())
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
                KeyRange::RangeInclusive(start.to_key()..=end.to_key())
            }
            (Bound::Included(start), Bound::Excluded(end)) => {
                KeyRange::Range(start.to_key()..end.to_key())
            }
            (Bound::Included(start), Bound::Unbounded) => KeyRange::RangeFrom(RangeFrom {
                start: start.to_key(),
            }),
            (Bound::Excluded(start), Bound::Included(end)) => {
                KeyRange::RangeInclusive(start.to_key()..=end.to_key())
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
