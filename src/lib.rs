//! Native DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, model migration, hot snapshot, and more.
//!
//! See [README.md](https://github.com/vincent-herlemont/native_db) for more information.
mod database;
mod database_builder;
mod database_instance;

/// A collection of type used by native_db internally (macro included).
pub mod db_type;
mod model;
mod serialization;
mod snapshot;
mod stats;
mod table_definition;
pub mod upgrade;

/// All database interactions here,[`r_transaction`](transaction/struct.RTransaction.html), [`rw_transaction`](transaction/struct.RwTransaction.html) and [`query`](transaction/query/index.html).
pub mod transaction;
/// Watch data in real-time.
pub mod watch;

// Re-export
pub use db_type::Input;
pub use db_type::Key;
/// Allow to use a type as a key in the database.
pub use db_type::ToKey;

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
