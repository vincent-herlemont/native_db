mod batch;
mod event;
mod filter;
pub mod query;
mod request;
mod sender;

pub(crate) use batch::*;
pub use event::*;
pub(crate) use filter::*;
pub(crate) use request::*;
pub(crate) use sender::*;

use std::{
    sync::{Arc, RwLock},
    vec,
};

#[cfg(not(feature = "tokio"))]
use std::sync::mpsc::SendError;
#[cfg(feature = "tokio")]
use tokio::sync::mpsc::error::SendError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatchEventError {
    #[error("LockErrorPoisoned")]
    LockErrorPoisoned,
    #[cfg(not(feature = "tokio"))]
    #[error("SendError")]
    SendError(#[from] std::sync::mpsc::SendError<Event>),
    #[cfg(feature = "tokio")]
    #[error("SendError")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Event>),
}

#[cfg(not(feature = "tokio"))]
pub type MpscSender<T> = std::sync::mpsc::Sender<T>;
#[cfg(not(feature = "tokio"))]
pub type MpscReceiver<T> = std::sync::mpsc::Receiver<T>;

#[cfg(feature = "tokio")]
pub type MpscSender<T> = tokio::sync::mpsc::UnboundedSender<T>;
#[cfg(feature = "tokio")]
pub type MpscReceiver<T> = tokio::sync::mpsc::UnboundedReceiver<T>;

pub(crate) fn push_batch(
    senders: Arc<RwLock<Watchers>>,
    batch: Batch,
) -> Result<(), WatchEventError> {
    let watchers = senders.read().map_err(|err| match err {
        _ => WatchEventError::LockErrorPoisoned,
    })?;

    let mut unused_watchers = vec![];
    for (watcher_request, event) in batch {
        for (id, sender) in watchers.find_senders(&watcher_request) {
            let l_sender = sender.lock().unwrap();
            if let Err(SendError(_)) = l_sender.send(event.clone()) {
                println!("Failed to send event to watcher {}", id);
                unused_watchers.push(id);
            }
        }
    }
    // Drop the lock before removing the watchers to avoid deadlock
    drop(watchers);

    // Remove unused watchers
    if unused_watchers.len() > 0 {
        let mut w = senders.write().map_err(|err| match err {
            _ => WatchEventError::LockErrorPoisoned,
        })?;
        for id in unused_watchers {
            w.remove_sender(id);
        }
    }

    Ok(())
}
