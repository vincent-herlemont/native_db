//! Native DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, model migration, hot snapshot, and more.
//!
//! # Summary
//! - [Api](#api)
//! - [Quick Start](#quick-start)
//!    - [Create a model](#create-a-model)
//!    - [Create a database](#create-a-database)
//!    - [Insert a model in the database](#insert-a-model-in-the-database)
//!    - [Update a model](#update-a-model)
//!    - [Migration](#migration)
//! - Advanced
//!    - [`Define a type as a key`](crate::db_type::ToKey)
//!       - [Example with `uuid`](crate::db_type::ToKey#example-with-uuid)
//!       - [Example with `chrono`](crate::db_type::ToKey#example-with-chrono)
//!
//! # Api
//!
//! - [`Models`] - Collection of models. *Equivalent to a schema in a traditional database*.
//!    - [`new`](crate::Models::new) - Create a new collection of models.
//!    - [`define`](crate::Models::define) - Define a model.
//! - [`Builder`] - Builder to create a database.
//!    - [`create_in_memory`](crate::Builder::create_in_memory) - Create a database in memory.
//!    - [`create`](crate::Builder::create) - Create a database in a file.
//!    - [`open`](crate::Builder::open) - Open a database.
//! - [`Database`] - Database instance.
//!    - [`compact`](crate::Database::compact) - Compact the database.
//!    - [`check_integrity`](crate::Database::check_integrity) - Check the integrity of the database.
//!    - [`rw_transaction`](crate::Database::rw_transaction) - Create a read-write transaction.
//!       - [`insert`](crate::transaction::RwTransaction::insert) - Insert a item, fail if the item already exists.
//!       - [`upsert`](crate::transaction::RwTransaction::upsert) - Upsert a item, update if the item already exists.
//!       - [`update`](crate::transaction::RwTransaction::update) - Update a item, replace an existing item.
//!       - [`remove`](crate::transaction::RwTransaction::remove) - Remove a item, remove an existing item.
//!       - [`migrate`](crate::transaction::RwTransaction::migrate) - Migrate a model, affect all items.
//!       - [`commit`](crate::transaction::RwTransaction::commit) - Commit the transaction.
//!       - [`abort`](crate::transaction::RwTransaction::abort) - Abort the transaction.
//!   - [`r_transaction`](crate::Database::r_transaction) - Create a read-only transaction.
//!       - [`get`](crate::transaction::RTransaction::get) - Get a item.
//!          - [`primary`](crate::transaction::query::RGet::primary) - Get a item by primary key.
//!          - [`secondary`](crate::transaction::query::RGet::secondary) - Get a item by secondary key.
//!       - [`scan`](crate::transaction::RTransaction::scan) - Scan items.
//!          - [`primary`](crate::transaction::query::RScan::primary) - Scan items by primary key.
//!             - [`all`](crate::transaction::query::PrimaryScan::all) - Scan all items.
//!             - [`start_with`](crate::transaction::query::PrimaryScan::start_with) - Scan items with a primary key starting with a key.
//!             - [`range`](crate::transaction::query::PrimaryScan::range) - Scan items with a primary key in a given range.
//!          - [`secondary`](crate::transaction::query::RScan::secondary) - Scan items by secondary key.
//!             - [`all`](crate::transaction::query::SecondaryScan::all) - Scan items with a given secondary key.
//!             - [`start_with`](crate::transaction::query::SecondaryScan::start_with) - Scan items with a secondary key starting with a key.
//!             - [`range`](crate::transaction::query::SecondaryScan::range) - Scan items with a secondary key in a given range.
//!       - [`len`](crate::transaction::RTransaction::len) - Get the number of items.
//!          - [`primary`](crate::transaction::query::RLen::primary) - Get the number of items by primary key.
//!          - [`secondary`](crate::transaction::query::RLen::secondary) - Get the number of items by secondary key.    
//!   - [`watch`](crate::Database::watch) - Watch items in real-time.  Works via [std channel](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html) based or [tokio channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html) based depending on the feature `tokio`.
//!       - [`get`](crate::watch::query::Watch::get) - Watch a item.
//!          - [`primary`](crate::watch::query::WatchGet::primary) - Watch a item by primary key.
//!          - [`secondary`](crate::watch::query::WatchGet::secondary) - Watch a item by secondary key.
//!       - [`scan`](crate::watch::query::Watch::scan) - Watch items.
//!          - [`primary`](crate::watch::query::WatchScan::primary) - Watch items by primary key.
//!             - [`all`](crate::watch::query::WatchScanPrimary::all) - Watch all items.
//!             - [`start_with`](crate::watch::query::WatchScanPrimary::start_with) - Watch items with a primary key starting with a key.
//!             - [`range`](crate::watch::query::WatchScanPrimary::range) - Watch items with a primary key in a given range.
//!          - [`secondary`](crate::watch::query::WatchScan::secondary) - Watch items by secondary key.
//!             - [`all`](crate::watch::query::WatchScanSecondary::all) - Watch items with a given secondary key.
//!             - [`start_with`](crate::watch::query::WatchScanSecondary::start_with) - Watch items with a secondary key starting with a key.
//!             - [`range`](crate::watch::query::WatchScanSecondary::range) - Watch items with a secondary key in a given range.
//!
//!
//! # Quick Start
//!
//! We will create a simple example to show how to use the library.
//!
//! ## Create a model
//!
//! > 👉 Unlike the usual database where there is a difference between *schema* and *model*, here, as we can directly use Rust types that are serialized in the database, we do not have the concept of *schema*, only that of the *model*.
//!
//! In this section, we will create a simple model. I have chosen a particular organization using Rust modules, which I find to be a best practice. However, it is not mandatory; you can do it as you prefer. (see [`define`](crate::Models::define) for more information)
//!
//! In this example:
//! - We create a module `data` which contains **all versions of all models**.
//! - We create a module `v1` which contains the **first version of your data**, we will put other versions later.
//! - We create a type alias `Person` to the latest version `v1::Person`, which allows us to use the **latest version** of the model in the application.
//!
//! ```rust
//! pub mod data {
//!     use native_db::{
//!         native_db,
//!         native_model::{self, native_model, Model},
//!         ToKey,
//!     };
//!     use serde::{Deserialize, Serialize};
//!
//!     pub type Person = v1::Person;
//!
//!     pub mod v1 {
//!         use super::*;
//!         
//!         #[derive(Serialize, Deserialize, Debug)]
//!         #[native_model(id = 1, version = 1)]
//!         #[native_db]
//!         pub struct Person {
//!            #[primary_key]
//!            pub name: String,
//!         }
//!     }
//! }
//! ```
//!
//! ## Create a database
//!
//! After creating the model in the previous step, we can now create the database with the model.
//!
//! Note good practices: [`define`](crate::Models::define) the [`models`](crate::Models) by **specifying each version**, in our case `data::v1::Person`.
//!
//! ```rust
//! # pub mod data {
//! #     use native_db::{
//! #         native_db,
//! #         native_model::{self, native_model, Model},
//! #         ToKey,
//! #     };
//! #     use serde::{Deserialize, Serialize};
//! #
//! #     pub type Person = v1::Person;
//! #
//! #     pub mod v1 {
//! #         use super::*;
//! #         
//! #         #[derive(Serialize, Deserialize, Debug)]
//! #         #[native_model(id = 1, version = 1)]
//! #         #[native_db]
//! #         pub struct Person {
//! #            #[primary_key]
//! #            pub name: String,
//! #         }
//! #     }
//! # }
//! use native_db::*;
//! use once_cell::sync::Lazy;
//!
//! // Define the models
//! // The lifetime of the models needs to be longer or equal to the lifetime of the database.
//! // In many cases, it is simpler to use a static variable but it is not mandatory.
//! static MODELS: Lazy<Models> = Lazy::new(|| {
//!    let mut models = Models::new();
//!    // It's a good practice to define the models by specifying the version
//!    models.define::<data::v1::Person>().unwrap();
//!    models
//! });
//!
//! fn main() -> Result<(), db_type::Error> {
//!     // Create the database
//!     let db = Builder::new().create_in_memory(&MODELS)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Insert a model in the database
//!
//! Note a good practice: use the **latest version** of the model in your application.
//! In our case, we use `data::Person`.
//!
//! ```rust
//! # pub mod data {
//! #     use native_db::{
//! #         native_db,
//! #         native_model::{self, native_model, Model},
//! #         ToKey,
//! #     };
//! #     use serde::{Deserialize, Serialize};
//! #
//! #     pub type Person = v1::Person;
//! #
//! #     pub mod v1 {
//! #         use super::*;
//! #         
//! #         #[derive(Serialize, Deserialize, Debug)]
//! #         #[native_model(id = 1, version = 1)]
//! #         #[native_db]
//! #         pub struct Person {
//! #            #[primary_key]
//! #            pub name: String,
//! #         }
//! #     }
//! # }
//! use native_db::*;
//! use once_cell::sync::Lazy;
//! #
//! # static MODELS: Lazy<Models> = Lazy::new(|| {
//! #    let mut models = Models::new();
//! #    models.define::<data::v1::Person>().unwrap();
//! #    models
//! # });
//!
//! fn main() -> Result<(), db_type::Error> {
//!     # let db = Builder::new().create_in_memory(&MODELS)?;
//!     // ... database creation see previous example
//!
//!     // Insert a person
//!     let rw = db.rw_transaction()?;
//!     // It's a good practice to use the latest version in your application
//!     rw.insert(data::Person { name: "Alice".to_string() })?;
//!     rw.commit()?;
//!
//!     // Get the person
//!     let r = db.r_transaction()?;
//!     let person: data::Person = r.get().primary("Alice".to_string())?.unwrap();
//!     assert_eq!(person.name, "Alice");
//!     Ok(())
//! }
//! ```
//!
//! ## Update a model
//!
//! We need to add the field `age` to the `Person` model, but data is already stored in the database so we need to migrate it.
//!
//! To do this we have to:
//! - Create a version `v2` of the model `Person` with the new field `age`.
//! - Implement the `From` (or `TryFrom`) trait for the previous version `v1` to the new version `v2`, so we can migrate the data.
//!   See [native_model#Data model](https://github.com/vincent-herlemont/native_model?tab=readme-ov-file#data-model) for more information.
//!
//! ```rust
//! pub mod data {
//!     // ... same imports
//! #     use native_db::{
//! #         native_db,
//! #         native_model::{self, native_model, Model},
//! #         ToKey,
//! #     };
//! #    use serde::{Deserialize, Serialize};
//!     
//!     // Update the type alias to the latest version
//!     pub type Person = v2::Person;
//!
//!     pub mod v1 {
//!          // ... the previous version of Person
//! #        use super::*;
//! #       #[derive(Serialize, Deserialize, Debug)]
//! #       #[native_model(id = 1, version = 1)]
//! #       #[native_db]
//! #       pub struct Person {
//! #          #[primary_key]
//! #          pub name: String,
//! #       }
//!         
//!         impl From<v2::Person> for Person {
//!            fn from(p: v2::Person) -> Self {
//!               Self {
//!                  name: p.name,
//!               }
//!            }
//!         }
//!     }
//!
//!     pub mod v2 {
//!         use super::*;
//!
//!         #[derive(Serialize, Deserialize, Debug)]
//!         #[native_model(id = 1, version = 2, from = v1::Person)]
//!         #[native_db]
//!         pub struct Person {
//!            #[primary_key]
//!            pub name: String,
//!            pub age: u8,
//!         }
//!         
//!         impl From<v1::Person> for Person {
//!            fn from(p: v1::Person) -> Self {
//!               Self {
//!                  name: p.name,
//!                  age: 0,
//!               }
//!            }
//!         }
//!     }
//! }
//! ```
//!
//! ## Migration
//!
//! After updating the model, we need to define the new version `v2` of the model `Person` and migrate the data.
//!
//! ```rust
//! # pub mod data {
//! #    // ... same imports
//! #     use native_db::{
//! #         native_db,
//! #         native_model::{self, native_model, Model},
//! #         ToKey,
//! #     };
//! #    use serde::{Deserialize, Serialize};
//! #    
//! #    // Update the type alias to the latest version
//! #    pub type Person = v2::Person;
//! #
//! #    pub mod v1 {
//! #         // ... the previous version of Person
//! #        use super::*;
//! #       #[derive(Serialize, Deserialize, Debug)]
//! #       #[native_model(id = 1, version = 1)]
//! #       #[native_db]
//! #       pub struct Person {
//! #          #[primary_key]
//! #          pub name: String,
//! #       }
//! #         
//! #         impl From<v2::Person> for Person {
//! #            fn from(p: v2::Person) -> Self {
//! #               Self {
//! #                  name: p.name,
//! #               }
//! #            }
//! #         }
//! #     }
//! #
//! #     pub mod v2 {
//! #         use super::*;
//! #
//! #         #[derive(Serialize, Deserialize, Debug)]
//! #         #[native_model(id = 1, version = 2, from = v1::Person)]
//! #         #[native_db]
//! #         pub struct Person {
//! #            #[primary_key]
//! #            pub name: String,
//! #            pub age: u8,
//! #         }
//! #         
//! #         impl From<v1::Person> for Person {
//! #            fn from(p: v1::Person) -> Self {
//! #               Self {
//! #                  name: p.name,
//! #                  age: 0,
//! #               }
//! #            }
//! #         }
//! #     }
//! # }
//! use native_db::*;
//! use once_cell::sync::Lazy;
//!
//! static MODELS: Lazy<Models> = Lazy::new(|| {
//!    let mut models = Models::new();
//!    // Define the models by specifying the version
//!    models.define::<data::v1::Person>().unwrap();
//!    models.define::<data::v2::Person>().unwrap();
//!    models
//! });
//!
//! fn main() -> Result<(), db_type::Error> {
//!     // Create the database
//!     let db = Builder::new().create_in_memory(&MODELS)?;
//!
//!     // Migrate the data in a transaction
//!     let rw = db.rw_transaction()?;
//!     rw.migrate::<data::Person>()?;
//!     rw.commit()?;
//!
//!     // Now we can insert a person with the new field age ...
//!
//!     Ok(())
//! }
//! ```
//!
//! More details [`migrate`](crate::transaction::RwTransaction::migrate) method.
//!
mod database;
mod database_builder;
mod database_instance;

/// A collection of type used by native_db internally (macro included).
pub mod db_type;
mod metadata;
mod model;
mod serialization;
mod snapshot;
mod stats;
mod table_definition;
pub mod upgrade;

mod models;

/// Database interactions here.
pub mod transaction;
/// Watch data in real-time.
pub mod watch;

// Re-export
pub use db_type::Key;
pub use db_type::ToInput;
/// Allow to use a type as a key in the database.
pub use db_type::ToKey;
pub use native_model;

// Export
pub use database::*;
pub use database_builder::*;
pub use metadata::*;
pub use model::*;
pub use models::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment! {
    include_str!("../README.md")
}

pub use native_db_macro::*;
pub use serialization::*;
