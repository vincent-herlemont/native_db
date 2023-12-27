use crate::db_type::Result;
use crate::table_definition::NativeModelOptions;
use crate::{watch, Database, DatabaseModel, Input};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

/// Builder that allows you to create a [`Database`](crate::Database) instance via [`create`](Self::create) or [`open`](Self::open) etc. and [define](Self::define) models.
#[derive(Debug)]
pub struct DatabaseBuilder {
    cache_size_bytes: Option<usize>,
    models_builder: HashMap<String, ModelBuilder>,
}

impl DatabaseBuilder {
    fn new_rdb_builder(&self) -> redb::Builder {
        let mut redb_builder = redb::Builder::new();
        if let Some(cache_size_bytes) = self.cache_size_bytes {
            redb_builder.set_cache_size(cache_size_bytes);
        }
        redb_builder
    }

    fn init<'a>(&'a self, redb_database: redb::Database) -> Result<Database<'a>> {
        let mut database = Database {
            instance: redb_database,
            primary_table_definitions: HashMap::new(),
            watchers: Arc::new(RwLock::new(watch::Watchers::new())),
            watchers_counter_id: AtomicU64::new(0),
        };

        for (_, model_builder) in &self.models_builder {
            database.seed_model(&model_builder)?;
        }

        Ok(database)
    }
}

impl DatabaseBuilder {
    /// Similar to [redb::Builder::new()](https://docs.rs/redb/latest/redb/struct.Builder.html#method.new).
    pub fn new() -> Self {
        Self {
            cache_size_bytes: None,
            models_builder: HashMap::new(),
        }
    }

    /// Similar to [redb::Builder::set_cache_size()](https://docs.rs/redb/latest/redb/struct.Builder.html#method.set_cache_size).
    pub fn set_cache_size(&mut self, bytes: usize) -> &mut Self {
        self.cache_size_bytes = Some(bytes);
        self
    }

    /// Creates a new `Db` instance using the given path.
    ///
    /// Similar to [redb::Builder.create(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.create)
    pub fn create(&self, path: impl AsRef<Path>) -> Result<Database> {
        let db = self.new_rdb_builder().create(path)?;
        // Ok(Self::from_redb(db))
        self.init(db)
    }

    /// Similar to [redb::Builder::open(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.open)
    pub fn open(&self, path: impl AsRef<Path>) -> Result<Database> {
        let db = self.new_rdb_builder().open(path)?;
        // Ok(Self::from_redb(db))
        self.init(db)
    }

    /// Creates a new [`Database`](crate::Database) instance in memory.
    pub fn create_in_memory(&self) -> Result<Database> {
        let in_memory_backend = redb::backends::InMemoryBackend::new();
        let db = self.new_rdb_builder();
        let db = db.create_with_backend(in_memory_backend)?;
        // Ok(Self::from_redb(db))
        self.init(db)
    }

    /// Defines a table using the given model.
    ///
    /// Native DB depends of `native_model` to define the model.
    /// And `native_model` by default uses [`serde`](https://serde.rs/) to serialize and deserialize the data but
    /// you can use any other serialization library see the documentation of [`native_model`](https://github.com/vincent-herlemont/native_model) for more information.
    /// So in the example below we import `serde` and we use the `Serialize` and `Deserialize` traits.
    ///
    /// # Primary key
    ///
    /// The primary key is *strict*, you **must**:
    /// - define it.
    /// - define only one.
    ///
    /// If the primary key is not defined, the compiler will return an error `Primary key is not set`.
    ///
    /// You can define with two ways:
    /// - `#[primary_key]` on the field
    /// - `#[native_db(primary_key(<method_name>))]` on any type `enum`, `struct`, `tuple struct` or `unit struct`.
    ///
    /// The primary key is **unique**, so you can't have two instances of the model with the same primary key saved in the database.
    ///
    /// ## Define a simple model with a primary key
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()
    /// }
    /// ```
    /// ## Define a model with a method as primary key
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db(
    ///     primary_key(custom_id)
    /// )]
    /// struct Data(u64);
    ///
    /// impl Data {
    ///   fn custom_id(&self) -> u32 {
    ///     (self.0 + 1) as u32
    ///   }
    /// }
    ///
    /// ```
    ///
    /// ## Secondary key
    ///
    /// The secondary key is *flexible*, you can:
    /// - define it or not.
    /// - define one or more.
    ///
    /// You can define with two ways:
    /// - `#[secondary_key]` on the field
    /// - `#[native_db(secondary_key(<method_name>, <options>))]` on any type `enum`, `struct`, `tuple struct` or `unit struct`.
    ///
    /// The secondary key can have two options:
    /// - [`unique`](#unique) (default: false)
    /// - [`optional`](#optional) (default: false)
    ///
    /// ## Define a model with a secondary key
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key]
    ///     name: String,
    /// }
    /// ```
    ///
    /// ## Define a model wit a secondary key optional and unique
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key(unique, optional)]
    ///     name: Option<String>,
    /// }
    /// ```
    /// - Note: the secondary key can be `unique` **or** `optional` as well.
    ///
    /// ## Unique
    ///
    /// This means that each instance of the model must have a unique value for the secondary key.
    /// If the value is not unique, the [`insert`](crate::transaction::RwTransaction::insert) method will return an error.
    ///
    /// ## Optional
    ///
    /// This means that an instance of the model can have a value for the secondary key or not.
    /// When`optional` is set the value **must** be an [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html).
    /// if the value is not an [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html) the compiler will return
    /// an error `error[E0282]: type annotations needed: cannot infer type`.
    ///  
    /// Under the hood, the secondary key is stored in a separate redb table. So if the secondary key is optional,
    /// the value will be stored in the table only if the value is not `None`.
    ///
    /// # Define a model with a secondary key and a custom secondary key optional
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db(
    ///     secondary_key(custom_name, optional)
    /// )]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key]
    ///     name: String,
    ///     flag: bool,
    /// }
    ///
    /// impl Data {
    ///     fn custom_name(&self) -> Option<String> {
    ///         if self.flag {
    ///             Some(self.name.clone().to_uppercase())
    ///         } else {
    ///             None
    ///         }
    ///     }
    /// }
    /// ```
    /// # Define multiple models
    ///
    /// To define multiple models, you **must** use different `id` for each model. If you use the same `id` for two models,
    /// the program will panic with the message `The table <table_name> has the same native model version as the table <table_name> and it's not allowed`.
    ///
    /// Example:
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Animal {
    ///     #[primary_key]
    ///     name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=2, version=1)]
    /// #[native_db]
    /// struct Vegetable {
    ///     #[primary_key]
    ///     name: String,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Animal>()?;
    ///     builder.define::<Vegetable>()
    /// }
    /// ```
    pub fn define<T: Input>(&mut self) -> Result<()> {
        let mut new_model_builder = ModelBuilder {
            model: T::native_db_model(),
            native_model_options: NativeModelOptions::default(),
        };

        new_model_builder.native_model_options.native_model_id = T::native_model_id();
        new_model_builder.native_model_options.native_model_version = T::native_model_version();

        // Set native model legacy
        for model in self.models_builder.values_mut() {
            if model.native_model_options.native_model_version
                > new_model_builder.native_model_options.native_model_version
            {
                model.native_model_options.native_model_legacy = false;
                new_model_builder.native_model_options.native_model_legacy = true;
            } else {
                model.native_model_options.native_model_legacy = true;
                new_model_builder.native_model_options.native_model_legacy = false;
            }

            // Panic if native model version are the same
            if model.native_model_options.native_model_id
                == new_model_builder.native_model_options.native_model_id
                && model.native_model_options.native_model_version
                    == new_model_builder.native_model_options.native_model_version
            {
                panic!(
                    "The table {} has the same native model version as the table {} and it's not allowed",
                    model.model.primary_key.unique_table_name,
                    new_model_builder.model.primary_key.unique_table_name,
                );
            }
        }

        self.models_builder.insert(
            new_model_builder
                .model
                .primary_key
                .unique_table_name
                .clone(),
            new_model_builder,
        );

        // for secondary_key in model.secondary_keys {
        //     model_builder.secondary_tables.insert(
        //         secondary_key.clone(),
        //         redb::TableDefinition::new(&secondary_key.table_name).into(),
        //     );
        // }
        // self.primary_table_definitions
        //     .insert(model.primary_key.table_name, primary_table_definition);

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ModelBuilder {
    pub(crate) model: DatabaseModel,
    pub(crate) native_model_options: NativeModelOptions,
}
