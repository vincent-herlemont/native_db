use crate::database_builder::ModelBuilder;
use crate::database_instance::DatabaseInstance;
use crate::db_type::Result;
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
    pub fn rw_transaction(&self) -> Result<RwTransaction<'_>> {
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
    pub fn r_transaction(&self) -> Result<RTransaction<'_>> {
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
    /// **Warning**: Active watchers consume memory until explicitly removed with [`unwatch()`](Self::unwatch).
    /// When using the `tokio` feature, watchers use unbounded channels which can accumulate memory
    /// if events are not consumed. Always call `unwatch()` when done or ensure events are consumed
    /// to prevent memory accumulation.
    ///
    /// - [`get`](crate::watch::query::Watch::get) - Watch a item.
    /// - [`scan`](crate::watch::query::Watch::scan) - Watch items.
    pub fn watch(&self) -> Watch<'_> {
        Watch {
            internal: InternalWatch {
                watchers: &self.watchers,
                watchers_counter_id: &self.watchers_counter_id,
            },
        }
    }

    /// Unwatch the given `id`.
    ///
    /// **Important**: Always call this method when you're done watching to free memory.
    /// Failing to unwatch can lead to memory accumulation as watchers and their channels
    /// are kept in memory indefinitely.
    ///
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
