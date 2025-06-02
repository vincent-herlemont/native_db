#![allow(dead_code)]

/// This module contains a legacy implementation of the `InnerKeyValue` trait for the `redb1` crate.
use super::Key as NewDatabaseInnerKeyValue;
use redb1::{RedbKey as Key, RedbValue as Value, TypeName};
use std::fmt::Debug;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseInnerKeyValue(Vec<u8>);

impl DatabaseInnerKeyValue {
    fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub(crate) fn extend(&mut self, data: &DatabaseInnerKeyValue) {
        self.0.extend(data.0.iter());
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<DatabaseInnerKeyValue> for NewDatabaseInnerKeyValue {
    fn from(data: DatabaseInnerKeyValue) -> Self {
        NewDatabaseInnerKeyValue::new(data.0)
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
        match self {
            Some(value) => value.database_inner_key_value(),
            None => DatabaseInnerKeyValue::new(Vec::new()),
        }
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

impl Value for DatabaseInnerKeyValue {
    type SelfType<'a> = DatabaseInnerKeyValue;
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
        data.database_inner_key_value()
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

impl Key for DatabaseInnerKeyValue {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

pub enum DatabaseInnerKeyValueRange {
    Range(Range<DatabaseInnerKeyValue>),
    RangeInclusive(RangeInclusive<DatabaseInnerKeyValue>),
    RangeFrom(RangeFrom<DatabaseInnerKeyValue>),
    RangeTo(RangeTo<DatabaseInnerKeyValue>),
    RangeToInclusive(RangeToInclusive<DatabaseInnerKeyValue>),
    RangeFull,
}

impl DatabaseInnerKeyValueRange {
    pub fn new<T>(bounds: impl RangeBounds<T>) -> DatabaseInnerKeyValueRange
    where
        T: InnerKeyValue,
    {
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Included(end)) => {
                DatabaseInnerKeyValueRange::RangeInclusive(
                    start.database_inner_key_value()..=end.database_inner_key_value(),
                )
            }
            (Bound::Included(start), Bound::Excluded(end)) => DatabaseInnerKeyValueRange::Range(
                start.database_inner_key_value()..end.database_inner_key_value(),
            ),
            (Bound::Included(start), Bound::Unbounded) => {
                DatabaseInnerKeyValueRange::RangeFrom(RangeFrom {
                    start: start.database_inner_key_value(),
                })
            }
            (Bound::Excluded(start), Bound::Included(end)) => {
                DatabaseInnerKeyValueRange::RangeInclusive(
                    start.database_inner_key_value()..=end.database_inner_key_value(),
                )
            }
            (Bound::Excluded(start), Bound::Excluded(end)) => DatabaseInnerKeyValueRange::Range(
                start.database_inner_key_value()..end.database_inner_key_value(),
            ),
            (Bound::Excluded(start), Bound::Unbounded) => {
                DatabaseInnerKeyValueRange::RangeFrom(RangeFrom {
                    start: start.database_inner_key_value(),
                })
            }
            (Bound::Unbounded, Bound::Included(end)) => {
                DatabaseInnerKeyValueRange::RangeTo(RangeTo {
                    end: { end.database_inner_key_value() },
                })
            }
            (Bound::Unbounded, Bound::Excluded(end)) => {
                DatabaseInnerKeyValueRange::RangeTo(RangeTo {
                    end: end.database_inner_key_value(),
                })
            }
            (Bound::Unbounded, Bound::Unbounded) => DatabaseInnerKeyValueRange::RangeFull,
        }
    }
}

impl RangeBounds<DatabaseInnerKeyValue> for DatabaseInnerKeyValueRange {
    fn start_bound(&self) -> Bound<&DatabaseInnerKeyValue> {
        match self {
            DatabaseInnerKeyValueRange::Range(range) => range.start_bound(),
            DatabaseInnerKeyValueRange::RangeInclusive(range) => range.start_bound(),
            DatabaseInnerKeyValueRange::RangeFrom(range) => range.start_bound(),
            DatabaseInnerKeyValueRange::RangeTo(range) => range.start_bound(),
            DatabaseInnerKeyValueRange::RangeToInclusive(range) => range.start_bound(),
            DatabaseInnerKeyValueRange::RangeFull => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&DatabaseInnerKeyValue> {
        match self {
            DatabaseInnerKeyValueRange::Range(range) => range.end_bound(),
            DatabaseInnerKeyValueRange::RangeInclusive(range) => range.end_bound(),
            DatabaseInnerKeyValueRange::RangeFrom(range) => range.end_bound(),
            DatabaseInnerKeyValueRange::RangeTo(range) => range.end_bound(),
            DatabaseInnerKeyValueRange::RangeToInclusive(range) => range.end_bound(),
            DatabaseInnerKeyValueRange::RangeFull => Bound::Unbounded,
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
        use redb1::ReadableTable;
        use redb1::TableDefinition;

        const TABLE: TableDefinition<DatabaseInnerKeyValue, u64> = TableDefinition::new("my_data");

        let backend = redb1::backends::InMemoryBackend::new();
        let db = redb1::Database::builder()
            .create_with_backend(backend)
            .unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert(0u32.database_inner_key_value(), &123).unwrap();
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
        let result: Vec<_> = iter.collect();
        assert_eq!(result.len(), 1);
    }
}
