use crate::db_type::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEntry {
    Default(Key),
    Optional(Option<Key>),
}
