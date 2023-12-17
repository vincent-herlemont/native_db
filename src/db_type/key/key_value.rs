use crate::db_type::DatabaseInnerKeyValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseKeyValue {
    Default(DatabaseInnerKeyValue),
    Optional(Option<DatabaseInnerKeyValue>),
}
