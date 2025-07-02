use crate::database_builder::ModelBuilder;
use crate::database_instance::DatabaseInstance;
use crate::db_type::{Result, ToInput, Error};
use crate::stats::{Stats, StatsTable};
use crate::table_definition::PrimaryTableDefinition;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;
use crate::transaction::RTransaction;
use crate::transaction::RwTransaction;
use crate::watch::query::{InternalWatch, Watch};
use crate::{watch, Metadata};
use redb::{MultimapTableHandle, ReadableTableMetadata, TableHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

/// The database instance. Allows you to create [rw_transaction](database/struct.Database.html#method.rw_transaction) and [r_transaction](database/struct.Database.html#method.r_transaction), [watch](database/struct.Database.html#method.watch) queries, and [unwatch](database/struct.Database.html#method.unwatch) etc.
///
/// # Example
/// ```rust
/// use native_db::*;
///
/// fn main() -> Result<(), db_type::Error> {
///    let models = Models::new();
///    // Define models ...
///    let db = Builder::new().create_in_memory(&models)?;
///    // Open transactions
///    // Watch data
///    // Create snapshots
///    // etc...
///    Ok(())
/// }
pub struct Database<'a> {
    pub(crate) instance: DatabaseInstance,
    pub(crate) metadata: Metadata,
    pub(crate) primary_table_definitions: HashMap<String, PrimaryTableDefinition<'a>>,
    pub(crate) watchers: Arc<RwLock<watch::Watchers>>,
    pub(crate) watchers_counter_id: AtomicU64,
}

impl Database<'_> {
    /// Creates a new read-write transaction.
    /// This transaction allows you to read and write data.
    ///
    /// - Write operations:
    ///     - [`insert`](crate::transaction::RwTransaction::insert) - Insert a item.
    ///     - [`update`](crate::transaction::RwTransaction::update) - Update a item.
    ///     - [`remove`](crate::transaction::RwTransaction::remove) - Remove a item.
    ///     - [`migrate`](crate::transaction::RwTransaction::migrate) - Migrate a model, affect all items.
    ///     - [`commit`](crate::transaction::RwTransaction::commit) - Commit the transaction.
    ///     - [`abort`](crate::transaction::RwTransaction::abort) - Abort the transaction.
    /// - Read operations:
    ///    - [`get`](crate::transaction::RwTransaction::get) - Get a item.
    ///    - [`scan`](crate::transaction::RwTransaction::scan) - Scan items.
    ///    - [`len`](crate::transaction::RwTransaction::len) - Get the number of items.
    pub fn rw_transaction(&self) -> Result<RwTransaction> {
        let rw = self.instance.redb_database()?.begin_write()?;
        let write_txn = RwTransaction {
            watcher: &self.watchers,
            batch: RefCell::new(watch::Batch::new()),
            internal: InternalRwTransaction {
                redb_transaction: rw,
                primary_table_definitions: &self.primary_table_definitions,
            },
        };
        Ok(write_txn)
    }

    /// Creates a new read-only transaction.
    /// This transaction allows you to read data.
    ///
    /// - Read operations:
    ///   - [`get`](crate::transaction::RTransaction::get) - Get a item.
    ///   - [`scan`](crate::transaction::RTransaction::scan) - Scan items.
    ///   - [`len`](crate::transaction::RTransaction::len) - Get the number of items.
    pub fn r_transaction(&self) -> Result<RTransaction> {
        let txn = self.instance.redb_database()?.begin_read()?;
        let read_txn = RTransaction {
            internal: InternalRTransaction {
                redb_transaction: txn,
                table_definitions: &self.primary_table_definitions,
            },
        };
        Ok(read_txn)
    }
}

impl Database<'_> {
    /// Watch queries.
    ///
    /// - [`get`](crate::watch::query::Watch::get) - Watch a item.
    /// - [`scan`](crate::watch::query::Watch::scan) - Watch items.
    pub fn watch(&self) -> Watch {
        Watch {
            internal: InternalWatch {
                watchers: &self.watchers,
                watchers_counter_id: &self.watchers_counter_id,
            },
        }
    }

    /// Unwatch the given `id`.
    /// You can get the `id` from the return value of [`watch`](Self::watch).
    /// If the `id` is not valid anymore, this function will do nothing and return `false`.
    /// If the `id` is valid, the corresponding watcher will be removed and return `true`.
    /// If the `id` is valid but the watcher is already removed, this function will return `false`.
    pub fn unwatch(&self, id: u64) -> Result<bool> {
        let mut watchers = self.watchers.write().unwrap();
        Ok(watchers.remove_sender(id))
    }
}

impl<'a> Database<'a> {
    pub(crate) fn seed_model(&mut self, model_builder: &'a ModelBuilder) -> Result<()> {
        let main_table_definition =
            redb::TableDefinition::new(model_builder.model.primary_key.unique_table_name.as_str());
        let mut primary_table_definition: PrimaryTableDefinition =
            (model_builder, main_table_definition).into();

        let rw = self.instance.redb_database()?.begin_write()?;
        rw.open_table(primary_table_definition.redb)?;

        for secondary_key in model_builder.model.secondary_keys.iter() {
            primary_table_definition.secondary_tables.insert(
                secondary_key.clone(),
                redb::MultimapTableDefinition::new(secondary_key.unique_table_name.as_str()).into(),
            );
            rw.open_multimap_table(primary_table_definition.secondary_tables[secondary_key].redb)?;
        }
        rw.commit()?;

        self.primary_table_definitions.insert(
            model_builder.model.primary_key.unique_table_name.clone(),
            primary_table_definition,
        );

        Ok(())
    }

    /// Returns the [`Metadata`](crate::Metadata) of the database.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Check the integrity of the database.
    ///
    /// Similar to [redb::Database::check_integrity()](https://docs.rs/redb/latest/redb/struct.Database.html#method.check_integrity).
    pub fn check_integrity(&mut self) -> Result<bool> {
        self.instance.redb_database_mut()?.check_integrity()?;
        Ok(true)
    }

    /// Compact the database.
    ///
    /// Similar to [redb::Database::compact()](https://docs.rs/redb/latest/redb/struct.Database.html#method.compact).
    pub fn compact(&mut self) -> Result<bool> {
        self.instance.redb_database_mut()?.compact()?;
        Ok(true)
    }

    /// Returns true if the database is upgrading from the given version selector.
    ///
    /// - If the database is the old version, not matching the selector the function will return `false.
    /// - If the database is not upgrading, the function will return always `false`.
    ///
    /// Generally used with the method [refresh](crate::transaction::RwTransaction::refresh),
    /// to refresh the data for the given model.
    ///
    /// Check [release notes](https://github.com/vincent-herlemont/native_db/releases) to know when to use this method.
    ///
    /// # Example
    /// ```rust,ignore
    /// if db.upgrading_from_version("<0.8.0") {
    ///     // Do something that runs only when the database is upgrading from version <0.8.0.
    ///     // If the database is already at version 0.8.0, the function will return false and
    ///     // the code will not be executed.
    ///     let rw = db.rw_transaction().unwrap();
    ///     rw.refresh::<Item1>().unwrap();
    ///     rw.refresh::<Item2>().unwrap();
    ///     rw.commit().unwrap();
    /// }
    /// ```
    pub fn upgrading_from_version(&self, selector: &str) -> Result<bool> {
        use semver::Version;
        use semver::VersionReq;
        let metadata = self.metadata();
        let comparator = VersionReq::parse(selector)
            .unwrap_or_else(|_| panic!("Invalid version selector: {}", selector));

        let previous_version = if let Some(previous_version) = metadata.previous_version() {
            previous_version
        } else {
            return Ok(true);
        };

        let previous_version = Version::parse(previous_version)
            .unwrap_or_else(|_| panic!("Invalid previous version: {}", previous_version));
        let current_version = Version::parse(metadata.current_version())
            .unwrap_or_else(|_| panic!("Invalid current version: {}", metadata.current_version()));

        // If the previous version is the same as the current version, the database is not upgrading
        if previous_version == current_version {
            return Ok(false);
        }

        Ok(comparator.matches(&previous_version))
    }

    /// Enhanced version upgrade with closure support and automatic backup/rollback.
    ///
    /// This method extends `upgrading_from_version` to provide:
    /// - Automatic backup creation before migration
    /// - Rollback on migration failure  
    /// - User-defined migration logic via closure
    ///
    /// The closure receives a reference to the database and should perform the migration logic.
    /// If the closure returns an error, the database is automatically rolled back to the backup.
    ///
    /// # Example
    /// ```rust,ignore
    /// db.upgrading_from_version_with("<0.9.0", |db| {
    ///     // Migrate user models from v0.8.1 to current
    ///     db.migrate_model::<v081::User, current::User, _>(|old_user| {
    ///         current::User {
    ///             id: old_user.id,
    ///             username: old_user.name,  // Field renamed
    ///             email: old_user.email,
    ///             created_at: Utc::now(),   // New field
    ///         }
    ///     })
    /// })?;
    /// ```
    pub fn upgrading_from_version_with<F>(&self, selector: &str, migration_fn: F) -> Result<()>
    where
        F: FnOnce(&Self) -> Result<()>,
    {
        if self.upgrading_from_version(selector)? {
            // Only backup for on-disk databases
            let backup_path = if let Some(path) = self.instance.path() {
                let backup_path = path.with_extension("backup");
                std::fs::copy(path, &backup_path).map_err(|e| Error::Io(e))?;
                Some(backup_path)
            } else {
                None
            };

            match migration_fn(self) {
                Ok(()) => {
                    // Migration successful, remove backup
                    if let Some(backup_path) = backup_path {
                        let _ = std::fs::remove_file(backup_path); // Ignore errors on cleanup
                    }
                    Ok(())
                }
                Err(e) => {
                    // Migration failed, rollback if we have a backup
                    if let (Some(backup_path), Some(db_path)) = (backup_path, self.instance.path()) {
                        if let Err(rollback_err) = std::fs::rename(&backup_path, db_path) {
                            // If rollback fails, keep the backup and return both errors
                            return Err(Error::MigrationFailed { 
                                message: format!("Migration failed: {}. Rollback also failed: {}. Backup available at: {}", 
                                    e, rollback_err, backup_path.display())
                            });
                        }
                        Err(Error::MigrationRolledBack)
                    } else {
                        // In-memory database or no backup created
                        Err(e)
                    }
                }
            }
        } else {
            // No migration needed
            Ok(())
        }
    }

    /// Helper method to migrate all records of one model type to another.
    ///
    /// This method:
    /// 1. Reads all records of the old model type
    /// 2. Applies the transformation function to each record
    /// 3. Removes old records and inserts new ones
    /// 4. Commits the transaction
    ///
    /// # Example
    /// ```rust,ignore
    /// db.migrate_model::<v081::User, current::User, _>(|old_user| {
    ///     current::User {
    ///         id: old_user.id,
    ///         username: old_user.name,
    ///         email: old_user.email,
    ///         created_at: Utc::now(),
    ///     }
    /// })?;
    /// ```
    pub fn migrate_model<OldModel, NewModel, F>(&self, migration_fn: F) -> Result<()>
    where
        OldModel: ToInput + Clone,
        NewModel: ToInput + Clone,
        F: Fn(OldModel) -> NewModel,
    {
        let tx = self.rw_transaction()?;
        
        // Read all old models
        let old_records: std::result::Result<Vec<OldModel>, Error> = tx
            .scan()
            .primary::<OldModel>()?
            .all()?
            .collect();
        let old_records = old_records?;
        
        // Transform and replace records
        for old_record in old_records {
            let new_record = migration_fn(old_record.clone());
            
            // Remove old record first to avoid potential key conflicts
            tx.remove(old_record)?;
            
            // Insert new record
            tx.insert(new_record)?;
        }
        
        tx.commit()?;
        Ok(())
    }

    pub fn redb_stats(&self) -> Result<Stats> {
        let rx = self.instance.redb_database()?.begin_read()?;
        let mut stats_primary_tables = vec![];
        for primary_table in self.primary_table_definitions.values() {
            let result_table_open = rx.open_table(primary_table.redb);
            let stats_table = match result_table_open {
                Err(redb::TableError::TableDoesNotExist(_)) => StatsTable {
                    name: primary_table.redb.name().to_string(),
                    n_entries: None,
                },
                Ok(table_open) => {
                    let num_raw = table_open.len()?;
                    StatsTable {
                        name: primary_table.redb.name().to_string(),
                        n_entries: Some(num_raw),
                    }
                }
                Err(err) => {
                    return Err(err.into());
                }
            };
            stats_primary_tables.push(stats_table);
        }
        let mut stats_secondary_tables = vec![];
        for primary_table in self.primary_table_definitions.values() {
            for secondary_table in primary_table.secondary_tables.values() {
                let result_table_open = rx.open_multimap_table(secondary_table.redb);
                let stats_table = match result_table_open {
                    Err(redb::TableError::TableDoesNotExist(_)) => StatsTable {
                        name: secondary_table.redb.name().to_string(),
                        n_entries: None,
                    },
                    Ok(table_open) => {
                        let num_raw = table_open.len()?;
                        StatsTable {
                            name: secondary_table.redb.name().to_string(),
                            n_entries: Some(num_raw),
                        }
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                };
                stats_secondary_tables.push(stats_table);
            }
        }
        stats_primary_tables.sort_by(|a, b| a.name.cmp(&b.name));
        stats_secondary_tables.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Stats {
            primary_tables: stats_primary_tables,
            secondary_tables: stats_secondary_tables,
        })
    }
}
