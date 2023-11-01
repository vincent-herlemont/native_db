use crate::table_definition::PrimaryTableDefinition;
use crate::ReadOnlyTables;
use std::collections::HashMap;

/// Can open only [`ReadOnlyTables`](crate::ReadOnlyTables).
pub struct ReadOnlyTransaction<'db> {
    pub(crate) table_definitions: &'db HashMap<&'static str, PrimaryTableDefinition>,
    pub(crate) txn: redb::ReadTransaction<'db>,
}

impl<'db> ReadOnlyTransaction<'db> {
    pub fn tables<'txn>(&'txn self) -> ReadOnlyTables<'db, 'txn> {
        ReadOnlyTables {
            table_definitions: self.table_definitions,
            opened_read_only_tables: Default::default(),
        }
    }
}
