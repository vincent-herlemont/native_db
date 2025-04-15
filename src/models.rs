use std::{cmp::Ordering, collections::HashMap};

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
/// #     use native_db::{
/// #         native_db,
/// #         native_model::{self, native_model, Model},
/// #         ToKey,
/// #     };
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

    ///
    /// # Global Options
    ///
    /// `export_keys`: You can export the keys enum using the `export_keys` option, example: `#[native_db(export_keys = true)]`.
    /// This option makes the keys enum visible outside of the crate with `pub` visibility, default value is `false` with visibility limited to `pub(crate)`.
    ///
    /// # Keys and Models
    ///
    /// Defines a table using the given model.
    ///
    /// Native DB depends on `native_model` to define the model.
    /// By default, `native_model` uses [`serde`](https://serde.rs/) to serialize and deserialize the data, but
    /// you can use any other serialization library. See the documentation of [`native_model`](https://github.com/vincent-herlemont/native_model) for more information.
    /// In the examples below, we import `serde` and use the `Serialize` and `Deserialize` traits.
    ///
    ///
    /// ## Primary Key
    ///
    /// The primary key is **mandatory**, and you **must**:
    /// - Define it.
    /// - Define only one.
    ///
    /// If the primary key is not defined, the compiler will return an error: `Primary key is not set`.
    ///
    /// There are two ways to define a primary key:
    ///
    /// 1. **On a Field**:
    ///    - Use the `#[primary_key]` attribute on the field that will serve as the primary key.
    ///    - The type of the field will be used as the primary key type.
    ///
    /// 2. **With a Custom Method**:
    ///    - Use the `#[native_db(primary_key(<method_name> -> <return_type>))]` attribute on the type (`enum`, `struct`, `tuple struct`, or `unit struct`).
    ///    - Implement a method with the given `<method_name>` that returns the primary key of type `<return_type>`.
    ///    - **Important:** You must specify both the method name and the return type using the syntax `primary_key(<method_name> -> <return_type>)`. The type must be specified because it is used at runtime to check the query types.
    ///
    /// The primary key is **unique**, so you can't have two instances of the model with the same primary key saved in the database.
    ///
    /// ### Defining a Simple Model with a Primary Key on a Field
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///
    /// In this example, we have:
    /// - **One primary key** named `id` of type `u64`, defined directly on the field using the `#[primary_key]` attribute.
    ///
    /// ### Defining a Model with a Method as Primary Key
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db(
    ///     primary_key(custom_id -> u32)
    /// )]
    /// struct Data(u64);
    ///
    /// impl Data {
    ///     fn custom_id(&self) -> u32 {
    ///         (self.0 + 1) as u32
    ///     }
    /// }
    /// ```
    ///
    /// In this example, we have:
    /// - **One primary key** named `custom_id` of type `u32`, defined using a custom method. The method `custom_id` computes and returns the primary key value.
    ///
    /// ## Secondary Key
    ///
    /// The secondary key is *flexible*, and you can:
    /// - Define it or not.
    /// - Define one or more.
    ///
    /// There are two ways to define a secondary key:
    ///
    /// 1. **On a Field**:
    ///    - Use the `#[secondary_key]` attribute on the field that will serve as a secondary key.
    ///    - The type of the field will be used as the secondary key type.
    ///
    /// 2. **With a Custom Method**:
    ///    - Use the `#[native_db(secondary_key(<method_name> -> <return_type>, <options>))]` attribute on the type.
    ///    - Implement a method with the given `<method_name>` that returns the secondary key value of type `<return_type>`.
    ///    - **Important:** You must specify both the method name and the return type using the syntax `secondary_key(<method_name> -> <return_type>, <options>)`. The type must be specified because it is used at runtime to check the query types.
    ///
    /// The secondary key can have two options:
    /// - [`unique`](#unique) (default: false)
    /// - [`optional`](#optional) (default: false)
    ///
    /// ### Defining a Model with a Secondary Key on a Field
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    /// In the above example, we have:
    /// - **One primary key** named `id` of type `u64`, defined on the field.
    /// - **One secondary key** named `name` of type `String`, defined on the field using the `#[secondary_key]` attribute.
    ///
    /// ### Defining a Model with an Optional and Unique Secondary Key
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///
    /// In the above example, we have:
    /// - **One primary key** named `id` of type `u64`, defined on the field.
    /// - **One secondary key** named `name` of type `Option<String>`, defined on the field with options `unique` and `optional`.
    ///
    /// - **Note:** The secondary key can be `unique`, `optional`, or both.
    ///
    /// ### Unique
    ///
    /// This means that each instance of the model must have a unique value for the secondary key.
    /// If the value is not unique, the [`insert`](crate::transaction::RwTransaction::insert) method will return an error.
    ///
    /// ### Optional
    ///
    /// This means that an instance of the model can have a value for the secondary key or not.
    /// When `optional` is set, the value **must** be an [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html).
    /// If the value is not an [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html), the compiler will return
    /// an error: `error[E0282]: type annotations needed: cannot infer type`.
    ///
    /// Under the hood, the secondary key is stored in a separate `redb` table. So if the secondary key is optional,
    /// the value will be stored in the table only if the value is not `None`.
    ///
    /// ### Defining a Model with a Custom Optional Secondary Key
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db(
    ///     secondary_key(custom_name -> Option<String>, optional)
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
    ///
    /// In the above example, we have:
    /// - **One primary key** named `id` of type `u64`, defined on the field.
    /// - **One secondary key** named `name` of type `String`, defined on the field.
    /// - **One custom secondary key** named `custom_name` of type `Option<String>`, defined using a custom method with the option `optional`.
    ///
    /// The method `custom_name` returns an `Option<String>` based on some logic involving the `flag` field.
    ///
    /// # Defining Multiple Models
    ///
    /// To define multiple models, you **must** use different `id` values for each model. If you use the same `id` for two models,
    /// the program will panic with the message: `The table <table_name> has the same native model version as the table <table_name> and it's not allowed`.
    ///
    /// Example:
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///
    /// In the above example, we have:
    /// - We have two models, `Animal` and `Vegetable`.
    /// - Both have:
    ///   - **One primary key** named `name` of type `String`, defined on the field.
    /// - Each model has a unique `id` (`id=1` for `Animal`, `id=2` for `Vegetable`), which is necessary to avoid conflicts.
    pub fn define<T: ToInput>(&mut self) -> Result<()> {
        let mut new_model_builder = ModelBuilder {
            model: T::native_db_model(),
            native_model_options: NativeModelOptions::default(),
        };

        new_model_builder.native_model_options.native_model_id = T::native_model_id();
        new_model_builder.native_model_options.native_model_version = T::native_model_version();

        // Set native model legacy
        for model in self.models_builder.values_mut() {
            if model.native_model_options.native_model_id
                != new_model_builder.native_model_options.native_model_id
            {
                continue;
            }

            match model
                .native_model_options
                .native_model_version
                .cmp(&new_model_builder.native_model_options.native_model_version)
            {
                Ordering::Greater => {
                    model.native_model_options.native_model_legacy = false;
                    new_model_builder.native_model_options.native_model_legacy = true;
                }
                Ordering::Less => {
                    model.native_model_options.native_model_legacy = true;
                    new_model_builder.native_model_options.native_model_legacy = false;
                }
                Ordering::Equal => {
                    panic!(
                        "The table {} has the same native model version as the table {} and it's not allowed",
                        model.model.primary_key.unique_table_name,
                        new_model_builder.model.primary_key.unique_table_name
                    )
                }
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
