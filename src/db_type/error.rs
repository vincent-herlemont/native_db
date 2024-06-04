use crate::{db_type, watch};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redb error")]
    Redb(#[from] redb::Error),

    #[error("Redb database error")]
    RedbDatabaseError(#[from] redb::DatabaseError),

    #[cfg(feature = "redb1")]
    #[error("Legacy redb1 database error")]
    LegacyRedb1DatabaseError(#[from] redb1::DatabaseError),

    #[error("Redb transaction error")]
    RedbTransactionError(#[from] redb::TransactionError),

    #[cfg(feature = "redb1")]
    #[error("Redb redb1 transaction error")]
    Redb1TransactionError(#[from] redb1::TransactionError),

    #[error("Redb storage error")]
    RedbStorageError(#[from] redb::StorageError),

    #[error("Redb table error")]
    RedbTableError(#[from] redb::TableError),

    #[error("Redb commit error")]
    RedbCommitError(#[from] redb::CommitError),

    #[error("Redb compaction error")]
    RedbCompactionError(#[from] redb::CompactionError),

    #[error("Database instance need upgrade")]
    DatabaseInstanceNeedUpgrade(u8),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Table definition not found {table}")]
    TableDefinitionNotFound { table: String },

    #[error("Secondary key definition not found {table} {key}")]
    SecondaryKeyDefinitionNotFound { table: String, key: String },

    #[error("Secondary key constraint mismatch {table} {key} got: {got:?}")]
    SecondaryKeyConstraintMismatch {
        table: String,
        key: String,
        got: db_type::KeyOptions,
    },

    #[error("The secondary key {key_name} is not unique ")]
    NotUniqueSecondaryKey { key_name: String },

    #[error("Key not found {key:?}")]
    KeyNotFound { key: Vec<u8> },

    #[error("Primary key associated with the secondary key not found")]
    PrimaryKeyNotFound,

    #[error("Duplicate key for \"{key_name}\"")]
    DuplicateKey { key_name: String },

    #[error("Watch event error")]
    WatchEventError(#[from] watch::WatchEventError),

    #[error("Max watcher reached (should be impossible)")]
    MaxWatcherReached,

    #[error("You can not migrate the table {0} because it is a legacy model")]
    MigrateLegacyModel(String),

    #[error("Model error")]
    ModelError(#[from] native_model::Error),
}
