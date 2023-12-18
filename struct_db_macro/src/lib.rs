/// **--> /!\ Important Update: This crate `struct_db` has been renamed to [`native_db`](https://crates.io/crates/native_db) to better reflect its functionality and purpose. Please update your dependencies to use [`native_db`](https://crates.io/crates/native_db) for the latest features and updates. <--**
extern crate proc_macro;

mod struct_db;

use proc_macro::TokenStream;

use struct_db::struct_db as struct_db_impl;

#[proc_macro_attribute]
pub fn struct_db(args: TokenStream, input: TokenStream) -> TokenStream {
    struct_db_impl(args, input)
}
