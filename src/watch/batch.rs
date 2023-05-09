use crate::watch::{Event, WatcherRequest};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Batch(Vec<(WatcherRequest, Event)>);

impl Batch {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn add(&mut self, watcher_request: WatcherRequest, event: Event) {
        self.0.push((watcher_request, event));
    }
}

impl Iterator for Batch {
    type Item = (WatcherRequest, Event);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl Debug for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (watcher_request, event) in &self.0 {
            write!(
                f,
                "({}, {:?}), ",
                String::from_utf8_lossy(&watcher_request.primary_key_value),
                event
            )?;
        }
        write!(f, "]")
    }
}
