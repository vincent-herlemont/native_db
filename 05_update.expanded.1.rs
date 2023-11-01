#![feature(prelude_import)]
#![cfg(not(feature = "native_model"))]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
mod tests {
    use shortcut_assert_fs::TmpFs;
    #[allow(dead_code)]
    pub fn init() -> TmpFs {
        TmpFs::new().unwrap()
    }
}
use serde::{Deserialize, Serialize};
use struct_db::*;
struct Item(u32);
impl Item {
    fn is_native_model() -> bool {
        false
    }
}
impl struct_db::SDBItem for Item {
    fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8> {
        struct_db::bincode_encode_to_vec(self).expect("Failed to serialize the struct #struct_name")
    }
    fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self {
        struct_db::bincode_decode_from_slice(slice)
            .expect("Failed to deserialize the struct #struct_name")
            .0
    }
    fn struct_db_schema() -> struct_db::Schema {
        let mut secondary_tables_name = std::collections::HashSet::new();
        struct_db::Schema {
            table_name: "item",
            primary_key: "p_key",
            secondary_tables_name: secondary_tables_name,
        }
    }
    fn struct_db_pk(&self) -> Vec<u8> {
        self.p_key()
    }
    fn struct_db_gks(&self) -> std::collections::HashMap<&'static str, Vec<u8>> {
        let mut secondary_tables_name = std::collections::HashMap::new();
        secondary_tables_name
    }
}
/// Index selection Enum for [#struct_name]
pub(crate) enum ItemKey {}
impl struct_db::KeyDefinition for ItemKey {
    fn secondary_table_name(&self) -> &'static str {
        match self {
            _ => {
                ::std::rt::begin_panic("Unknown key");
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Item {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serializer::serialize_newtype_struct(__serializer, "Item", &self.0)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Item {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Item>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Item;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "tuple struct Item")
                }
                #[inline]
                fn visit_newtype_struct<__E>(
                    self,
                    __e: __E,
                ) -> _serde::__private::Result<Self::Value, __E::Error>
                where
                    __E: _serde::Deserializer<'de>,
                {
                    let __field0: u32 = <u32 as _serde::Deserialize>::deserialize(__e)?;
                    _serde::__private::Ok(Item(__field0))
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct Item with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Item(__field0))
                }
            }
            _serde::Deserializer::deserialize_newtype_struct(
                __deserializer,
                "Item",
                __Visitor {
                    marker: _serde::__private::PhantomData::<Item>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralEq for Item {}
#[automatically_derived]
impl ::core::cmp::Eq for Item {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u32>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Item {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Item {
    #[inline]
    fn eq(&self, other: &Item) -> bool {
        self.0 == other.0
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Item {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Item", &&self.0)
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Item {
    #[inline]
    fn clone(&self) -> Item {
        Item(::core::clone::Clone::clone(&self.0))
    }
}
impl Item {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "update"]
pub const update: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("update"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/05_update.rs",
        start_line: 18usize,
        start_col: 4usize,
        end_line: 18usize,
        end_col: 10usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(update())),
};
fn update() {
    let tf = tests::init();
    let o_v1 = Item(1);
    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>();
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, o_v1.clone()).unwrap();
    }
    tx.commit().unwrap();
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Item = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap().unwrap();
        match (&o_v1, &o2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let o_v2 = Item(2);
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.update(&tx, o_v1.clone(), o_v2.clone()).unwrap();
    }
    tx.commit().unwrap();
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<Item> = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap();
        match (&o2, &None) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Item = tables.primary_get(&tx_r, &o_v2.p_key()).unwrap().unwrap();
        match (&o_v2, &o2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
}
struct Item1K(u32, String);
impl Item1K {
    fn is_native_model() -> bool {
        false
    }
}
impl struct_db::SDBItem for Item1K {
    fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8> {
        struct_db::bincode_encode_to_vec(self).expect("Failed to serialize the struct #struct_name")
    }
    fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self {
        struct_db::bincode_decode_from_slice(slice)
            .expect("Failed to deserialize the struct #struct_name")
            .0
    }
    fn struct_db_schema() -> struct_db::Schema {
        let mut secondary_tables_name = std::collections::HashSet::new();
        secondary_tables_name.insert("item1k_s_key");
        struct_db::Schema {
            table_name: "item1k",
            primary_key: "p_key",
            secondary_tables_name: secondary_tables_name,
        }
    }
    fn struct_db_pk(&self) -> Vec<u8> {
        self.p_key()
    }
    fn struct_db_gks(&self) -> std::collections::HashMap<&'static str, Vec<u8>> {
        let mut secondary_tables_name = std::collections::HashMap::new();
        secondary_tables_name.insert("item1k_s_key", self.s_key());
        secondary_tables_name
    }
}
/// Index selection Enum for [#struct_name]
pub(crate) enum Item1KKey {
    s_key,
}
impl struct_db::KeyDefinition for Item1KKey {
    fn secondary_table_name(&self) -> &'static str {
        match self {
            _ => {
                ::std::rt::begin_panic("Unknown key");
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Item1K {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_tuple_struct(__serializer, "Item1K", 0 + 1 + 1)?;
            _serde::ser::SerializeTupleStruct::serialize_field(&mut __serde_state, &self.0)?;
            _serde::ser::SerializeTupleStruct::serialize_field(&mut __serde_state, &self.1)?;
            _serde::ser::SerializeTupleStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Item1K {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Item1K>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Item1K;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "tuple struct Item1K")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct Item1K with 2 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"tuple struct Item1K with 2 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(Item1K(__field0, __field1))
                }
            }
            _serde::Deserializer::deserialize_tuple_struct(
                __deserializer,
                "Item1K",
                2usize,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Item1K>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralEq for Item1K {}
#[automatically_derived]
impl ::core::cmp::Eq for Item1K {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u32>;
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Item1K {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Item1K {
    #[inline]
    fn eq(&self, other: &Item1K) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Item1K {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field2_finish(f, "Item1K", &self.0, &&self.1)
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Item1K {
    #[inline]
    fn clone(&self) -> Item1K {
        Item1K(
            ::core::clone::Clone::clone(&self.0),
            ::core::clone::Clone::clone(&self.1),
        )
    }
}
impl Item1K {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
    pub fn s_key(&self) -> Vec<u8> {
        self.1.as_bytes().to_vec()
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "update_1k"]
pub const update_1k: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("update_1k"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/05_update.rs",
        start_line: 84usize,
        start_col: 4usize,
        end_line: 84usize,
        end_col: 13usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(update_1k())),
};
fn update_1k() {
    let tf = tests::init();
    let o_v1 = Item1K(1, "1".to_string());
    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item1K>();
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, o_v1.clone()).unwrap();
    }
    tx.commit().unwrap();
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Item1K = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap().unwrap();
        match (&o_v1, &o2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Item1K = tables
            .secondary_get(&tx_r, Item1KKey::s_key, &o_v1.s_key())
            .unwrap()
            .unwrap();
        match (&o_v1, &o2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let o_v2 = Item1K(2, "2".to_string());
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.update(&tx, o_v1.clone(), o_v2.clone()).unwrap();
    }
    tx.commit().unwrap();
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<Item1K> = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap();
        match (&o2, &None) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<Item1K> = tables
            .secondary_get(&tx_r, Item1KKey::s_key, &o_v1.s_key())
            .unwrap();
        match (&o2, &None) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Item1K = tables.primary_get(&tx_r, &o_v2.p_key()).unwrap().unwrap();
        match (&o_v2, &o2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
}
#[rustc_main]
#[no_coverage]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&update, &update_1k])
}
