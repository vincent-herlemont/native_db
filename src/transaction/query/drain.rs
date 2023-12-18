use crate::db_type::{DatabaseSecondaryKeyOptions, Input, KeyDefinition, Result};
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

pub struct RwDrain<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl<'db, 'txn> RwDrain<'db, 'txn> {
    pub fn primary<T: Input>(&self) -> Result<Vec<T>> {
        let model = T::native_db_model();
        let out = self.internal.concrete_primary_drain(model)?;
        Ok(out.into_iter().map(|b| b.inner()).collect())
    }

    /// **TODO: needs to be implemented**
    pub fn secondary<T: Input>(
        &self,
        _key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> () {
        todo!()
    }
}
