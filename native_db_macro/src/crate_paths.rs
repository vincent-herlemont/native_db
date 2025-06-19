use quote::quote;
use syn::Path;

#[derive(Clone)]
pub(crate) struct CratePaths {
    pub native_db: Path,
}

impl CratePaths {
    pub fn new(native_db_crate: Path) -> Self {
        Self {
            native_db: native_db_crate,
        }
    }

    pub fn to_input_trait(&self) -> proc_macro2::TokenStream {
        let crate_path = &self.native_db;
        quote! { #crate_path::db_type::ToInput }
    }

    pub fn key_attributes_derive(&self) -> proc_macro2::TokenStream {
        let crate_path = &self.native_db;
        quote! { #crate_path::KeyAttributes }
    }

    pub fn bincode_encode_to_vec_fn(&self) -> proc_macro2::TokenStream {
        let crate_path = &self.native_db;
        quote! { #crate_path::serialization::bincode_encode_to_vec }
    }

    pub fn bincode_decode_from_slice_fn(&self) -> proc_macro2::TokenStream {
        let crate_path = &self.native_db;
        quote! { #crate_path::serialization::bincode_decode_from_slice }
    }
}
