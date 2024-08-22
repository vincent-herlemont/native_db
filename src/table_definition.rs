use crate::database_builder::ModelBuilder;
use crate::db_type::{Key, KeyDefinition, KeyOptions};
use std::collections::HashMap;
use std::fmt::Debug;

pub(crate) type RedbPrimaryTableDefinition<'a> = redb::TableDefinition<'a, Key, &'static [u8]>;
pub(crate) type RedbSecondaryTableDefinition<'a> = redb::MultimapTableDefinition<'a, Key, Key>;

pub struct PrimaryTableDefinition<'a> {
    pub(crate) model: crate::Model,
    pub(crate) redb: RedbPrimaryTableDefinition<'a>,
    pub(crate) secondary_tables: HashMap<KeyDefinition<KeyOptions>, SecondaryTableDefinition<'a>>,
    pub(crate) native_model_options: NativeModelOptions,
}

#[derive(Clone, Debug)]
pub struct NativeModelOptions {
    pub(crate) native_model_id: u32,
    pub(crate) native_model_version: u32,
    // If a model as a new version, the old version is still available but marked as legacy.
    // NOTE: Is impossible to write or read on a legacy table definition.
    //       Just a migration to a new version is allowed.
    pub(crate) native_model_legacy: bool,
}

impl Default for NativeModelOptions {
    fn default() -> Self {
        Self {
            native_model_id: 0,
            native_model_version: 0,
            native_model_legacy: false,
        }
    }
}

impl<'a> From<(&ModelBuilder, RedbPrimaryTableDefinition<'a>)> for PrimaryTableDefinition<'a> {
    fn from(input: (&ModelBuilder, RedbPrimaryTableDefinition<'a>)) -> Self {
        let (builder, redb) = input;
        Self {
            model: builder.model.clone(),
            redb,
            secondary_tables: HashMap::new(),
            native_model_options: builder.native_model_options.clone(),
        }
    }
}

impl Debug for PrimaryTableDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use redb::TableHandle;
        f.debug_struct("TableDefinition")
            .field("name", &self.redb.name())
            .field("model_id", &self.native_model_options.native_model_id)
            .field(
                "model_version",
                &self.native_model_options.native_model_version,
            )
            .field("legacy", &self.native_model_options.native_model_legacy)
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct SecondaryTableDefinition<'a> {
    pub(crate) redb: RedbSecondaryTableDefinition<'a>,
}

impl<'a> From<RedbSecondaryTableDefinition<'a>> for SecondaryTableDefinition<'a> {
    fn from(rdb: RedbSecondaryTableDefinition<'a>) -> SecondaryTableDefinition<'a> {
        Self { redb: rdb }
    }
}
