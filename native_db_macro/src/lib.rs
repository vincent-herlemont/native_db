extern crate proc_macro;

mod keys;
mod model_attributes;
mod model_native_db;
mod native_db;
mod struct_name;

use proc_macro::TokenStream;

use native_db::native_db as native_db_impl;

#[proc_macro_attribute]
pub fn native_db(args: TokenStream, input: TokenStream) -> TokenStream {
    native_db_impl(args, input)
}

#[proc_macro_derive(KeyAttributes, attributes(primary_key, secondary_key))]
pub fn key_attributes(_input: TokenStream) -> TokenStream {
    let gen = quote::quote! {};
    gen.into()
}

trait ToTokenStream {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream;
}
