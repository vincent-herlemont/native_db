use crate::{schema, SDBItem};
use redb::TableHandle;
use std::collections::HashMap;
use std::fmt::Debug;

#[cfg(not(feature = "use_native_model"))]
pub(crate) struct PrimaryTableDefinition {
    pub(crate) schema: crate::Schema,
    pub(crate) redb: redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
    pub(crate) secondary_tables: HashMap<&'static str, SecondaryTableDefinition>,
}

#[cfg(feature = "use_native_model")]
pub(crate) struct PrimaryTableDefinition {
    pub(crate) schema: crate::Schema,
    pub(crate) redb: redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
    pub(crate) secondary_tables: HashMap<&'static str, SecondaryTableDefinition>,
    pub(crate) native_model_id: u32,
    pub(crate) native_model_version: u32,
    // If a model as a new version, the old version is still available but marked as legacy.
    // NOTE: Is impossible to write or read on a legacy table definition.
    //       Just a migration to a new version is allowed.
    pub(crate) native_model_legacy: bool,
}

impl
    From<(
        schema::Schema,
        redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
    )> for PrimaryTableDefinition
{
    #[cfg(not(feature = "use_native_model"))]
    fn from(
        input: (
            schema::Schema,
            redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
        ),
    ) -> Self {
        let (schema, redb) = input;
        Self {
            schema,
            redb,
            secondary_tables: HashMap::new(),
        }
    }

    #[cfg(feature = "use_native_model")]
    fn from(
        input: (
            schema::Schema,
            redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
        ),
    ) -> Self {
        let (schema, redb) = input;
        Self {
            schema,
            redb,
            secondary_tables: HashMap::new(),
            native_model_id: 0,
            native_model_version: 0,
            native_model_legacy: false,
        }
    }
}

#[cfg(feature = "use_native_model")]
impl Debug for PrimaryTableDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableDefinition")
            .field("name", &self.redb.name())
            .field("model_id", &self.native_model_id)
            .field("model_version", &self.native_model_version)
            .field("legacy", &self.native_model_legacy)
            .finish()
    }
}

#[cfg(not(feature = "use_native_model"))]
impl Debug for PrimaryTableDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableDefinition")
            .field("name", &self.redb.name())
            .finish()
    }
}

pub(crate) struct SecondaryTableDefinition {
    pub(crate) rdb: redb::TableDefinition<'static, &'static [u8], &'static [u8]>,
}

impl From<redb::TableDefinition<'static, &'static [u8], &'static [u8]>>
    for SecondaryTableDefinition
{
    fn from(rdb: redb::TableDefinition<'static, &'static [u8], &'static [u8]>) -> Self {
        Self { rdb }
    }
}

impl SecondaryTableDefinition {
    pub(crate) fn rdb(&self) -> redb::TableDefinition<'static, &'static [u8], &'static [u8]> {
        self.rdb
    }
}
