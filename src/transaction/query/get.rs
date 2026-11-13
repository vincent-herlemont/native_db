use crate::db_type::{
    check_key_type, check_key_type_from_key_definition, KeyOptions, Result, ToInput, ToKey,
    ToKeyDefinition,
};
use crate::transaction::inner::GetInner;
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get a value from the database.
pub struct RGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}
pub struct RwGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}
impl<'db, 'txn> GetTrait<'db, 'txn> for RGet<'db, 'txn> {}
impl<'db, 'txn> GetTrait<'db, 'txn> for RwGet<'db, 'txn> {}

pub trait GetTrait<'db, 'txn>: GetInner<'db, 'txn>
where
    'db: 'txn,
{
    fn primary<T: ToInput>(&'txn self, key: impl ToKey) -> Result<Option<T>> {
        let model = T::native_db_model();
        check_key_type(&model, &key)?;
        let inner = self.inner();
        let result = inner.get_by_primary_key(model, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }

    fn secondary<T: ToInput>(
        &'txn self,
        key_def: impl ToKeyDefinition<KeyOptions>,
        key: impl ToKey,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        check_key_type_from_key_definition(&key_def.key_definition(), &key)?;
        let result = self.inner().get_by_secondary_key(model, key_def, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }
}
