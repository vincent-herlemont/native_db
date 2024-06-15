use crate::db_type::{Output, Result, ToInput};
use std::fmt::Debug;

#[derive(Clone)]
pub enum Event {
    Insert(Insert),
    Update(Update),
    Delete(Delete),
}

impl Event {
    pub(crate) fn new_insert(value: Output) -> Self {
        Self::Insert(Insert(value))
    }

    pub(crate) fn new_update(old_value: Output, new_value: Output) -> Self {
        Self::Update(Update {
            old: old_value,
            new: new_value,
        })
    }

    pub(crate) fn new_delete(value: Output) -> Self {
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
pub struct Insert(pub(crate) Output);

impl Insert {
    pub fn inner<T: ToInput>(&self) -> Result<T> {
        self.0.inner()
    }
}

#[derive(Clone)]
pub struct Update {
    pub(crate) old: Output,
    pub(crate) new: Output,
}

impl Update {
    pub fn inner_old<T: ToInput>(&self) -> Result<T> {
        self.old.inner()
    }
    pub fn inner_new<T: ToInput>(&self) -> Result<T> {
        self.new.inner()
    }
}

#[derive(Clone)]
pub struct Delete(pub(crate) Output);

impl Delete {
    pub fn inner<T: ToInput>(&self) -> Result<T> {
        self.0.inner()
    }
}
