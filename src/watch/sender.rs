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
                        if let Some(value) = &value {
                            if value == &request.primary_key_value {
                                event_senders.push(Arc::clone(event_sender));
                            }
                        } else {
                            event_senders.push(Arc::clone(event_sender));
                        }
                    }
                    KeyFilter::PrimaryStartWith(start_with) => {
                        if request.primary_key_value.starts_with(start_with) {
                            event_senders.push(Arc::clone(event_sender));
                        }
                    }
                    KeyFilter::Secondary(key, value) => {
                        for (request_secondary_key, request_secondary_key_value) in
                            &request.secondary_keys_value
                        {
                            if key == request_secondary_key.as_bytes() {
                                if let Some(value) = &value {
                                    if value == request_secondary_key_value {
                                        event_senders.push(Arc::clone(event_sender));
                                    }
                                } else {
                                    event_senders.push(Arc::clone(event_sender));
                                }
                            }
                        }
                    }
                    KeyFilter::SecondaryStartWith(key, start_with) => {
                        for (request_secondary_key, request_secondary_key_value) in
                            &request.secondary_keys_value
                        {
                            if key == request_secondary_key.as_bytes() {
                                if request_secondary_key_value.starts_with(start_with) {
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
