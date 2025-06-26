use crate::crate_paths::CratePaths;
use crate::keys::ToTokenStream;
use crate::model_attributes::ModelAttributes;
use crate::struct_name::StructName;
use proc_macro::Span;
use quote::quote;
use syn::Ident;

pub(crate) struct ModelNativeDB {
    struct_name: StructName,
    attrs: ModelAttributes,
    crate_paths: CratePaths,
}

impl ModelNativeDB {
    pub fn new(struct_name: StructName, attrs: ModelAttributes, crate_paths: CratePaths) -> Self {
        Self {
            struct_name,
            attrs,
            crate_paths,
        }
    }

    pub(crate) fn native_db_secondary_key(&self) -> proc_macro2::TokenStream {
        let native_db_path = &self.crate_paths.native_db;
        let tokens = self
            .attrs
            .secondary_keys
            .iter()
            .map(|key| {
                let key_ident = key.ident();
                let new_secondary_key = key.new_to_token_stream_with_crate(native_db_path);
                let out = if key.is_field() {
                    if key.options.optional {
                        quote! {
                            let value: Option<#native_db_path::db_type::Key>  = self.#key_ident.as_ref().map(|v|(&v).to_key());
                            let value = #native_db_path::db_type::KeyEntry::Optional(value);
                        }
                    } else {
                        quote! {
                            let value: #native_db_path::db_type::Key  = (&self.#key_ident).to_key();
                            let value = #native_db_path::db_type::KeyEntry::Default(value);
                        }
                    }
                } else if key.is_function() {
                    if key.options.optional {
                        quote! {
                            let value: Option<#native_db_path::db_type::Key> = self.#key_ident().map(|v|(&v).to_key());
                            let value = #native_db_path::db_type::KeyEntry::Optional(value);
                        }
                    } else {
                        quote! {
                            let value: #native_db_path::db_type::Key = (&self.#key_ident()).to_key();
                            let value = #native_db_path::db_type::KeyEntry::Default(value);
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
            fn native_db_secondary_keys(&self) -> std::collections::HashMap<#native_db_path::db_type::KeyDefinition<#native_db_path::db_type::KeyOptions>, #native_db_path::db_type::KeyEntry> {
                let mut secondary_tables_name = std::collections::HashMap::new();
                #(#tokens)*
                secondary_tables_name
            }
        }
    }

    pub(crate) fn native_db_primary_key(&self) -> proc_macro2::TokenStream {
        let native_db_path = &self.crate_paths.native_db;
        let primary_key = self.attrs.primary_key();
        let ident = primary_key.ident();
        if primary_key.is_function() {
            quote! {
                fn native_db_primary_key(&self) -> #native_db_path::db_type::Key {
                    (&self.#ident()).to_key()
                }
            }
        } else {
            quote! {
                fn native_db_primary_key(&self) -> #native_db_path::db_type::Key {
                    (&self.#ident).to_key()
                }
            }
        }
    }

    pub(crate) fn native_db_model(&self) -> proc_macro2::TokenStream {
        let native_db_path = &self.crate_paths.native_db;
        let primary_key = self
            .attrs
            .primary_key()
            .new_to_token_stream_with_crate(native_db_path);
        let secondary_keys = self
            .attrs
            .secondary_keys
            .iter()
            .map(|key| {
                let new_key = key.new_to_token_stream_with_crate(native_db_path);
                quote! {
                    secondary_tables_name.insert(#new_key);
                }
            })
            .collect::<Vec<_>>();

        quote! {
            fn native_db_model() -> #native_db_path::Model {
                let mut secondary_tables_name = std::collections::HashSet::new();
                #(#secondary_keys)*
                #native_db_path::Model {
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

    pub(crate) fn keys_enum_visibility(&self) -> proc_macro2::TokenStream {
        let do_export = match &self.attrs.do_export_keys {
            Some(do_export_keys) => do_export_keys.value,
            None => false,
        };

        let visibility = if do_export { "" } else { "(crate)" };

        format!("pub{}", visibility).parse().unwrap()
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
        let native_db_path = &self.crate_paths.native_db;

        let insert_secondary_key_def = self.attrs.secondary_keys.iter().map(|key| {
            let name = key.ident();
            let new_key = key.new_to_token_stream_with_crate(native_db_path);
            quote! {
                #keys_enum_name_token::#name => #new_key,
            }
        });

        quote! {
            fn key_definition(&self) -> #native_db_path::db_type::KeyDefinition<#native_db_path::db_type::KeyOptions> {
                match self {
                    #(#insert_secondary_key_def)*
                    _ => panic!("Unknown key"),
                }
            }
        }
    }
}
