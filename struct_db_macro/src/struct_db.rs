use proc_macro::{Span, TokenStream};
use quote::quote;
use std::cell::RefCell;
use std::collections::HashSet;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn struct_db(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let fn_primary_key_def: RefCell<Option<Ident>> = RefCell::new(None);
    let fn_secondary_keys_def: RefCell<HashSet<Ident>> = RefCell::new(HashSet::new());

    let simple_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("fn_primary_key") {
            meta.parse_nested_meta(|meta| {
                if let Some(iden) = meta.path.get_ident() {
                    *fn_primary_key_def.borrow_mut() = Some(iden.clone());
                }
                Ok(())
            })?;
        }
        if meta.path.is_ident("fn_secondary_key") {
            meta.parse_nested_meta(|meta| {
                if let Some(iden) = meta.path.get_ident() {
                    let mut fn_secondary_keys_values = fn_secondary_keys_def.borrow_mut();
                    fn_secondary_keys_values.insert(iden.clone());
                }
                Ok(())
            })?;
        }
        return Ok(());
    });
    if !args.to_string().is_empty() {
        parse_macro_input!(args with simple_parser);
    }

    // Value of the table:
    let table_name = struct_name.to_string().to_lowercase();
    let primary_key_function_value = fn_primary_key_def
        .borrow()
        .clone()
        .expect("fn_primary_key is required");
    let primary_key_name = primary_key_function_value.to_string().to_lowercase();
    let fn_secondary_keys_name = fn_secondary_keys_def.borrow().clone();

    let fn_secondary_keys_name = fn_secondary_keys_name
        .iter()
        .map(|fn_secondary_key| {
            let fn_secondary_key_name = fn_secondary_key.to_string().to_lowercase();
            let secondary_table = format!("{}_{}", table_name, fn_secondary_key_name);
            (
                Ident::new(&fn_secondary_key_name, Span::call_site().into()),
                Ident::new(&secondary_table, Span::call_site().into()),
            )
        })
        .collect::<Vec<(Ident, Ident)>>();

    let struct_db_keys_tokens = fn_secondary_keys_name.iter().map(|fn_secondary_key| {
        let (fn_secondary_key_name, secondary_table) = fn_secondary_key.clone();
        let secondary_table = secondary_table.to_string();
        quote! {
            secondary_tables_name.insert(#secondary_table, self.#fn_secondary_key_name());
        }
    });

    let struct_db_schema_tokens = fn_secondary_keys_name.iter().map(|fn_secondary_key| {
        let (_, secondary_table) = fn_secondary_key.clone();
        let secondary_table = secondary_table.to_string();
        quote! {
            secondary_tables_name.insert(#secondary_table);
        }
    });

    let keys_enum_name_token = Ident::new(&format!("{}Key", struct_name), Span::call_site().into());
    let keys_enum_tokens = fn_secondary_keys_name.iter().map(|fn_secondary_key| {
        let (fn_secondary_key_name, _) = fn_secondary_key.clone();
        quote! {
            #fn_secondary_key_name
        }
    });
    let keys_enum_fn_secondary_table_name_tokens =
        fn_secondary_keys_name.iter().map(|fn_secondary_key| {
            let (fn_key_name, secondary_table) = fn_secondary_key.clone();
            let secondary_table = secondary_table.to_string();
            quote! {
                #keys_enum_name_token::#fn_key_name => #secondary_table,
            }
        });

    #[cfg(feature = "use_native_model")]
    let is_native_model_exists = quote! {
        true
    };
    #[cfg(not(feature = "use_native_model"))]
    let is_native_model_exists = quote! {
        false
    };

    let gen = quote! {
        #ast

        impl #struct_name {
            fn is_native_model() -> bool {
                #is_native_model_exists
            }
        }

        impl struct_db::SDBItem for #struct_name {
            fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8> {
                struct_db::bincode_encode_to_vec(self).expect("Failed to serialize the struct #struct_name")
            }

            fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self {
                struct_db::bincode_decode_from_slice(slice).expect("Failed to deserialize the struct #struct_name").0
            }

            fn struct_db_schema() -> struct_db::Schema {
                let mut secondary_tables_name = std::collections::HashSet::new();
                #(#struct_db_schema_tokens)*
                struct_db::Schema {
                    table_name: #table_name,
                    primary_key: #primary_key_name,
                    secondary_tables_name: secondary_tables_name,
                }
            }

            fn struct_db_primary_key(&self) -> Vec<u8> {
                self.#primary_key_function_value()
            }

            fn struct_db_keys(&self) -> std::collections::HashMap<&'static str, Vec<u8>> {
                let mut secondary_tables_name = std::collections::HashMap::new();
                #(#struct_db_keys_tokens)*
                secondary_tables_name
            }
        }

        /// Index selection Enum for [#struct_name]
        pub(crate) enum #keys_enum_name_token {
            #(#keys_enum_tokens),*
        }

        impl struct_db::KeyDefinition for #keys_enum_name_token {
            fn secondary_table_name(&self) -> &'static str {
                match self {
                    #(#keys_enum_fn_secondary_table_name_tokens)*
                    _ => panic!("Unknown key"),
                }
            }
        }
    };

    gen.into()
}
