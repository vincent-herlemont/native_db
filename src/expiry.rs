//! Expiry (time to live) of the values of a model.
//!
//! A model declares an expiry with the `expire_after` attribute on the field that holds the
//! moment to measure from, expressed as a number of seconds (for example a Unix timestamp).
//! The attribute argument is the lifetime of a value in seconds.
//!
//! ```rust
//! use native_db::*;
//! use native_db::native_model::{native_model, Model};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! #[native_model(id = 1, version = 1)]
//! #[native_db]
//! struct Session {
//!     #[primary_key]
//!     id: u32,
//!     #[expire_after(60)]
//!     created_at: u64,
//! }
//!
//! fn main() -> Result<(), db_type::Error> {
//!     let mut models = Models::new();
//!     models.define::<Session>()?;
//!     let db = Builder::new().create_in_memory(&models)?;
//!
//!     let rw = db.rw_transaction()?;
//!     rw.insert(Session { id: 1, created_at: 1000 })?;
//!     rw.commit()?;
//!
//!     let r = db.r_transaction_observed_at(1059)?;
//!     assert!(r.get().primary::<Session>(1u32)?.is_some());
//!
//!     let r = db.r_transaction_observed_at(1060)?;
//!     assert!(r.get().primary::<Session>(1u32)?.is_none());
//!     assert_eq!(r.len().primary::<Session>()?, 0);
//!     Ok(())
//! }
//! ```
//!
//! The current time is never read from the system clock. A value is only hidden when it is read
//! through a transaction opened with [`Database::r_transaction_observed_at`](crate::Database::r_transaction_observed_at)
//! or [`Database::rw_transaction_observed_at`](crate::Database::rw_transaction_observed_at),
//! which take the moment to compare against. Transactions opened with
//! [`Database::r_transaction`](crate::Database::r_transaction) or
//! [`Database::rw_transaction`](crate::Database::rw_transaction) never hide a value.
//!
//! An expired value is hidden from `get`, `scan` and `len`. It stays stored until it is removed,
//! and `remove` still works on it.

use std::time::Duration;

/// A fixed lifetime after which a value expires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expiry {
    lifetime: Duration,
}

impl Expiry {
    /// Creates an expiry that applies once `lifetime` has elapsed. Only whole seconds are used.
    pub fn after(lifetime: Duration) -> Self {
        Self { lifetime }
    }

    /// Returns `true` when the time elapsed between `recorded_at` and `now` (both in seconds)
    /// reaches the lifetime. A value recorded after `now` is never expired.
    pub fn is_expired(&self, recorded_at: u64, now: u64) -> bool {
        match now.checked_sub(recorded_at) {
            Some(elapsed) => elapsed >= self.lifetime.as_secs(),
            None => false,
        }
    }

    /// The opposite of [`is_expired`](Self::is_expired).
    pub fn is_visible(&self, recorded_at: u64, now: u64) -> bool {
        !self.is_expired(recorded_at, now)
    }
}

/// The expiry declared by a model, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpiryPolicy {
    pub(crate) expiry: Option<Expiry>,
}

impl ExpiryPolicy {
    /// A policy under which values never expire.
    pub fn none() -> Self {
        Self { expiry: None }
    }

    /// A policy under which values expire according to `expiry`.
    pub fn of(expiry: Expiry) -> Self {
        Self {
            expiry: Some(expiry),
        }
    }

    /// Returns `false` only when the policy has an expiry and the value is expired at `now`.
    pub fn is_visible(&self, recorded_at: u64, now: u64) -> bool {
        match self.expiry {
            Some(expiry) => expiry.is_visible(recorded_at, now),
            None => true,
        }
    }
}

pub(crate) fn visible_at<T: crate::db_type::ToInput>(
    model: &crate::Model,
    value: &T,
    observed_at: Option<u64>,
) -> bool {
    let expiry = match model.expiry.expiry {
        Some(expiry) => expiry,
        None => return true,
    };

    let now = match observed_at {
        Some(now) => now,
        None => return true,
    };

    match value.native_db_recorded_at() {
        Some(recorded_at) => expiry.is_visible(recorded_at, now),
        None => true,
    }
}
