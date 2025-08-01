use crate::db_type::{Result, UpgradeRequiredError};
use redb::Builder;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct DatabaseInstance {
    kind: DatabaseInstanceKind,
}

impl DatabaseInstance {
    pub(crate) fn create_on_disk(builder: Builder, path: impl AsRef<Path>) -> Result<Self> {
        let db = builder.create(path.as_ref())?;
        Ok(Self {
            kind: DatabaseInstanceKind::OnDisk {
                redb_database: db,
                path: path.as_ref().to_path_buf(),
            },
        })
    }

    pub(crate) fn open_on_disk(builder: Builder, path: impl AsRef<Path>) -> Result<Self> {
        match builder.open(path.as_ref()) {
            Ok(db) => Ok(Self {
                kind: DatabaseInstanceKind::OnDisk {
                    redb_database: db,
                    path: path.as_ref().to_path_buf(),
                },
            }),
            Err(redb::DatabaseError::UpgradeRequired(version)) => Err(Box::new(
                UpgradeRequiredError::new().with_redb_version(version),
            )
            .build()
            .unwrap_err()
            .into()),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) fn create_in_memory(builder: Builder) -> Result<Self> {
        let in_memory_backend = redb::backends::InMemoryBackend::new();
        let db = builder.create_with_backend(in_memory_backend)?;
        Ok(Self {
            kind: DatabaseInstanceKind::InMemory { redb_database: db },
        })
    }

    pub(crate) fn redb_database(&self) -> Result<&redb::Database> {
        self.kind.redb_database()
    }

    pub(crate) fn redb_database_mut(&mut self) -> Result<&mut redb::Database> {
        self.kind.redb_database_mut()
    }
}

enum DatabaseInstanceKind {
    InMemory {
        redb_database: redb::Database,
    },
    OnDisk {
        redb_database: redb::Database,
        #[allow(dead_code)]
        path: PathBuf,
    },
}

impl DatabaseInstanceKind {
    pub(crate) fn redb_database(&self) -> Result<&redb::Database> {
        match self {
            DatabaseInstanceKind::InMemory { redb_database } => Ok(redb_database),
            DatabaseInstanceKind::OnDisk { redb_database, .. } => Ok(redb_database),
        }
    }

    pub(crate) fn redb_database_mut(&mut self) -> Result<&mut redb::Database> {
        match self {
            DatabaseInstanceKind::InMemory { redb_database } => Ok(redb_database),
            DatabaseInstanceKind::OnDisk { redb_database, .. } => Ok(redb_database),
        }
    }
}
