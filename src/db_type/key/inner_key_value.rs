use redb::{RedbKey, RedbValue, TypeName};
use std::fmt::Debug;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseInnerKeyValue(Vec<u8>);

impl DatabaseInnerKeyValue {
    fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub(crate) fn extend(&mut self, data: &Self) {
        self.0.extend(data.0.iter());
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

pub trait InnerKeyValue: Debug {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue;
}

// Implement for char
impl InnerKeyValue for char {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(u32::from(*self).to_be_bytes().to_vec())
    }
}

// Implement for String
impl InnerKeyValue for String {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        self.as_str().database_inner_key_value()
    }
}

// Implement for &str
impl InnerKeyValue for &str {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(self.as_bytes().to_vec())
    }
}

impl InnerKeyValue for DatabaseInnerKeyValue {
    // TODO: Bad because that cause a copy of the data when we pass a DatabaseInnerKeyValue to a function
    //       which has a impl InnerKeyValue parameter
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        self.clone()
    }
}

// Implement for Slice
impl<T> InnerKeyValue for &[T]
where
    T: InnerKeyValue,
{
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        let mut data = Vec::new();
        for item in self.iter().as_slice() {
            data.extend(item.database_inner_key_value().0);
        }
        DatabaseInnerKeyValue::new(data)
    }
}

// Implement for tuples
impl InnerKeyValue for () {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(Vec::new())
    }
}

// Macro for tuples
macro_rules! impl_inner_key_value_for_tuple {
    ( $($t:ident, $i:tt),+ | $t_last:ident, $i_last:tt ) => {
        impl<$($t: InnerKeyValue,)+ $t_last: InnerKeyValue> InnerKeyValue for ($($t,)+ $t_last) {
            fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
                let mut data = Vec::new();
                $(
                    data.extend(self.$i.database_inner_key_value().0);
                )+
                data.extend(self.$i_last.database_inner_key_value().0);
                DatabaseInnerKeyValue::new(data)
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
impl<T> InnerKeyValue for Vec<T>
where
    T: InnerKeyValue,
{
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        let mut data = Vec::new();
        for item in self {
            data.extend(item.database_inner_key_value().0);
        }
        DatabaseInnerKeyValue::new(data)
    }
}

// Implement InnerKeyValue for Option<T> where T: InnerKeyValue
impl<T> InnerKeyValue for Option<T>
where
    T: InnerKeyValue,
{
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        self.as_ref().map_or_else(
            || DatabaseInnerKeyValue::new(Vec::new()),
            |value| value.database_inner_key_value(),
        )
    }
}

// Macro for implementing InnerKeyValue for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
macro_rules! impl_inner_key_value_for_primitive {
    ($type:ty) => {
        impl InnerKeyValue for $type {
            fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
                DatabaseInnerKeyValue::new(self.to_be_bytes().to_vec())
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
impl InnerKeyValue for uuid::Uuid {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(self.as_bytes().to_vec())
    }
}

#[cfg(feature = "uuid")]
impl InnerKeyValue for &uuid::Uuid {
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(self.as_bytes().to_vec())
    }
}

// Implement chrono::DateTime<TZ>

#[cfg(feature = "chrono")]
impl<TZ> InnerKeyValue for chrono::DateTime<TZ>
where
    TZ: chrono::TimeZone,
{
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(self.timestamp().to_be_bytes().to_vec())
    }
}

#[cfg(feature = "chrono")]
impl<TZ> InnerKeyValue for &chrono::DateTime<TZ>
where
    TZ: chrono::TimeZone,
{
    fn database_inner_key_value(&self) -> DatabaseInnerKeyValue {
        DatabaseInnerKeyValue::new(self.timestamp().to_be_bytes().to_vec())
    }
}

impl RedbValue for DatabaseInnerKeyValue {
    type SelfType<'a> = Self;
    type AsBytes<'a> = &'a [u8] where Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        data.database_inner_key_value()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a + 'b,
    {
        value.0.as_slice()
    }

    fn type_name() -> TypeName {
        TypeName::new("DatabaseInnerKeyValue")
    }
}

impl RedbKey for DatabaseInnerKeyValue {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

#[derive(Debug)]
pub enum DatabaseInnerKeyValueRange {
    Range(Range<DatabaseInnerKeyValue>),
    RangeInclusive(RangeInclusive<DatabaseInnerKeyValue>),
    RangeFrom(RangeFrom<DatabaseInnerKeyValue>),
    RangeTo(RangeTo<DatabaseInnerKeyValue>),
    RangeToInclusive(RangeToInclusive<DatabaseInnerKeyValue>),
    RangeFull,
}

impl DatabaseInnerKeyValueRange {
    pub fn new<T>(bounds: impl RangeBounds<T>) -> Self
    where
        T: InnerKeyValue,
    {
        // FIXME: Here are a lot of match arms with identical bodies to another arm
        //        this needs to be refactored
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Included(end)) => Self::RangeInclusive(
                start.database_inner_key_value()..=end.database_inner_key_value(),
            ),
            (Bound::Included(start), Bound::Excluded(end)) => {
                Self::Range(start.database_inner_key_value()..end.database_inner_key_value())
            }
            (Bound::Included(start), Bound::Unbounded) => Self::RangeFrom(RangeFrom {
                start: start.database_inner_key_value(),
            }),
            (Bound::Excluded(start), Bound::Included(end)) => Self::RangeInclusive(
                start.database_inner_key_value()..=end.database_inner_key_value(),
            ),
            (Bound::Excluded(start), Bound::Excluded(end)) => {
                Self::Range(start.database_inner_key_value()..end.database_inner_key_value())
            }
            (Bound::Excluded(start), Bound::Unbounded) => Self::RangeFrom(RangeFrom {
                start: start.database_inner_key_value(),
            }),
            (Bound::Unbounded, Bound::Included(end)) => Self::RangeTo(RangeTo {
                end: { end.database_inner_key_value() },
            }),
            (Bound::Unbounded, Bound::Excluded(end)) => Self::RangeTo(RangeTo {
                end: end.database_inner_key_value(),
            }),
            (Bound::Unbounded, Bound::Unbounded) => Self::RangeFull,
        }
    }
}

impl RangeBounds<DatabaseInnerKeyValue> for DatabaseInnerKeyValueRange {
    fn start_bound(&self) -> Bound<&DatabaseInnerKeyValue> {
        match self {
            Self::Range(range) => range.start_bound(),
            Self::RangeInclusive(range) => range.start_bound(),
            Self::RangeFrom(range) => range.start_bound(),
            Self::RangeTo(range) => range.start_bound(),
            Self::RangeToInclusive(range) => range.start_bound(),
            Self::RangeFull => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&DatabaseInnerKeyValue> {
        match self {
            Self::Range(range) => range.end_bound(),
            Self::RangeInclusive(range) => range.end_bound(),
            Self::RangeFrom(range) => range.end_bound(),
            Self::RangeTo(range) => range.end_bound(),
            Self::RangeToInclusive(range) => range.end_bound(),
            Self::RangeFull => Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::RangeBounds;

    fn range<T: InnerKeyValue, R: RangeBounds<T>>(range: R) -> DatabaseInnerKeyValueRange {
        DatabaseInnerKeyValueRange::new(range)
    }

    #[test]
    fn test_range() {
        use redb::{ReadableTable, TableDefinition};

        const TABLE: TableDefinition<'_, DatabaseInnerKeyValue, u64> =
            TableDefinition::new("my_data");

        let backend = redb::backends::InMemoryBackend::new();
        let db = redb::Database::builder()
            .create_with_backend(backend)
            .unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            _ = table.insert(0u32.database_inner_key_value(), &123).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(
            table
                .get(0u32.database_inner_key_value())
                .unwrap()
                .unwrap()
                .value(),
            123
        );

        let range = range(0..2);
        let iter = table.range::<DatabaseInnerKeyValue>(range).unwrap();
        assert_eq!(iter.count(), 1);
    }
}
