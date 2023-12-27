//! Native DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, model migration, hot snapshot, and more.
//!
//! See [README.md](https://github.com/vincent-herlemont/native_db) for more information.

#![allow(dead_code)]
#![forbid(unsafe_code)]
#![warn(
    // TODO: frequently check
    // unreachable_pub,
    // TODO: Activate if you're feeling like fixing stuff 
    // clippy::pedantic,
    // clippy::correctness,
    // clippy::suspicious,
    // clippy::complexity,
    // clippy::perf,
    // TODO: Fix documentation
    // missing_docs,
    // TODO: Implement or derive Debug
    // missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications,
    clippy::nursery,
    bad_style,
    dead_code,
    improper_ctypes,
    missing_copy_implementations,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    trivial_numeric_casts,
    unused_results,
    trivial_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::cast_lossless,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::manual_string_new,
    clippy::match_same_arms,
    clippy::semicolon_if_nothing_returned,
    clippy::trivially_copy_pass_by_ref
)]
#![allow(clippy::type_complexity)]
// FIXME!
#![allow(clippy::match_same_arms)]

mod database;
mod database_builder;
pub mod db_type;
mod model;
mod serialization;
mod snapshot;
mod stats;
mod table_definition;

/// All database interactions here,[`r_transaction`](transaction/struct.RTransaction.html), [`rw_transaction`](transaction/struct.RwTransaction.html) and [`query`](transaction/query/index.html).
pub mod transaction;
/// Watch data in real-time.
pub mod watch;

// Re-export
pub use db_type::InnerKeyValue;
pub use db_type::Input;

// Export
pub use database::*;
pub use database_builder::*;
pub use model::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment! {
    include_str!("../README.md")
}

/// Macro which link [`native_model`](https://crates.io/crates/native_model) to the Native DB. See [`DatabaseBuilder.define`](struct.DatabaseBuilder.html#method.define) for more information.
pub use native_db_macro::*;
pub use serialization::*;
