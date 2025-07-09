use crate::db_type::Error;

/// Extension trait for converting errors during database upgrade migrations.
///
/// This trait provides convenient methods to convert errors from old database versions
/// into the current error type with proper context.
///
/// # Example
///
/// ```ignore
/// use native_db::upgrade::UpgradeResultExt;
///
/// // Inside an upgrade closure:
/// let old_db = V08xBuilder::new()
///     .open(&old_models, &db_path)
///     .upgrade_context("opening old database")?;
/// ```
pub trait UpgradeResultExt<T> {
    /// Converts an error into an `UpgradeMigration` error with the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - A description of what operation was being performed when the error occurred
    ///
    /// # Example
    ///
    /// ```ignore
    /// old_models.define::<V08xModel>()
    ///     .upgrade_context("defining old model")?;
    /// ```
    fn upgrade_context(self, context: &str) -> Result<T, Error>;

    /// Converts an error into an `UpgradeMigration` error with a context that includes
    /// information about a specific item being processed.
    ///
    /// # Arguments
    ///
    /// * `context` - A description of the operation
    /// * `item` - The item being processed (will be formatted using Debug)
    ///
    /// # Example
    ///
    /// ```ignore
    /// process_item(&item)
    ///     .upgrade_with_item("processing", &item)?;
    /// ```
    fn upgrade_with_item<I: std::fmt::Debug>(self, context: &str, item: &I) -> Result<T, Error>;
}

impl<T, E> UpgradeResultExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn upgrade_context(self, context: &str) -> Result<T, Error> {
        self.map_err(|e| Error::UpgradeMigration {
            context: context.to_string(),
            source: Box::new(e),
        })
    }

    fn upgrade_with_item<I: std::fmt::Debug>(self, context: &str, item: &I) -> Result<T, Error> {
        self.map_err(|e| Error::UpgradeMigration {
            context: format!("{context} item: {item:?}"),
            source: Box::new(e),
        })
    }
}

/// A prelude module that re-exports commonly used upgrade-related items.
///
/// # Example
///
/// ```ignore
/// use native_db::upgrade::prelude::*;
/// ```
pub mod prelude {
    pub use super::UpgradeResultExt;
}
