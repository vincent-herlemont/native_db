use crate::table_definition::PrimaryTableDefinition;
use crate::watch;
use crate::{Result, Tables};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Can open only [`Tables`](crate::Tables).
pub struct Transaction<'db> {
    pub(crate) table_definitions: &'db HashMap<&'static str, PrimaryTableDefinition>,
    pub(crate) txn: redb::WriteTransaction<'db>,
    pub(crate) watcher: &'db Arc<RwLock<watch::Watchers>>,
    pub(crate) batch: RefCell<watch::Batch>,
}

impl<'db> Transaction<'db> {
    pub fn tables<'txn>(&'txn self) -> Tables<'db, 'txn> {
        Tables {
            table_definitions: self.table_definitions,
            opened_tables: HashMap::new(),
            batch: &self.batch,
        }
    }

    pub fn commit(self) -> Result<()> {
        self.txn.commit()?;
        // Send batch to watchers after commit succeeds
        let batch = self.batch.into_inner();
        watch::push_batch(Arc::clone(&self.watcher), batch)?;
        Ok(())
    }
}
