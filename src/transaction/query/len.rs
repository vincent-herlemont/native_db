use crate::db_type::{DatabaseSecondaryKeyOptions, Input, KeyDefinition, Result};
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

pub struct RLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}

impl RLen<'_, '_> {
    pub fn primary<T: Input>(&self) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.internal.primary_len(model)?;
        Ok(result)
    }

    pub fn secondary<T: Input>(
        &self,
        _key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> Result<Option<T>> {
        todo!()
    }
}

pub struct RwLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl RwLen<'_, '_> {
    pub fn primary<T: Input>(&self) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.internal.primary_len(model)?;
        Ok(result)
    }

    pub fn secondary<T: Input>(
        &self,
        _key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> Result<Option<T>> {
        todo!()
    }
}
