mod batch;
mod event;
mod filter;
mod request;
mod sender;

pub(crate) use batch::*;
pub use event::*;
pub(crate) use filter::*;
pub(crate) use request::*;
pub(crate) use sender::*;

use std::sync::{Arc, RwLock, TryLockError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatchEventError {
    #[error("TryLockErrorPoisoned")]
    TryLockErrorPoisoned(Batch),
    #[error("TryLockErrorWouldBlock")]
    TryLockErrorWouldBlock(Batch),
    #[error("SendError")]
    SendError(#[from] std::sync::mpsc::SendError<Event>),
}

pub(crate) fn push_batch(
    senders: Arc<RwLock<Watchers>>,
    batch: Batch,
) -> Result<(), WatchEventError> {
    let watchers = senders.try_read().map_err(|err| match err {
        TryLockError::Poisoned(_) => WatchEventError::TryLockErrorPoisoned(batch.clone()),
        TryLockError::WouldBlock => WatchEventError::TryLockErrorWouldBlock(batch.clone()),
    })?;

    for (watcher_request, event) in batch {
        for sender in watchers.find_senders(&watcher_request) {
            let sender = sender.lock().unwrap();
            sender.send(event.clone())?;
        }
    }

    Ok(())
}
