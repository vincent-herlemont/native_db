use crate::database_instance::DatabaseInstance;
use crate::db_type::{Result, UpgradeRequiredError};
use crate::table_definition::NativeModelOptions;
use crate::{metadata, Models};
use crate::{watch, Database, Model};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

/// Guard that ensures lock file cleanup on drop
struct LockFileGuard {
    path: PathBuf,
}

impl LockFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for LockFileGuard {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors
        let _ = fs::remove_file(&self.path);
    }
}

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

        // Check version compatibility
        self.check_version_compatibility(&database_metadata)?;

        let mut database = Database {
            instance: database_instance,
            metadata: database_metadata,
            primary_table_definitions: HashMap::new(),
            watchers: Arc::new(RwLock::new(watch::Watchers::new())),
            watchers_counter_id: AtomicU64::new(0),
        };

        for (_, model_builder) in models.models_builder.iter() {
            database.seed_model(model_builder)?;
        }

        // TODO: Maybe we can do some migration with models here.

        Ok(database)
    }

    fn check_version_compatibility(&self, metadata: &metadata::Metadata) -> Result<()> {
        let mut upgrade_error = UpgradeRequiredError::new();

        // Check Native DB version
        if metadata.current_version() != metadata::CURRENT_VERSION {
            upgrade_error = upgrade_error.with_native_db_version(
                metadata.current_version().to_string(),
                metadata::CURRENT_VERSION.to_string(),
            );
        }

        // Check Native Model version
        if metadata.current_native_model_version() != metadata::CURRENT_NATIVE_MODEL_VERSION {
            upgrade_error = upgrade_error.with_native_model_version(
                metadata.current_native_model_version().to_string(),
                metadata::CURRENT_NATIVE_MODEL_VERSION.to_string(),
            );
        }

        upgrade_error.build().map_err(|e| e.into())
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
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
    pub fn open<'a>(&self, models: &'a Models, path: impl AsRef<Path>) -> Result<Database<'a>> {
        let builder = self.database_configuration.new_rdb_builder();
        match DatabaseInstance::open_on_disk(builder, &path) {
            Ok(database_instance) => self.init(database_instance, models),
            Err(crate::db_type::Error::UpgradeRequired(boxed_upgrade_err)) => {
                let mut upgrade_err = *boxed_upgrade_err;
                // We already have a redb upgrade error, but we should still check other versions
                // by trying to read the metadata directly
                if let Ok(temp_builder) = self.try_read_metadata_for_upgrade_check(&path) {
                    if let Ok(metadata) = metadata::load_or_create_metadata(&temp_builder) {
                        // Check Native DB version
                        if metadata.current_version() != metadata::CURRENT_VERSION {
                            upgrade_err = UpgradeRequiredError {
                                details: upgrade_err.details,
                                native_db_version: Some((
                                    metadata.current_version().to_string(),
                                    metadata::CURRENT_VERSION.to_string(),
                                )),
                                native_model_version: upgrade_err.native_model_version,
                                redb_version: upgrade_err.redb_version,
                            };
                            upgrade_err.details.push(format!(
                                "  - Native DB: {} → {}",
                                metadata.current_version(),
                                metadata::CURRENT_VERSION
                            ));
                        }

                        // Check Native Model version
                        if metadata.current_native_model_version()
                            != metadata::CURRENT_NATIVE_MODEL_VERSION
                        {
                            upgrade_err = UpgradeRequiredError {
                                details: upgrade_err.details,
                                native_db_version: upgrade_err.native_db_version,
                                native_model_version: Some((
                                    metadata.current_native_model_version().to_string(),
                                    metadata::CURRENT_NATIVE_MODEL_VERSION.to_string(),
                                )),
                                redb_version: upgrade_err.redb_version,
                            };
                            upgrade_err.details.push(format!(
                                "  - Native Model: {} → {}",
                                metadata.current_native_model_version(),
                                metadata::CURRENT_NATIVE_MODEL_VERSION
                            ));
                        }
                    }
                }
                Err(Box::new(upgrade_err).into())
            }
            Err(e) => Err(e),
        }
    }

    fn try_read_metadata_for_upgrade_check(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<DatabaseInstance> {
        // Try to open in read-only mode to check metadata even if redb format is old
        // This might fail, but that's okay - we'll just report what we know
        let builder = redb::Builder::new();
        DatabaseInstance::open_on_disk(builder, path)
    }

    /// Creates a new [`Database`](crate::Database) instance in memory.
    pub fn create_in_memory<'a>(&self, models: &'a Models) -> Result<Database<'a>> {
        let builder = self.database_configuration.new_rdb_builder();
        let database_instance = DatabaseInstance::create_in_memory(builder)?;
        self.init(database_instance, models)
    }

    /// Upgrades an existing database to the current version by creating a new database
    /// and running a user-provided migration closure.
    ///
    /// This method implements a safe upgrade process:
    /// 1. Creates a new empty database with ".upgrading" suffix  
    /// 2. Runs the migration closure with a write transaction for the new database
    /// 3. On success, renames old database to ".old_vX.X.X" and new database to original path
    /// 4. Opens and returns the upgraded database
    ///
    /// # Important
    ///
    /// You MUST open and close the old database inside the migration closure. The closure
    /// receives a write transaction for the new database only. This design ensures proper
    /// isolation between old and new databases during migration.
    ///
    /// # Warnings
    ///
    /// ## Lock File Persistence
    /// If the upgrade process crashes or is forcibly terminated, the lock file
    /// (`.upgrade.lock`) will remain on disk, preventing future upgrade attempts.
    /// In such cases, you must manually verify that no upgrade is in progress and
    /// then delete the lock file before retrying.
    ///
    /// ## Backup Management
    /// This method creates backup files with the pattern `.old_vX.X.X` where X.X.X
    /// is the old database version. These backups are NOT automatically cleaned up.
    /// You should implement a retention policy based on your needs:
    /// - Keep only the most recent N backups
    /// - Delete backups older than a certain date
    /// - Manually verify and remove backups after successful migration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use native_db::*;
    /// # use native_db::db_type::Result;
    /// # fn example() -> Result<()> {
    /// # let models = Models::new();
    /// let new_db = Builder::new().upgrade(&models, "mydb.db", |new_txn| {
    ///     // Open old database inside closure
    ///     let old_models = Models::new(); // Define old models
    ///     let old_db = Builder::new().open(&old_models, "mydb.db")?;
    ///     
    ///     // Migrate data
    ///     let old_txn = old_db.r_transaction()?;
    ///     // ... perform migration logic ...
    ///     
    ///     // Old database automatically closed when it goes out of scope
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ".upgrading" database already exists (concurrent upgrade attempt)
    /// - The migration closure returns an error
    /// - File operations fail (create, rename, etc.)
    /// - The upgraded database cannot be opened
    pub fn upgrade<'a, F>(
        &self,
        models: &'a Models,
        path: impl AsRef<Path>,
        migration: F,
    ) -> Result<Database<'a>>
    where
        F: FnOnce(&mut crate::transaction::RwTransaction) -> Result<()>,
    {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();
        let upgrading_path = PathBuf::from(format!("{path_str}.upgrading"));
        let lock_path = PathBuf::from(format!("{path_str}.upgrade.lock"));

        // Try to create lock file atomically
        let _lock_file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true) // This ensures O_EXCL behavior - fails if file exists
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check if upgrading database exists to provide better error message
                if upgrading_path.exists() {
                    return Err(crate::db_type::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!(
                            "Upgrade already in progress. Database '{}' already exists. \
                            Remove it and the lock file '{}' manually if the previous upgrade failed.",
                            upgrading_path.display(),
                            lock_path.display()
                        ),
                    )));
                } else {
                    return Err(crate::db_type::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!(
                            "Upgrade already in progress. Lock file '{}' exists. \
                            Remove it manually if the previous upgrade failed.",
                            lock_path.display()
                        ),
                    )));
                }
            }
            Err(e) => {
                return Err(crate::db_type::Error::Io(std::io::Error::other(format!(
                    "Failed to create upgrade lock file '{}': {}",
                    lock_path.display(),
                    e
                ))));
            }
        };

        // Ensure lock file is removed on any exit path
        let _lock_guard = LockFileGuard::new(lock_path.clone());

        // Try to read metadata from old database to get version for backup naming
        let old_version = match self.try_read_metadata_for_upgrade_check(path) {
            Ok(old_instance) => match metadata::load_or_create_metadata(&old_instance) {
                Ok(old_metadata) => Some(old_metadata.current_version().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        };

        // Create new database with .upgrading suffix
        let new_db = self.create(models, &upgrading_path)?;

        // Run migration in a transaction
        let migration_result = {
            let mut txn = new_db.rw_transaction()?;
            let result = migration(&mut txn);
            if result.is_ok() {
                txn.commit()?;
            }
            result
        };

        // Handle migration result
        match migration_result {
            Ok(()) => {
                // Migration successful, perform atomic rename operations

                // Close the new database before renaming
                drop(new_db);

                // Sync the new database file to disk before rename operations
                // This ensures the migrated data is durable
                let upgrading_file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&upgrading_path)
                    .map_err(|e| {
                        crate::db_type::Error::Io(std::io::Error::other(format!(
                            "Failed to open upgrading database for sync: {e}"
                        )))
                    })?;
                upgrading_file.sync_all().map_err(|e| {
                    crate::db_type::Error::Io(std::io::Error::other(format!(
                        "Failed to sync upgrading database to disk: {e}"
                    )))
                })?;
                drop(upgrading_file);

                // Create backup path with old version if available
                let backup_path = if let Some(version) = old_version {
                    PathBuf::from(format!("{path_str}.old_v{version}"))
                } else {
                    PathBuf::from(format!("{path_str}.old"))
                };

                // Rename old database to backup
                fs::rename(path, &backup_path).map_err(|e| {
                    crate::db_type::Error::Io(std::io::Error::other(format!(
                        "Failed to rename old database '{}' to '{}': {}",
                        path.display(),
                        backup_path.display(),
                        e
                    )))
                })?;

                // Rename new database to original path
                if let Err(rename_err) = fs::rename(&upgrading_path, path) {
                    // Try to restore old database if rename fails
                    match fs::rename(&backup_path, path) {
                        Ok(()) => {
                            // Rollback successful
                            return Err(crate::db_type::Error::Io(std::io::Error::other(format!(
                                "Failed to complete upgrade: could not rename new database '{}' to '{}': {}. \
                                The original database has been restored successfully.",
                                upgrading_path.display(),
                                path.display(),
                                rename_err
                            ))));
                        }
                        Err(rollback_err) => {
                            // Rollback also failed - critical situation
                            return Err(crate::db_type::Error::Io(std::io::Error::other(format!(
                                "CRITICAL: Upgrade failed and rollback also failed. \
                                Failed to rename new database '{}' to '{}': {}. \
                                Failed to restore backup '{}' to original location: {}. \
                                Your original database is preserved at '{}'. \
                                The new database is at '{}'. \
                                To recover: manually rename '{}' back to '{}'.",
                                upgrading_path.display(),
                                path.display(),
                                rename_err,
                                backup_path.display(),
                                rollback_err,
                                backup_path.display(),
                                upgrading_path.display(),
                                backup_path.display(),
                                path.display()
                            ))));
                        }
                    }
                }

                // Sync the parent directory to ensure rename operations are durable
                // This is important for crash safety of directory metadata
                // Note: On Windows, directories cannot be opened as files for sync operations
                #[cfg(not(target_os = "windows"))]
                if let Some(parent) = path.parent() {
                    let dir = fs::File::open(parent).map_err(|e| {
                        crate::db_type::Error::Io(std::io::Error::other(format!(
                            "Failed to open parent directory for sync: {e}"
                        )))
                    })?;
                    dir.sync_all().map_err(|e| {
                        crate::db_type::Error::Io(std::io::Error::other(format!(
                            "Failed to sync parent directory to disk: {e}"
                        )))
                    })?;
                }

                // Open and return the upgraded database
                self.open(models, path)
            }
            Err(e) => {
                // Migration failed, clean up
                drop(new_db);
                if let Err(cleanup_err) = fs::remove_file(&upgrading_path) {
                    // Log cleanup failure but return original error
                    eprintln!(
                        "Warning: Failed to clean up upgrading database '{}' after migration failure: {}",
                        upgrading_path.display(),
                        cleanup_err
                    );
                }
                Err(e)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct ModelBuilder {
    pub(crate) model: Model,
    pub(crate) native_model_options: NativeModelOptions,
}
