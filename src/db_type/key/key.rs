use redb::{Key as RedbKey, TypeName, Value as RedbValue};
use std::fmt::Debug;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Key(Vec<u8>);

impl Key {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
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
/// ## Example
/// ```rust
/// use native_db::*;
/// use native_db::native_model::{native_model, Model};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize)]
/// struct City(String);
///
/// impl ToKey for City {
///   fn to_key(&self) -> Key {
///       Key::new(self.0.as_bytes().to_vec())
///   }
///
///   fn key_names() -> Vec<String> {
///     vec!["City".to_string()]
///   }
/// }
///
/// #[derive(Serialize, Deserialize)]
/// #[native_model(id=1, version=1)]
/// #[native_db]
/// struct Country {
///     #[primary_key]
///     capital: City,
///     #[secondary_key(unique)]
///     bigest_city: City,
/// }
///
/// fn main() -> Result<(), db_type::Error> {
///     let mut models = Models::new();
///     models.define::<Country>()?;
///     let db = Builder::new().create_in_memory(&models)?;
///     
///     // Open a read transaction
///     let r = db.r_transaction()?;
///     
///     // Get contry by the capital city (primary key)
///     let _us: Option<Country> = r.get().primary(City("Washington, D.C.".to_string()))?;
///
///     // Get contry by the bigest city (secondary key)
///     let _us: Option<Country> = r.get().secondary(CountryKey::bigest_city, City("New York".to_string()))?;
///     Ok(())
/// }
/// ```
///
/// ## Example with `Uuid`
///
/// You can use [uuid](https://crates.io/crates/uuid) crate to generate a `Uuid` key.
///
/// ```rust
/// use native_db::*;
/// use native_db::native_model::{native_model, Model};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
/// struct Uuid(uuid::Uuid);
///
/// impl ToKey for Uuid {
///     fn to_key(&self) -> Key {
///         Key::new(self.0.as_bytes().to_vec())
///     }
///
///     fn key_names() -> Vec<String> {
///         vec!["Uuid".to_string()]
///     }
/// }
///
/// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
/// #[native_model(id = 1, version = 1)]
/// #[native_db]
/// struct Item {
///     #[primary_key]
///     uuid: Uuid,
/// }
///
/// fn main() -> Result<(), db_type::Error> {
///     let mut models = Models::new();
///     models.define::<Item>()?;
///     let db = Builder::new().create_in_memory(&models)?;
///     
///     let rw = db.rw_transaction()?;
///     let item = Item { uuid: Uuid(uuid::Uuid::new_v4()) };
///     rw.insert(item.clone())?;
///     rw.commit()?;
///
///     let r = db.r_transaction()?;
///     let result_item: Item = r.get().primary(item.uuid.clone())?.unwrap();
///     assert_eq!(result_item.uuid, item.uuid);
///     Ok(())
/// }
/// ```
///
/// ## Example with `chrono`
///
/// You can use [chrono](https://crates.io/crates/chrono) crate to generate a `chrono::DateTime` key.
///
/// ```rust
/// use native_db::*;
/// use native_db::native_model::{native_model, Model};
/// use serde::{Deserialize, Serialize};
/// use itertools::Itertools;
///
/// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
/// struct DateTime(chrono::DateTime<chrono::Utc>);
///
/// impl ToKey for DateTime {
///     fn to_key(&self) -> Key {
///         Key::new(self.0.timestamp_millis().to_be_bytes().to_vec())
///     }
///
///     fn key_names() -> Vec<String> {
///         vec!["DateTime".to_string()]
///     }
/// }
///
/// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
/// #[native_model(id = 1, version = 1)]
/// #[native_db]
/// struct Item {
///     #[primary_key]
///     id: u32,
///     #[secondary_key]
///     created_at: DateTime,
/// }
///
/// fn main() -> Result<(), db_type::Error> {
///     let mut models = Models::new();
///     models.define::<Item>()?;
///     let db = Builder::new().create_in_memory(&models)?;
///     
///     let rw = db.rw_transaction()?;
///     let item1 = Item { id: 2, created_at: DateTime(chrono::Utc::now()) };
///     rw.insert(item1.clone())?;
///     std::thread::sleep(std::time::Duration::from_millis(2));
///
///     let item2 = Item { id: 1, created_at: DateTime(chrono::Utc::now()) };
///     rw.insert(item2.clone())?;
///     rw.commit()?;    
///     
///     let r = db.r_transaction()?;
///     let result_items: Vec<Item> = r.scan().secondary(ItemKey::created_at)?.all()?.try_collect()?;
///     assert_eq!(result_items.len(), 2);
///     assert_eq!(result_items[0].id, 2);
///     assert_eq!(result_items[1].id, 1);
///     Ok(())
/// }
///
/// ```
pub trait ToKey: Debug {
    fn to_key(&self) -> Key;
    fn key_names() -> Vec<String>;

    /// Whether the key's type will be checked at runtime.
    fn check_type() -> bool {
        true
    }
}

// Implement for char
impl ToKey for char {
    fn to_key(&self) -> Key {
        Key::new(u32::from(*self).to_be_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["char".to_string()]
    }
}

// Implement for &String
impl ToKey for String {
    fn to_key(&self) -> Key {
        self.as_str().to_key()
    }
    fn key_names() -> Vec<String> {
        vec!["String".to_string()]
    }
}

// Implement for &str
impl ToKey for &str {
    fn to_key(&self) -> Key {
        Key::new(self.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["String".to_string(), "&str".to_string()]
    }
}

impl ToKey for Key {
    // TODO: Bad because that cause a copy of the data when we pass a DatabaseInnerKeyValue to a function
    //       which has a impl InnerKeyValue parameter
    fn to_key(&self) -> Key {
        self.clone()
    }

    fn key_names() -> Vec<String> {
        vec!["Key".to_string()]
    }

    fn check_type() -> bool {
        // Disable type checking for generic key because otherwise you can't query
        // with a `Key` for a key whose type is not `Key`.
        false
    }
}

// Implement for tuples
impl ToKey for () {
    fn to_key(&self) -> Key {
        Key::new(Vec::new())
    }
    fn key_names() -> Vec<String> {
        vec!["()".to_string()]
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
            fn key_names() -> Vec<String> {
                let mut name = String::new();
                $(
                    name.push_str(<$t as ToKey>::key_names()[0].as_str());
                    name.push_str(", ");
                )+
                name.push_str(<$t_last as ToKey>::key_names()[0].as_str());
                vec![format!("({})", name)]
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
    fn key_names() -> Vec<String> {
        let mut names = Vec::new();
        for name in T::key_names() {
            names.push(format!("Vec<{name}>"));
            names.push(format!("[{name}]"));
        }
        names
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
    fn key_names() -> Vec<String> {
        let mut names = Vec::new();
        for name in T::key_names() {
            names.push(format!("[{name}]"));
        }

        names
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
    fn key_names() -> Vec<String> {
        let mut names = Vec::new();
        for name in T::key_names() {
            names.push(format!("Option<{name}>"));
        }
        names
    }
}

// Macro for implementing InnerKeyValue for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
macro_rules! impl_inner_key_value_for_primitive {
    ($type:ty) => {
        impl ToKey for $type {
            fn to_key(&self) -> Key {
                Key::new(self.to_be_bytes().to_vec())
            }
            fn key_names() -> Vec<String> {
                vec![stringify!($type).to_string()]
            }
        }
    };
}

impl_inner_key_value_for_primitive!(u8);
impl_inner_key_value_for_primitive!(u16);
impl_inner_key_value_for_primitive!(u32);
impl_inner_key_value_for_primitive!(u64);
impl_inner_key_value_for_primitive!(u128);
// Implement ToKey for signed integers with order-preserving encoding
impl ToKey for i8 {
    fn to_key(&self) -> Key {
        Key::new(((*self as u8) ^ 0x80).to_be_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["i8".to_string()]
    }
}

impl ToKey for i16 {
    fn to_key(&self) -> Key {
        Key::new(((*self as u16) ^ 0x8000).to_be_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["i16".to_string()]
    }
}

impl ToKey for i32 {
    fn to_key(&self) -> Key {
        Key::new(((*self as u32) ^ 0x80000000).to_be_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["i32".to_string()]
    }
}

impl ToKey for i64 {
    fn to_key(&self) -> Key {
        Key::new(((*self as u64) ^ 0x8000000000000000).to_be_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["i64".to_string()]
    }
}

impl ToKey for i128 {
    fn to_key(&self) -> Key {
        Key::new(
            ((*self as u128) ^ 0x80000000000000000000000000000000)
                .to_be_bytes()
                .to_vec(),
        )
    }
    fn key_names() -> Vec<String> {
        vec!["i128".to_string()]
    }
}
impl_inner_key_value_for_primitive!(f32);
impl_inner_key_value_for_primitive!(f64);

impl ToKey for bool {
    fn to_key(&self) -> Key {
        Key::new(vec![*self as u8])
    }

    fn key_names() -> Vec<String> {
        vec!["bool".to_string()]
    }
}

impl RedbValue for Key {
    type SelfType<'a> = Key;
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;

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
        data1.cmp(data2)
    }
}

#[derive(Clone, PartialEq, Eq)]
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
            (Bound::Excluded(_), Bound::Included(_)) => {
                unreachable!("Excluded => Included bound is not supported")
            }
            (Bound::Excluded(_), Bound::Excluded(_)) => {
                unreachable!("Excluded => Excluded bound is not supported")
            }
            (Bound::Excluded(_), Bound::Unbounded) => {
                unreachable!("Excluded => Unbounded bound is not supported")
            }
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
        KeyRange::new(range)
    }

    #[test]
    fn test_range() {
        use redb::{ReadableDatabase, TableDefinition};

        const TABLE: TableDefinition<Key, u64> = TableDefinition::new("my_data");

        let backend = redb::backends::InMemoryBackend::new();
        let db = redb::Database::builder()
            .create_with_backend(backend)
            .unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert(0i32.to_key(), &123).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(table.get(0i32.to_key()).unwrap().unwrap().value(), 123);

        let range = range(0i32..2i32);
        let iter = table.range::<Key>(range).unwrap();
        let result: Vec<_> = iter.collect();
        assert_eq!(result.len(), 1);
    }
}
