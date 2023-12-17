//! Native DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, model migration.

mod builder;
mod database;
pub mod db_type;
mod model;
mod serialization;
mod snapshot;
mod stats;
mod table_definition;
pub mod transaction;
pub mod watch;

// Re-export
pub use db_type::InnerKeyValue;
pub use db_type::Input;

// Export
pub use builder::*;
pub use database::*;
pub use model::*;
pub use native_db_macro::*;
pub use native_db_macro::*;
pub use serialization::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment! {
    include_str!("../README.md")
}
