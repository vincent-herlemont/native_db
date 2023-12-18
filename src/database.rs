use crate::database_builder::ModelBuilder;
use crate::db_type::Result;
use crate::stats::{Stats, StatsTable};
use crate::table_definition::PrimaryTableDefinition;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;
use crate::transaction::RTransaction;
use crate::transaction::RwTransaction;
use crate::watch;
use crate::watch::query::{InternalWatch, Watch};
use redb::TableHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::u64;

/// The database instance. Allows you to create [rw_transaction](database/struct.Database.html#method.rw_transaction) and [r_transaction](database/struct.Database.html#method.r_transaction), [watch](database/struct.Database.html#method.watch) queries, and [unwatch](database/struct.Database.html#method.unwatch) etc.
///
/// # Example
/// ```rust
/// use native_db::*;
///
/// fn main() -> Result<(), db_type::Error> {
///    let builder = DatabaseBuilder::new();
///    // Define models ...
///    let db = builder.create_in_memory()?;
///    // Open transactions
///    // Watch data
///    // Create snapshots
///    // etc...
///    Ok(())
/// }
pub struct Database<'a> {
    pub(crate) instance: redb::Database,
    pub(crate) primary_table_definitions: HashMap<String, PrimaryTableDefinition<'a>>,
    pub(crate) watchers: Arc<RwLock<watch::Watchers>>,
    pub(crate) watchers_counter_id: AtomicU64,
}

impl Database<'_> {
    /// Creates a new read-write transaction.
    pub fn rw_transaction(&self) -> Result<RwTransaction> {
        let rw = self.instance.begin_write()?;
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
    pub fn r_transaction(&self) -> Result<RTransaction> {
        let txn = self.instance.begin_read()?;
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
    /// If the `id` is not valid anymore, this function will do nothing.
    /// If the `id` is valid, the corresponding watcher will be removed.
    pub fn unwatch(&self, id: u64) -> Result<()> {
        let mut watchers = self.watchers.write().unwrap();
        watchers.remove_sender(id);
        Ok(())
    }
}

impl<'a> Database<'a> {
    pub(crate) fn seed_model(&mut self, model_builder: &'a ModelBuilder) -> Result<()> {
        let main_table_definition =
            redb::TableDefinition::new(model_builder.model.primary_key.unique_table_name.as_str());
        let mut primary_table_definition: PrimaryTableDefinition =
            (model_builder, main_table_definition).into();

        let rw = self.instance.begin_write()?;
        rw.open_table(primary_table_definition.redb.clone())?;

        for secondary_key in model_builder.model.secondary_keys.iter() {
            primary_table_definition.secondary_tables.insert(
                secondary_key.clone(),
                redb::TableDefinition::new(secondary_key.unique_table_name.as_str()).into(),
            );
            rw.open_table(
                primary_table_definition.secondary_tables[&secondary_key]
                    .redb
                    .clone(),
            )?;
        }
        rw.commit()?;

        self.primary_table_definitions.insert(
            model_builder.model.primary_key.unique_table_name.clone(),
            primary_table_definition,
        );

        Ok(())
    }

    pub fn redb_stats(&self) -> Result<Stats> {
        use redb::ReadableTable;
        let rx = self.instance.begin_read()?;
        let mut stats_primary_tables = vec![];
        for primary_table in self.primary_table_definitions.values() {
            let result_table_open = rx.open_table(primary_table.redb.clone());
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
                let result_table_open = rx.open_table(secondary_table.redb.clone());
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
