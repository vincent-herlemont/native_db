use crate::crate_paths::CratePaths;
use crate::model_attributes::ModelAttributes;
use crate::model_native_db::ModelNativeDB;
use crate::struct_name::StructName;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn native_db(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = StructName::new(ast.ident.clone());

    let mut attrs = ModelAttributes {
        struct_name: struct_name.clone(),
        primary_key: None,
        secondary_keys: Default::default(),
        do_export_keys: None,
        native_db_crate: None,
        native_db_macro_crate: None,
        native_db_version: None,
    };
    let model_attributes_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(args with model_attributes_parser);

    if let Data::Struct(data_struct) = &ast.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                if let Err(err) = attrs.parse_field(field) {
                    return TokenStream::from(err.to_compile_error());
                }
            }
        }
    }

    let crate_paths = CratePaths::new(attrs.native_db_crate_path());
    let model_native_db =
        ModelNativeDB::new(struct_name.clone(), attrs.clone(), crate_paths.clone());

    let native_db_pk = model_native_db.native_db_primary_key();
    let native_db_gks = model_native_db.native_db_secondary_key();
    let native_db_model = model_native_db.native_db_model();

    let keys_enum_visibility = model_native_db.keys_enum_visibility();
    let keys_enum_name = model_native_db.keys_enum_name();
    let keys_enum = model_native_db.secondary_keys_enum();
    let keys_enum_database_key = model_native_db.keys_enum_database_key();

    let struct_name = struct_name.ident();
    let key_attributes_derive = crate_paths.key_attributes_derive();
    let to_input_trait = crate_paths.to_input_trait();
    let bincode_encode_to_vec_fn = crate_paths.bincode_encode_to_vec_fn();
    let bincode_decode_from_slice_fn = crate_paths.bincode_decode_from_slice_fn();
    let native_db_path = &crate_paths.native_db;

    let gen = quote! {
        #[derive(#key_attributes_derive)]
        #ast

        impl #to_input_trait for #struct_name {
            fn native_db_bincode_encode_to_vec(&self) -> #native_db_path::db_type::Result<Vec<u8>> {
                #bincode_encode_to_vec_fn(self)
            }

            fn native_db_bincode_decode_from_slice(slice: &[u8]) -> #native_db_path::db_type::Result<Self> {
                Ok(#bincode_decode_from_slice_fn(slice)?.0)
            }

            #native_db_model
            #native_db_pk
            #native_db_gks
        }

        #[allow(non_camel_case_types)]
        #keys_enum_visibility enum #keys_enum_name {
            #(#keys_enum),*
        }

        impl #native_db_path::db_type::ToKeyDefinition<#native_db_path::db_type::KeyOptions> for #keys_enum_name {
            #keys_enum_database_key
        }
    };

    gen.into()
}
