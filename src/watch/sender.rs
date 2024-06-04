use crate::db_type::KeyEntry;
use crate::watch::filter::{KeyFilter, TableFilter};
use crate::watch::request::WatcherRequest;
use crate::watch::{Event, MpscSender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) struct Watchers(HashMap<u64, (TableFilter, Arc<Mutex<MpscSender<Event>>>)>);

impl Watchers {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn add_sender(
        &mut self,
        id: u64,
        table_filter: &TableFilter,
        event_sender: Arc<Mutex<MpscSender<Event>>>,
    ) {
        self.0.insert(id, (table_filter.clone(), event_sender));
    }

    pub(crate) fn remove_sender(&mut self, id: u64) -> bool {
        self.0.remove(&id).is_some()
    }

    pub(crate) fn find_senders(
        &self,
        request: &WatcherRequest,
    ) -> Vec<(u64, Arc<Mutex<MpscSender<Event>>>)> {
        let mut event_senders = Vec::new();
        for (id, (filter, event_sender)) in &self.0 {
            if filter.table_name == request.table_name {
                match &filter.key_filter {
                    KeyFilter::Primary(value) => {
                        if let Some(key) = &value {
                            if key == &request.primary_key {
                                event_senders.push((*id, Arc::clone(event_sender)));
                            }
                        } else {
                            event_senders.push((*id, Arc::clone(event_sender)));
                        }
                    }
                    KeyFilter::PrimaryStartWith(key_prefix) => {
                        if request
                            .primary_key
                            .as_slice()
                            .starts_with(key_prefix.as_slice())
                        {
                            event_senders.push((*id, Arc::clone(event_sender)));
                        }
                    }
                    KeyFilter::Secondary(key_def, key) => {
                        for (request_secondary_key_def, request_secondary_key) in
                            &request.secondary_keys_value
                        {
                            if key_def == request_secondary_key_def {
                                if let Some(filter_value) = &key {
                                    match request_secondary_key {
                                        KeyEntry::Default(value) => {
                                            if value == filter_value {
                                                event_senders.push((*id, Arc::clone(event_sender)));
                                            }
                                        }
                                        KeyEntry::Optional(value) => {
                                            if let Some(value) = value {
                                                if value == filter_value {
                                                    event_senders
                                                        .push((*id, Arc::clone(event_sender)));
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    event_senders.push((*id, Arc::clone(event_sender)));
                                }
                            }
                        }
                    }
                    KeyFilter::SecondaryStartWith(key_def, key_prefix) => {
                        for (request_secondary_key_def, request_secondary_key) in
                            &request.secondary_keys_value
                        {
                            match request_secondary_key {
                                KeyEntry::Default(value) => {
                                    if key_def == request_secondary_key_def {
                                        if value.as_slice().starts_with(key_prefix.as_slice()) {
                                            event_senders.push((*id, Arc::clone(event_sender)));
                                        }
                                    }
                                }
                                KeyEntry::Optional(value) => {
                                    if let Some(value) = value {
                                        if key_def == request_secondary_key_def {
                                            if value.as_slice().starts_with(key_prefix.as_slice()) {
                                                event_senders.push((*id, Arc::clone(event_sender)));
                                            }
                                        }
                                    }
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
