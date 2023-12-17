use crate::{db_type, watch};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redb error")]
    Redb(#[from] redb::Error),

    #[error("Redb database error")]
    RedbDatabaseError(#[from] redb::DatabaseError),

    #[error("Redb transaction error")]
    RedbTransactionError(#[from] redb::TransactionError),

    #[error("Redb storage error")]
    RedbStorageError(#[from] redb::StorageError),

    #[error("Redb table error")]
    RedbTableError(#[from] redb::TableError),

    #[error("Redb commit error")]
    RedbCommitError(#[from] redb::CommitError),

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
        got: db_type::DatabaseSecondaryKeyOptions,
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
}
