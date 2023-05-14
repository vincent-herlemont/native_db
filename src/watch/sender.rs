use crate::watch::filter::{KeyFilter, TableFilter};
use crate::watch::request::WatcherRequest;
use crate::watch::Event;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub(crate) struct Watchers(HashMap<u64, (TableFilter, Arc<Mutex<mpsc::Sender<Event>>>)>);

impl Watchers {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn add_sender(
        &mut self,
        id: u64,
        table_filter: &TableFilter,
        event_sender: Arc<Mutex<mpsc::Sender<Event>>>,
    ) {
        self.0.insert(id, (table_filter.clone(), event_sender));
    }

    pub(crate) fn remove_sender(&mut self, id: u64) {
        self.0.remove(&id);
    }

    pub(crate) fn find_senders(
        &self,
        request: &WatcherRequest,
    ) -> Vec<Arc<Mutex<mpsc::Sender<Event>>>> {
        let mut event_senders = Vec::new();
        for (_, (filter, event_sender)) in &self.0 {
            if filter.table_name == request.table_name {
                match &filter.key_filter {
                    KeyFilter::Primary(value) => {
                        if let Some(key) = &value {
                            if key == &request.primary_key {
                                event_senders.push(Arc::clone(event_sender));
                            }
                        } else {
                            event_senders.push(Arc::clone(event_sender));
                        }
                    }
                    KeyFilter::PrimaryStartWith(key_prefix) => {
                        if request.primary_key.starts_with(key_prefix) {
                            event_senders.push(Arc::clone(event_sender));
                        }
                    }
                    KeyFilter::Secondary(key_def, key) => {
                        for (request_secondary_key_def, request_secondary_key) in
                            &request.secondary_keys_value
                        {
                            if key_def == request_secondary_key_def.as_bytes() {
                                if let Some(value) = &key {
                                    if value == request_secondary_key {
                                        event_senders.push(Arc::clone(event_sender));
                                    }
                                } else {
                                    event_senders.push(Arc::clone(event_sender));
                                }
                            }
                        }
                    }
                    KeyFilter::SecondaryStartWith(key_def, key_prefix) => {
                        for (request_secondary_key_def, request_secondary_key) in
                            &request.secondary_keys_value
                        {
                            if key_def == request_secondary_key_def.as_bytes() {
                                if request_secondary_key.starts_with(key_prefix) {
                                    event_senders.push(Arc::clone(event_sender));
                                }
                            }
                        }
                    }
                }
            }
        }
        event_senders
    }
}
