//! Native DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, model migration, hot snapshot, and more.
//!
//! See [README.md](https://github.com/vincent-herlemont/native_db) for more information.
//!
//! # Quick Start
//!
//! 1. [Create a model](#create-a-model)
//! 2. [Create the database with the model](#create-the-database-with-the-model)
//! 3. [Use a model in the database](#use-a-model-in-the-database)
//! 4. [Update the model](#update-the-model)
//! 5. [Use the updated model in the database (migration)](#use-the-updated-model-in-the-database-migration)
//!
//! ## Create a model
//!
//! > 👉 Unlike the usual database where there is a differentiation between *schema* and *model*, here, as we can directly use Rust types that are serialized in the database, we do not have the concept of *schema*, only that of the *model*.
//!
//! Note that the organization of the models it's a best practice but not mandatory, you can organize your models as you want if you prefer.
//!
//! In this example:
//! - We create a module `data` which contains **all versions of all models**.
//! - We create a module `v1` which contains the **first version of your data**, we will put other versions later.
//! - We create a type alias `Person` to the latest version `v1::Person`, which allows us to use the **latest version** of the model in the application.
//!
//! ```rust
//! pub mod data {
//!     use native_db::{native_db, ToKey};
//!     use native_model::{native_model, Model};
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
//! ## Create the database with the model
//!
//! After creating the model in the previous step, we can now create the database with the model.
//!
//! Note good practices: define the models by **specifying each version**, in our case `data::v1::Person`.
//!
//! ```rust
//! # pub mod data {
//! #     use native_db::{native_db, ToKey};
//! #     use native_model::{native_model, Model};
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
//!
//! fn main() -> Result<(), db_type::Error> {
//!     // Create the models collection
//!     let mut models = Models::new();
//!     // It's a good practice to define the models by specifying the version
//!     models.define::<data::v1::Person>()?;
//!     // Create the database
//!     let db = Builder::new().create_in_memory(&models)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Use a model in the database
//!
//! Note a good practice: use the **latest version** of the model in your application.
//! In our case, we use `data::Person`.
//!
//! ```rust
//! # pub mod data {
//! #     use native_db::{native_db, ToKey};
//! #     use native_model::{native_model, Model};
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
//!
//! fn main() -> Result<(), db_type::Error> {
//!     # // Create the models collection
//!     # let mut models = Models::new();
//!     # // It's a good practice to define the models by specifying the version
//!     # models.define::<data::v1::Person>()?;
//!     # // Create the database
//!     # let db = Builder::new().create_in_memory(&models)?;
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
//!     let person: data::Person = r.get().primary(&"Alice".to_string())?.unwrap();
//!     assert_eq!(person.name, "Alice");
//!     Ok(())
//! }
//! ```
//!
//! ## Update the model
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
//! #    use native_db::{native_db, ToKey};
//! #    use native_model::{native_model, Model};
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
//! ## Use the updated model in the database (migration)
//!
//! After updating the model, we need to define the new version `v2` of the model `Person` and migrate the data.
//!
//! ```rust
//! # pub mod data {
//! #    // ... same imports
//! #    use native_db::{native_db, ToKey};
//! #    use native_model::{native_model, Model};
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
//!
//! fn main() -> Result<(), db_type::Error> {
//!     // Create the models collection
//!     let mut models = Models::new();
//!     // Define the models by specifying the version
//!     models.define::<data::v1::Person>()?;
//!     models.define::<data::v2::Person>()?;
//!
//!     // Create the database
//!     let db = Builder::new().create_in_memory(&models)?;
//!
//!     // Migrate the data in a transaction
//!     let rw = db.rw_transaction()?;
//!     rw.migrate::<data::Person>()?;
//!     rw.commit()?;
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
mod model;
mod serialization;
mod snapshot;
mod stats;
mod table_definition;
pub mod upgrade;

mod models;

/// All database interactions here,[`r_transaction`](transaction/struct.RTransaction.html), [`rw_transaction`](transaction/struct.RwTransaction.html) and [`query`](transaction/query/index.html).
pub mod transaction;
/// Watch data in real-time.
pub mod watch;

// Re-export
pub use db_type::Key;
pub use db_type::ToInput;
/// Allow to use a type as a key in the database.
pub use db_type::ToKey;

// Export
pub use database::*;
pub use database_builder::*;
pub use model::*;
pub use models::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment! {
    include_str!("../README.md")
}

/// Macro which link [`native_model`](https://crates.io/crates/native_model) to the Native DB. See [`Builder.define`](struct.Builder.html#method.define) for more information.
pub use native_db_macro::*;
pub use serialization::*;
