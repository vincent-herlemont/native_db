extern crate proc_macro;

mod struct_db;

use proc_macro::TokenStream;

use struct_db::struct_db as struct_db_impl;

#[proc_macro_attribute]
pub fn struct_db(args: TokenStream, input: TokenStream) -> TokenStream {
    struct_db_impl(args, input)
}
