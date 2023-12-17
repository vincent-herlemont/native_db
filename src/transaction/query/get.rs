use crate::db_type::{DatabaseSecondaryKeyOptions, InnerKeyValue, Input, KeyDefinition, Result};
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

pub struct RGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}

impl RGet<'_, '_> {
    pub fn primary<T: Input>(&self, key: impl InnerKeyValue) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_primary_key(model, key)?;
        Ok(result.map(|value| value.inner()))
    }

    pub fn secondary<T: Input>(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        Ok(result.map(|value| value.inner()))
    }
}

pub struct RwGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl RwGet<'_, '_> {
    pub fn primary<T: Input>(&self, key: impl InnerKeyValue) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_primary_key(model, key)?;
        Ok(result.map(|value| value.inner()))
    }

    pub fn secondary<T: Input>(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        Ok(result.map(|value| value.inner()))
    }
}
