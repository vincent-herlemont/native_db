use crate::model_attributes::ModelAttributes;
use proc_macro::Span;
use quote::quote;
use syn::Ident;

pub(crate) struct ModelStructDB {
    struct_name: Ident,
    attrs: ModelAttributes,
}

impl ModelStructDB {
    pub fn new(struct_name: Ident, attrs: ModelAttributes) -> Self {
        Self { struct_name, attrs }
    }

    pub fn table_name(&self) -> String {
        self.struct_name().to_lowercase()
    }

    pub fn struct_name(&self) -> String {
        self.struct_name.clone().to_string()
    }

    pub(crate) fn secondary_key_function_names(&self) -> Vec<(Ident, Ident)> {
        let table_name = self.table_name();
        self.attrs
            .gk_function_names()
            .iter()
            .map(|secondary_key_function_name| {
                let secondary_key_function_name =
                    secondary_key_function_name.to_string().to_lowercase();
                let secondary_table_name =
                    format!("{}_{}", table_name, secondary_key_function_name);
                (
                    Ident::new(&secondary_key_function_name, Span::call_site().into()),
                    Ident::new(&secondary_table_name, Span::call_site().into()),
                )
            })
            .collect::<Vec<(Ident, Ident)>>()
    }

    pub(crate) fn struct_db_gks(&self) -> proc_macro2::TokenStream {
        let tokens = self
            .secondary_key_function_names()
            .iter()
            .map(|secondary_key_function| {
                let (gk_name, secondary_table) = secondary_key_function.clone();
                let secondary_table = secondary_table.to_string();
                quote! {
                    secondary_tables_name.insert(#secondary_table, self.#gk_name());
                }
            })
            .collect::<Vec<_>>();

        quote! {
            fn struct_db_gks(&self) -> std::collections::HashMap<&'static str, Vec<u8>> {
                let mut secondary_tables_name = std::collections::HashMap::new();
                #(#tokens)*
                secondary_tables_name
            }
        }
    }

    pub(crate) fn struct_db_pk(&self) -> proc_macro2::TokenStream {
        let primary_key_function_name = self.attrs.pk();
        quote! {
            fn struct_db_pk(&self) -> Vec<u8> {
                self.#primary_key_function_name()
            }
        }
    }

    pub(crate) fn struct_db_schema(&self) -> proc_macro2::TokenStream {
        let table_name = self.table_name();
        let primary_key_name = self.attrs.pk_name();
        let insert_tokens = self
            .secondary_key_function_names()
            .iter()
            .map(|gk| {
                let (_, secondary_table) = gk.clone();
                let secondary_table = secondary_table.to_string();
                quote! {
                    secondary_tables_name.insert(#secondary_table);
                }
            })
            .collect::<Vec<_>>();

        quote! {
            fn struct_db_schema() -> struct_db::Schema {
                let mut secondary_tables_name = std::collections::HashSet::new();
                #(#insert_tokens)*
                struct_db::Schema {
                    table_name: #table_name,
                    primary_key: #primary_key_name,
                    secondary_tables_name: secondary_tables_name,
                }
            }
        }
    }

    pub(crate) fn keys_enum_name(&self) -> Ident {
        let struct_name = self.struct_name();
        Ident::new(&format!("{}Key", struct_name), Span::call_site().into())
    }

    pub(crate) fn keys_enum(&self) -> Vec<proc_macro2::TokenStream> {
        self.secondary_key_function_names()
            .iter()
            .map(|gk| {
                let (gk_name, _) = gk.clone();
                quote! {
                    #gk_name
                }
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn keys_enum_fn_secondary_table_name(&self) -> Vec<proc_macro2::TokenStream> {
        let keys_enum_name_token = self.keys_enum_name();
        self.secondary_key_function_names()
            .iter()
            .map(|gk| {
                let (fn_key_name, secondary_table) = gk.clone();
                let secondary_table = secondary_table.to_string();
                quote! {
                    #keys_enum_name_token::#fn_key_name => #secondary_table,
                }
            })
            .collect::<Vec<_>>()
    }
}
