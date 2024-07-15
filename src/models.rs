use std::collections::HashMap;

use crate::{db_type::Result, table_definition::NativeModelOptions, ModelBuilder, ToInput};

/// A collection of [`Model`](crate::Model) used by the [`Models`](crate::Models) to
/// [define](Self::define) models.
///
/// This collection allows you to manage multiple models efficiently, facilitating the process
/// of defining and manipulating them within your application.
///
/// # Note
/// Usually, there is little point in creating models at runtime. In some cases, it is necessary to define them with a `'static` lifetime, for example, to address compatibility issues with certain asynchronous libraries such as [Axum](https://github.com/tokio-rs/axum).
/// There are multiple ways to achieve this, including the [`once_cell::sync::Lazy`](https://docs.rs/once_cell/1.19.0/once_cell/sync/struct.Lazy.html) crate,
/// or the [`LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html) from the standard library, which is available when the relevant Rust feature is enabled.
///
/// ## Example using `once_cell::sync::Lazy`
///
/// ```rust
/// # pub mod data {
/// #     use native_db::{native_db, ToKey};
/// #     use native_model::{native_model, Model};
/// #     use serde::{Deserialize, Serialize};
/// #
/// #     pub type Person = v1::Person;
/// #
/// #     pub mod v1 {
/// #         use super::*;
/// #         
/// #         #[derive(Serialize, Deserialize, Debug)]
/// #         #[native_model(id = 1, version = 1)]
/// #         #[native_db]
/// #         pub struct Person {
/// #            #[primary_key]
/// #            pub name: String,
/// #         }
/// #     }
/// # }
/// use native_db::*;
/// use once_cell::sync::Lazy;
///
/// // The lifetime of the models needs to be longer or equal to the lifetime of the database.
/// // In many cases, it is simpler to use a static variable but it is not mandatory.
/// static MODELS: Lazy<Models> = Lazy::new(|| {
///     let mut models = Models::new();
///     // It's a good practice to define the models by specifying the version
///     models.define::<data::v1::Person>().unwrap();
///     models
/// });
///
/// fn main() -> Result<(), db_type::Error> {
///     // Initialize the database with the models
///     let db = Builder::new().create_in_memory(&MODELS)?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct Models {
    pub(crate) models_builder: HashMap<String, ModelBuilder>,
}

impl Models {
    /// Create a new collection of Models.
    pub fn new() -> Self {
        Self {
            models_builder: HashMap::new(),
        }
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
    ///     let mut models = Models::new();
    ///     models.define::<Data>()
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
    ///     let mut models = Models::new();
    ///     models.define::<Animal>()?;
    ///     models.define::<Vegetable>()
    /// }
    /// ```
    pub fn define<T: ToInput>(&mut self) -> Result<()> {
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

        Ok(())
    }
}
