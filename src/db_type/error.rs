use crate::{db_type, watch};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redb error")]
    Redb(#[from] Box<redb::Error>),

    #[error("Redb database error")]
    RedbDatabaseError(#[from] Box<redb::DatabaseError>),

    #[cfg(feature = "redb1")]
    #[error("Legacy redb1 database error")]
    LegacyRedb1DatabaseError(#[from] Box<redb1::DatabaseError>),

    #[error("Redb transaction error")]
    RedbTransactionError(#[from] Box<redb::TransactionError>),

    #[cfg(feature = "redb1")]
    #[error("Redb redb1 transaction error")]
    Redb1TransactionError(#[from] Box<redb1::TransactionError>),

    #[error("Redb storage error")]
    RedbStorageError(#[from] redb::StorageError),

    #[cfg(feature = "redb1")]
    #[error("Redb redb1 storage error")]
    Redb1StorageError(#[from] redb1::StorageError),

    #[error("Redb table error")]
    RedbTableError(#[from] redb::TableError),

    #[cfg(feature = "redb1")]
    #[error("Redb redb1 table error")]
    Redb1TableError(#[from] redb1::TableError),

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

    // TODO: key with key name.
    #[error("Key not found {key:?}")]
    KeyNotFound { key: Vec<u8> },

    #[error("Primary key associated with the secondary key not found")]
    PrimaryKeyNotFound,

    #[error("Duplicate key for \"{key_name}\"")]
    DuplicateKey { key_name: String },

    #[error("Missmatched key type for \"{key_name}\" expected {expected_types:?} got {got_types:?} during {operation:?}")]
    MissmatchedKeyType {
        key_name: String,
        expected_types: Vec<String>,
        got_types: Vec<String>,
        operation: String,
    },

    #[error("Watch event error")]
    WatchEventError(#[from] watch::WatchEventError),

    #[error("Max watcher reached (should be impossible)")]
    MaxWatcherReached,

    #[error("You can not migrate the table {0} because it is a legacy model")]
    MigrateLegacyModel(String),

    #[error("Model error")]
    ModelError(#[from] Box<native_model::Error>),

    #[error("Fail to remove secondary key: {0}")]
    RemoveSecondaryKeyError(String),

    #[error("Inccorect input data it does not match the model")]
    IncorrectInputData { value: Vec<u8> },
}

impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Error::Redb(Box::new(e))
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(e: redb::DatabaseError) -> Self {
        Error::RedbDatabaseError(Box::new(e))
    }
}

impl From<redb::TransactionError> for Error {
    fn from(e: redb::TransactionError) -> Self {
        Error::RedbTransactionError(Box::new(e))
    }
}

impl From<native_model::Error> for Error {
    fn from(e: native_model::Error) -> Self {
        Error::ModelError(Box::new(e))
    }
}

#[cfg(feature = "redb1")]
impl From<redb1::DatabaseError> for Error {
    fn from(e: redb1::DatabaseError) -> Self {
        Error::LegacyRedb1DatabaseError(Box::new(e))
    }
}

#[cfg(feature = "redb1")]
impl From<redb1::TransactionError> for Error {
    fn from(e: redb1::TransactionError) -> Self {
        Error::Redb1TransactionError(Box::new(e))
    }
}
