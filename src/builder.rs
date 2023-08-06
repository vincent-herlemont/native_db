use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicU64;
use crate::{Db, watch};
use super::Result;

/// Builder for the [`Db`](super::Db) instance.
pub struct Builder {
    cache_size_bytes: Option<usize>,
}

impl Builder {
    /// Similar to [redb::Builder::new()](https://docs.rs/redb/latest/redb/struct.Builder.html#method.new).
    pub fn new() -> Self {
        Self {
            cache_size_bytes: None,
        }
    }

    fn new_rdb_builder(&self) -> redb::Builder {
        let mut redb_builder = redb::Builder::new();
        if let Some(cache_size_bytes) = self.cache_size_bytes {
            redb_builder.set_cache_size(cache_size_bytes);
        }
        redb_builder
    }

    fn new_redb(redb_database: redb::Database) -> Db {
        Db {
            instance: redb_database,
            table_definitions: HashMap::new(),
            watchers: Arc::new(RwLock::new(watch::Watchers::new())),
            watchers_counter_id: AtomicU64::new(0),
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
    pub fn create(&self,path: impl AsRef<Path>) -> Result<Db> {
        let db = self.new_rdb_builder().create(path)?;
        Ok(Self::new_redb(db))
    }

    /// Creates a new `Db` instance using [Builder::create] in order to create it in a temporary
    /// directory with the given path.
    ///
    /// Example: `builder::create_tmp('project/my_db')` will create the db to `/tmp/project/my_db`.
    pub fn create_tmp(&self, path: impl AsRef<Path>) -> Result<Db> {
        let tmp_dir = std::env::temp_dir();
        let tmp_dir = tmp_dir.join(path);
        self.create(tmp_dir.as_path())
    }

    /// Similar to [redb::Builder::open(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.open)
    pub fn open(&self, path: impl AsRef<Path>) -> Result<Db> {
        let db = self.new_rdb_builder().open(path)?;
        Ok(Self::new_redb(db))
    }

    /// Similar to [Builder::open] in order to open a database in a temporary directory with the
    /// given path.
    pub fn open_tmp(&self, path: impl AsRef<Path>) -> Result<Db> {
        let tmp_dir = std::env::temp_dir();
        let tmp_dir = tmp_dir.join(path);
        self.open(tmp_dir.as_path())
    }
}