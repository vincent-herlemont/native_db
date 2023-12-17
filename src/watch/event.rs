use crate::db_type::{DatabaseOutputValue, Input};
use std::fmt::Debug;

#[derive(Clone)]
pub enum Event {
    Insert(Insert),
    Update(Update),
    Delete(Delete),
}

impl Event {
    pub(crate) fn new_insert(value: DatabaseOutputValue) -> Self {
        Self::Insert(Insert(value))
    }

    pub(crate) fn new_update(
        old_value: DatabaseOutputValue,
        new_value: DatabaseOutputValue,
    ) -> Self {
        Self::Update(Update {
            old: old_value,
            new: new_value,
        })
    }

    pub(crate) fn new_delete(value: DatabaseOutputValue) -> Self {
        Self::Delete(Delete(value))
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Insert(_) => write!(f, "Insert"),
            Event::Update(_) => write!(f, "Update"),
            Event::Delete(_) => write!(f, "Delete"),
        }
    }
}

#[derive(Clone)]
pub struct Insert(pub(crate) DatabaseOutputValue);

impl Insert {
    pub fn inner<T: Input>(&self) -> T {
        self.0.inner()
    }
}

#[derive(Clone)]
pub struct Update {
    pub(crate) old: DatabaseOutputValue,
    pub(crate) new: DatabaseOutputValue,
}

impl Update {
    pub fn inner_old<T: Input>(&self) -> T {
        self.old.inner()
    }
    pub fn inner_new<T: Input>(&self) -> T {
        self.new.inner()
    }
}

#[derive(Clone)]
pub struct Delete(pub(crate) DatabaseOutputValue);

impl Delete {
    pub fn inner<T: Input>(&self) -> T {
        self.0.inner()
    }
}
