use crate::model_attributes::ModelAttributes;
use crate::model_struct_db::ModelStructDB;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn struct_db(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let mut attrs = ModelAttributes::default();
    let model_attributes_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(args with model_attributes_parser);
    let model_struct_db = ModelStructDB::new(struct_name.clone(), attrs.clone());

    let struct_db_pk = model_struct_db.struct_db_pk();
    let struct_db_gks = model_struct_db.struct_db_gks();
    let struct_db_schema = model_struct_db.struct_db_schema();

    let keys_enum_name = model_struct_db.keys_enum_name();
    let keys_enum = model_struct_db.keys_enum();
    let keys_enum_fn_secondary_table_name = model_struct_db.keys_enum_fn_secondary_table_name();

    let gen = quote! {
        #ast

        impl struct_db::SDBItem for #struct_name {
            fn struct_db_bincode_encode_to_vec(&self) -> Vec<u8> {
                struct_db::bincode_encode_to_vec(self).expect("Failed to serialize the struct #struct_name")
            }

            fn struct_db_bincode_decode_from_slice(slice: &[u8]) -> Self {
                struct_db::bincode_decode_from_slice(slice).expect("Failed to deserialize the struct #struct_name").0
            }

            #struct_db_schema
            #struct_db_pk
            #struct_db_gks
        }

        /// Index selection Enum for [#struct_name]
        pub(crate) enum #keys_enum_name {
            #(#keys_enum),*
        }

        impl struct_db::KeyDefinition for #keys_enum_name {
            fn secondary_table_name(&self) -> &'static str {
                match self {
                    #(#keys_enum_fn_secondary_table_name)*
                    _ => panic!("Unknown key"),
                }
            }
        }
    };

    gen.into()
}
