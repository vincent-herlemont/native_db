use crate::db_type::Result;
use crate::{Builder, Database, Models};
use redb::{ReadableDatabase, ReadableMultimapTable, ReadableTable};
use std::path::Path;

impl Database<'_> {
    /// Creates a point-in-time copy (snapshot) of the database to the specified file path.
    ///
    /// This method is useful for creating backups or for working with a stable version of the
    /// database while new transactions are being committed to the original database.
    ///
    /// # Arguments
    ///
    /// * `models`: A reference to the `Models` instance that defines the database schema.
    /// * `path`: The `Path` where the snapshot file will be created.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Database` instance representing the snapshot,
    /// or an error if the snapshot creation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    /// use std::path::Path;
    /// use tempfile::NamedTempFile;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
    /// #[native_model(id = 1, version = 1)]
    /// #[native_db]
    /// struct Item {
    ///     #[primary_key]
    ///     id: u32,
    ///     name: String,
    /// }
    ///
    /// fn main() -> Result<(), native_db::Error> {
    ///     let mut models = Models::new();
    ///     models.define::<Item>()?;
    ///
    ///     // 1. Create an in-memory database
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///
    ///     // 2. Insert an item
    ///     let rw = db.rw_transaction()?;
    ///     rw.insert(Item {
    ///         id: 1,
    ///         name: "test_item".to_string(),
    ///     })?;
    ///     rw.commit()?;
    ///
    ///     // 3. Create a temporary file path for the snapshot
    ///     let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    ///     let snapshot_path = temp_file.path();
    ///
    ///     // 4. Call the snapshot method to save the database to the file
    ///     let db_snapshot_creator = db.snapshot(&models, snapshot_path)?;
    ///     // Note: db_snapshot_creator is a new Database instance.
    ///     // If you only need to ensure the snapshot is written to disk and then open it later,
    ///     // you might not need to use `db_snapshot_creator` immediately,
    ///     // or you can let it go out of scope if the file handle within it is not critical.
    ///     // For this example, we proceed to open it as a new instance.
    ///
    ///     // 5. Open the created snapshot file as a new Database instance
    ///     let snapshot_db = Builder::new().open(&models, snapshot_path)?;
    ///
    ///     // 6. Read the inserted Item from the snapshot to verify its contents
    ///     let r = snapshot_db.r_transaction()?;
    ///     let retrieved_item: Item = r.get().primary(1u32)?.unwrap();
    ///     assert_eq!(retrieved_item.name, "test_item");
    ///     println!("Retrieved item: {:?}", retrieved_item);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn snapshot<'a>(&self, models: &'a Models, path: &Path) -> Result<Database<'a>> {
        let new_db = Builder::new().create(models, path)?;
        let r = self.instance.redb_database()?.begin_read()?;
        let w = new_db.instance.redb_database()?.begin_write()?;
        {
            // Copy primary tables
            for primary_table_definition in self.primary_table_definitions.values() {
                let table = r.open_table(primary_table_definition.redb)?;
                let mut new_table = w.open_table(primary_table_definition.redb)?;
                for result in table.iter()? {
                    let (key, value) = result?;
                    new_table.insert(key.value(), value.value())?;
                }

                // Copy secondary tables
                for secondary_table_definition in primary_table_definition.secondary_tables.values()
                {
                    let table = r.open_multimap_table(secondary_table_definition.redb)?;
                    let mut new_table = w.open_multimap_table(secondary_table_definition.redb)?;
                    for result in table.iter()? {
                        let (secondary_key, primary_keys) = result?;
                        for primary_key in primary_keys {
                            let primary_key = primary_key?;
                            new_table.insert(secondary_key.value(), primary_key.value())?;
                        }
                    }
                }
            }
        }
        w.commit()?;
        Ok(new_db)
    }
}
