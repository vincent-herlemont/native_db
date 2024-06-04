use crate::db_type::{DatabaseKey, Error, Input, KeyOptions, Result, ToKey};
use crate::watch;
use crate::watch::{MpscReceiver, TableFilter};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

pub(crate) struct InternalWatch<'db> {
    pub(crate) watchers: &'db Arc<RwLock<watch::Watchers>>,
    pub(crate) watchers_counter_id: &'db AtomicU64,
}

impl InternalWatch<'_> {
    fn watch_generic(
        &self,
        table_filter: watch::TableFilter,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        #[cfg(not(feature = "tokio"))]
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        #[cfg(feature = "tokio")]
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
        let event_sender = Arc::new(Mutex::new(event_sender));
        let id = self.generate_watcher_id()?;
        let mut watchers = self.watchers.write().unwrap();
        watchers.add_sender(id, &table_filter, Arc::clone(&event_sender));
        Ok((event_receiver, id))
    }

    fn generate_watcher_id(&self) -> Result<u64> {
        let value = self
            .watchers_counter_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if value == u64::MAX {
            Err(Error::MaxWatcherReached.into())
        } else {
            Ok(value)
        }
    }

    pub(crate) fn watch_primary<T: Input>(
        &self,
        key: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let key = key.to_key();
        let table_filter =
            TableFilter::new_primary(table_name.unique_table_name.clone(), Some(key));
        self.watch_generic(table_filter)
    }

    pub(crate) fn watch_primary_all<T: Input>(&self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let table_filter = TableFilter::new_primary(table_name.unique_table_name.clone(), None);
        self.watch_generic(table_filter)
    }

    pub(crate) fn watch_primary_start_with<T: Input>(
        &self,
        start_with: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let start_with = start_with.to_key();
        let table_filter =
            TableFilter::new_primary_start_with(table_name.unique_table_name.clone(), start_with);
        self.watch_generic(table_filter)
    }

    pub(crate) fn watch_secondary<T: Input>(
        &self,
        key_def: &impl DatabaseKey<KeyOptions>,
        key: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let key = key.to_key();
        let table_filter =
            TableFilter::new_secondary(table_name.unique_table_name.clone(), key_def, Some(key));
        self.watch_generic(table_filter)
    }

    pub(crate) fn watch_secondary_all<T: Input>(
        &self,
        key_def: &impl DatabaseKey<KeyOptions>,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let table_filter =
            TableFilter::new_secondary(table_name.unique_table_name.clone(), key_def, None);
        self.watch_generic(table_filter)
    }

    pub(crate) fn watch_secondary_start_with<T: Input>(
        &self,
        key_def: &impl DatabaseKey<KeyOptions>,
        start_with: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::native_db_model().primary_key;
        let start_with = start_with.to_key();
        let table_filter = TableFilter::new_secondary_start_with(
            table_name.unique_table_name.clone(),
            key_def,
            start_with,
        );
        self.watch_generic(table_filter)
    }
}
