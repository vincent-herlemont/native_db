use crate::{db_type, watch};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redb error")]
    Redb(#[from] Box<redb::Error>),

    #[error("Redb database error")]
    RedbDatabaseError(#[from] Box<redb::DatabaseError>),

    #[error("Redb transaction error")]
    RedbTransactionError(#[from] Box<redb::TransactionError>),

    #[error("Redb storage error")]
    RedbStorageError(#[from] redb::StorageError),

    #[error("Redb table error")]
    RedbTableError(#[from] redb::TableError),

    #[error("Redb commit error")]
    RedbCommitError(#[from] redb::CommitError),

    #[error("Redb compaction error")]
    RedbCompactionError(#[from] redb::CompactionError),

    #[error(transparent)]
    UpgradeRequired(#[from] Box<crate::db_type::UpgradeRequiredError>),

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

    #[error("Mismatched key type for \"{key_name}\" expected {expected_types:?} got {got_types:?} during {operation:?}")]
    MismatchedKeyType {
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

    #[error("Upgrade migration error: {context}")]
    UpgradeMigration {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
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
