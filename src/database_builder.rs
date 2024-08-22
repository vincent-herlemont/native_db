use crate::database_instance::DatabaseInstance;
use crate::db_type::{Error, Result};
use crate::table_definition::NativeModelOptions;
use crate::{metadata, Models};
use crate::{upgrade, watch, Database, Model};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub(crate) struct Configuration {
    pub(crate) cache_size_bytes: Option<usize>,
}

impl Configuration {
    pub(crate) fn new_rdb_builder(&self) -> redb::Builder {
        let mut redb_builder = redb::Builder::new();
        if let Some(cache_size_bytes) = self.cache_size_bytes {
            redb_builder.set_cache_size(cache_size_bytes);
        }
        redb_builder
    }
}

#[cfg(feature = "redb1")]
impl Configuration {
    pub(crate) fn redb1_new_rdb1_builder(&self) -> redb1::Builder {
        let mut redb_builder = redb1::Builder::new();
        if let Some(cache_size_bytes) = self.cache_size_bytes {
            redb_builder.set_cache_size(cache_size_bytes);
        }
        redb_builder
    }
}
/// Builder that allows you to create a [`Database`](crate::Database) instance via [`create`](Self::create) or [`open`](Self::open) etc.
#[derive(Debug)]
pub struct Builder {
    database_configuration: Configuration,
}

impl Builder {
    fn init<'a>(
        &self,
        database_instance: DatabaseInstance,
        models: &'a Models,
    ) -> Result<Database<'a>> {
        let database_metadata = metadata::load_or_create_metadata(&database_instance)?;

        let mut database = Database {
            instance: database_instance,
            metadata: database_metadata,
            primary_table_definitions: HashMap::new(),
            watchers: Arc::new(RwLock::new(watch::Watchers::new())),
            watchers_counter_id: AtomicU64::new(0),
        };

        for (_, model_builder) in models.models_builder.iter() {
            database.seed_model(&model_builder)?;
        }

        // TODO: Maybe we can do some migration with models here.

        Ok(database)
    }
}

impl Builder {
    /// Construct a new [Builder] with sensible defaults.
    pub fn new() -> Self {
        Self {
            database_configuration: Configuration {
                cache_size_bytes: None,
            },
        }
    }

    /// Similar to [redb::Builder::set_cache_size()](https://docs.rs/redb/latest/redb/struct.Builder.html#method.set_cache_size).
    pub fn set_cache_size(&mut self, bytes: usize) -> &mut Self {
        self.database_configuration.cache_size_bytes = Some(bytes);
        self
    }

    /// Creates a new `Db` instance using the given path.
    ///
    /// Similar to [redb::Builder.create(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.create)
    pub fn create<'a>(&self, models: &'a Models, path: impl AsRef<Path>) -> Result<Database<'a>> {
        let builder = self.database_configuration.new_rdb_builder();
        let database_instance = DatabaseInstance::create_on_disk(builder, path)?;
        self.init(database_instance, models)
    }

    /// Similar to [redb::Builder::open(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.open)
    /// But it also upgrades the database if needed if
    pub fn open<'a>(&self, models: &'a Models, path: impl AsRef<Path>) -> Result<Database<'a>> {
        let builder = self.database_configuration.new_rdb_builder();
        let database_instance = match DatabaseInstance::open_on_disk(builder, &path) {
            Err(Error::RedbDatabaseError(redb::DatabaseError::UpgradeRequired(_))) => {
                upgrade::upgrade_redb(&self.database_configuration, &path, &models.models_builder)
            }
            Err(error) => return Err(error),
            Ok(database_instance) => Ok(database_instance),
        }?;
        upgrade::upgrade_underlying_database(&database_instance, &models.models_builder)?;
        self.init(database_instance, models)
    }

    /// Creates a new [`Database`](crate::Database) instance in memory.
    pub fn create_in_memory<'a>(&self, models: &'a Models) -> Result<Database<'a>> {
        let builder = self.database_configuration.new_rdb_builder();
        let database_instance = DatabaseInstance::create_in_memory(builder)?;
        self.init(database_instance, models)
    }
}

#[derive(Debug)]
pub(crate) struct ModelBuilder {
    pub(crate) model: Model,
    pub(crate) native_model_options: NativeModelOptions,
}
