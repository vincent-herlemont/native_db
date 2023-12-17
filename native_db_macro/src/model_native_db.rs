use crate::model_attributes::ModelAttributes;
use crate::struct_name::StructName;
use crate::ToTokenStream;
use proc_macro::Span;
use quote::quote;
use syn::Ident;

pub(crate) struct ModelNativeDB {
    struct_name: StructName,
    attrs: ModelAttributes,
}

impl ModelNativeDB {
    pub fn new(struct_name: StructName, attrs: ModelAttributes) -> Self {
        Self { struct_name, attrs }
    }

    pub(crate) fn native_db_secondary_key(&self) -> proc_macro2::TokenStream {
        let tokens = self
            .attrs
            .secondary_keys
            .iter()
            .map(|key| {
                let key_ident = key.ident();
                let new_secondary_key = key.new_to_token_stream();
                let out = if key.is_field() {
                    if key.options.optional {
                        quote! {
                            let value: Option<native_db::db_type::DatabaseInnerKeyValue>  = self.#key_ident.as_ref().map(|v|v.database_inner_key_value());
                            let value = native_db::db_type::DatabaseKeyValue::Optional(value);
                        }
                    } else {
                        quote! {
                            let value: native_db::db_type::DatabaseInnerKeyValue  = self.#key_ident.database_inner_key_value();
                            let value = native_db::db_type::DatabaseKeyValue::Default(value);
                        }
                    }
                } else if key.is_function() {
                    if key.options.optional {
                        quote! {
                            let value: Option<native_db::db_type::DatabaseInnerKeyValue> = self.#key_ident().map(|v|v.database_inner_key_value());
                            let value = native_db::db_type::DatabaseKeyValue::Optional(value);
                        }
                    } else {
                        quote! {
                            let value: native_db::db_type::DatabaseInnerKeyValue = self.#key_ident().database_inner_key_value();
                            let value = native_db::db_type::DatabaseKeyValue::Default(value);
                        }
                    }
                } else {
                    panic!("Unknown key type")
                };

                quote! {
                    #out
                    secondary_tables_name.insert(#new_secondary_key, value);
                }
            })
            .collect::<Vec<_>>();

        quote! {
            fn native_db_secondary_keys(&self) -> std::collections::HashMap<native_db::db_type::DatabaseKeyDefinition<native_db::db_type::DatabaseSecondaryKeyOptions>, native_db::db_type::DatabaseKeyValue> {
                let mut secondary_tables_name = std::collections::HashMap::new();
                #(#tokens)*
                secondary_tables_name
            }
        }
    }

    pub(crate) fn native_db_primary_key(&self) -> proc_macro2::TokenStream {
        let primary_key = self.attrs.primary_key();
        let ident = primary_key.ident();
        if primary_key.is_function() {
            quote! {
                fn native_db_primary_key(&self) -> native_db::db_type::DatabaseInnerKeyValue {
                    self.#ident().database_inner_key_value()
                }
            }
        } else {
            quote! {
                fn native_db_primary_key(&self) -> native_db::db_type::DatabaseInnerKeyValue {
                    self.#ident.database_inner_key_value()
                }
            }
        }
    }

    pub(crate) fn native_db_model(&self) -> proc_macro2::TokenStream {
        let primary_key = self.attrs.primary_key().new_to_token_stream();
        let secondary_keys = self
            .attrs
            .secondary_keys
            .iter()
            .map(|key| {
                let new_key = key.new_to_token_stream();
                quote! {
                    secondary_tables_name.insert(#new_key);
                }
            })
            .collect::<Vec<_>>();

        quote! {
            fn native_db_model() -> native_db::Model {
                let mut secondary_tables_name = std::collections::HashSet::new();
                #(#secondary_keys)*
                native_db::Model {
                    primary_key: #primary_key,
                    secondary_keys: secondary_tables_name,
                }
            }
        }
    }

    pub(crate) fn keys_enum_name(&self) -> Ident {
        let struct_name = self.struct_name.ident();
        Ident::new(&format!("{}Key", struct_name), Span::call_site().into())
    }

    pub(crate) fn secondary_keys_enum(&self) -> Vec<proc_macro2::TokenStream> {
        self.attrs
            .secondary_keys
            .iter()
            .map(|key| {
                let name = key.ident();
                quote! {
                    #[allow(non_camel_case_types,dead_code)]
                    #name
                }
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn keys_enum_database_key(&self) -> proc_macro2::TokenStream {
        let keys_enum_name_token = self.keys_enum_name();

        let insert_secondary_key_def = self.attrs.secondary_keys.iter().map(|key| {
            let name = key.ident();
            let new_key = key.new_to_token_stream();
            quote! {
                #keys_enum_name_token::#name => #new_key,
            }
        });

        quote! {
            fn database_key(&self) -> native_db::db_type::DatabaseKeyDefinition<native_db::db_type::DatabaseSecondaryKeyOptions> {
                match self {
                    #(#insert_secondary_key_def)*
                    _ => panic!("Unknown key"),
                }
            }
        }
    }
}
