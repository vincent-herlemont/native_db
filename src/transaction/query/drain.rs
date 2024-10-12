use crate::db_type::{KeyOptions, Result, ToInput, ToKeyDefinition};
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

pub struct RwDrain<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl<'db, 'txn> RwDrain<'db, 'txn> {
    /// Drain all items.
    ///
    /// **TODO: needs to be improved, so don't use it yet.**
    pub fn primary<T: ToInput>(&self) -> Result<Vec<T>> {
        let model = T::native_db_model();
        let out = self.internal.concrete_primary_drain(model)?;
        let out = out
            .into_iter()
            .map(|b| b.inner())
            .collect::<Result<Vec<T>>>()?;
        Ok(out)
    }

    /// Drain all items with a given secondary key.
    ///
    /// **TODO: needs to be implemented**
    pub fn secondary<T: ToInput>(&self, _key_def: impl ToKeyDefinition<KeyOptions>) {
        todo!()
    }
}
