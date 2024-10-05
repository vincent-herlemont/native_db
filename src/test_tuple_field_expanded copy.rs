### parsed_type: "(String, u32)"
### parsed_type_token_stream: TokenStream [Group { delimiter: Parenthesis, stream: TokenStream [Ident { ident: "String", span: #21 bytes(197..209) }, Punct { ch: ',', spacing: Alone, span: #21 bytes(197..209) }, Ident { ident: "u32", span: #21 bytes(197..209) }], span: #21 bytes(197..209) }]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
struct ItemCustomPk {
    #[primary_key]
    id: (String, u32),
}
impl native_db::db_type::ToInput for ItemCustomPk {
    fn native_db_bincode_encode_to_vec(&self) -> native_db::db_type::Result<Vec<u8>> {
        native_db::bincode_encode_to_vec(self)
    }
    fn native_db_bincode_decode_from_slice(
        slice: &[u8],
    ) -> native_db::db_type::Result<Self> {
        Ok(native_db::bincode_decode_from_slice(slice)?.0)
    }
    fn native_db_model() -> native_db::Model {
        let mut secondary_tables_name = std::collections::HashSet::new();
        native_db::Model {
            primary_key: native_db::db_type::KeyDefinition::new(
                ItemCustomPk::native_model_id(),
                ItemCustomPk::native_model_version(),
                "id",
                <(String, u32)>::key_names(),
                (),
            ),
            secondary_keys: secondary_tables_name,
        }
    }
    fn native_db_primary_key(&self) -> native_db::db_type::Key {
        (&self.id).to_key()
    }
    fn native_db_secondary_keys(
        &self,
    ) -> std::collections::HashMap<
        native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions>,
        native_db::db_type::KeyEntry,
    > {
        let mut secondary_tables_name = std::collections::HashMap::new();
        secondary_tables_name
    }
}
pub(crate) enum ItemCustomPkKey {}
impl native_db::db_type::ToKeyDefinition<native_db::db_type::KeyOptions>
for ItemCustomPkKey {
    fn key_definition(
        &self,
    ) -> native_db::db_type::KeyDefinition<native_db::db_type::KeyOptions> {
        match self {
            _ => {
                ::std::rt::begin_panic("Unknown key");
            }
        }
    }
}
impl native_model::Model for ItemCustomPk {
    fn native_model_id() -> u32 {
        1
    }
    fn native_model_id_str() -> &'static str {
        "1"
    }
    fn native_model_version() -> u32 {
        1
    }
    fn native_model_version_str() -> &'static str {
        "1"
    }
    fn native_model_encode_body(
        &self,
    ) -> std::result::Result<Vec<u8>, native_model::EncodeBodyError> {
        use native_model::Encode;
        native_model::bincode_1_3::Bincode::encode(self)
            .map_err(|e| native_model::EncodeBodyError {
                msg: {
                    let res = ::alloc::fmt::format(format_args!("{0}", e));
                    res
                },
                source: e.into(),
            })
    }
    fn native_model_encode_downgrade_body(
        self,
        version: u32,
    ) -> native_model::Result<Vec<u8>> {
        if version == Self::native_model_version() {
            let result = self.native_model_encode_body()?;
            Ok(result)
        } else if version < Self::native_model_version() {
            Err(native_model::Error::DowngradeNotSupported {
                from: version,
                to: Self::native_model_version(),
            })
        } else {
            Err(native_model::Error::DowngradeNotSupported {
                from: version,
                to: Self::native_model_version(),
            })
        }
    }
    fn native_model_decode_body(
        data: Vec<u8>,
        id: u32,
    ) -> std::result::Result<Self, native_model::DecodeBodyError> {
        if id != 1 {
            return Err(native_model::DecodeBodyError::MismatchedModelId);
        }
        use native_model::Decode;
        native_model::bincode_1_3::Bincode::decode(data)
            .map_err(|e| native_model::DecodeBodyError::DecodeError {
                msg: {
                    let res = ::alloc::fmt::format(format_args!("{0}", e));
                    res
                },
                source: e.into(),
            })
    }
    fn native_model_decode_upgrade_body(
        data: Vec<u8>,
        id: u32,
        version: u32,
    ) -> native_model::Result<Self> {
        if version == Self::native_model_version() {
            let result = Self::native_model_decode_body(data, id)?;
            Ok(result)
        } else if version < Self::native_model_version() {
            Err(native_model::Error::UpgradeNotSupported {
                from: version,
                to: Self::native_model_version(),
            })
        } else {
            Err(native_model::Error::UpgradeNotSupported {
                from: version,
                to: Self::native_model_version(),
            })
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ItemCustomPk {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "ItemCustomPk",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "id",
                &self.id,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for ItemCustomPk {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "id" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"id" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<ItemCustomPk>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = ItemCustomPk;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct ItemCustomPk",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        (String, u32),
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct ItemCustomPk with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private::Ok(ItemCustomPk { id: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<(String, u32)> = _serde::__private::None;
                    while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<
                                        (String, u32),
                                    >(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("id")?
                        }
                    };
                    _serde::__private::Ok(ItemCustomPk { id: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["id"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "ItemCustomPk",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<ItemCustomPk>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::cmp::Eq for ItemCustomPk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<(String, u32)>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ItemCustomPk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ItemCustomPk {
    #[inline]
    fn eq(&self, other: &ItemCustomPk) -> bool {
        self.id == other.id
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ItemCustomPk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "ItemCustomPk",
            "id",
            &&self.id,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ItemCustomPk {
    #[inline]
    fn clone(&self) -> ItemCustomPk {
        ItemCustomPk {
            id: ::core::clone::Clone::clone(&self.id),
        }
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "test_tuple_tokey"]
pub const test_tuple_tokey: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_tuple_tokey"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_tuple_field.rs",
        start_line: 14usize,
        start_col: 4usize,
        end_line: 14usize,
        end_col: 20usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_tuple_tokey()),
    ),
};
fn test_tuple_tokey() {
    let to_key = ("test".to_string(), 3u32);
    let to_key = to_key.to_key();
    match &to_key {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}:{2}] {3} = {4:#?}\n",
                        "tests/test_tuple_field.rs",
                        17u32,
                        5u32,
                        "&to_key",
                        &tmp,
                    ),
                );
            };
            tmp
        }
    };
    let key_names = <(String, u32)>::key_names();
    match &key_names {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}:{2}] {3} = {4:#?}\n",
                        "tests/test_tuple_field.rs",
                        20u32,
                        5u32,
                        "&key_names",
                        &tmp,
                    ),
                );
            };
            tmp
        }
    };
}
#[rustc_main]
#[coverage(off)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&test_tuple_tokey])
}
