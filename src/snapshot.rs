use crate::db_type::Result;
use crate::{Database, DatabaseBuilder};
use redb::ReadableTable;
use std::path::Path;

impl Database<'_> {
    pub fn snapshot<'a>(&self, builder: &'a DatabaseBuilder, path: &Path) -> Result<Database<'a>> {
        // TODO: builder must have well defined models
        let new_db = builder.create(path)?;
        let r = self.instance.redb_database()?.begin_read()?;
        let w = new_db.instance.redb_database()?.begin_write()?;
        {
            // Copy primary tables
            for (_, primary_table_definition) in &self.primary_table_definitions {
                let table = r.open_table(primary_table_definition.redb)?;
                let mut new_table = w.open_table(primary_table_definition.redb)?;
                for result in table.iter()? {
                    let (key, value) = result?;
                    new_table.insert(key.value(), value.value())?;
                }

                // Copy secondary tables
                for (_, secondary_table_definition) in &primary_table_definition.secondary_tables {
                    let table = r.open_table(secondary_table_definition.redb)?;
                    let mut new_table = w.open_table(secondary_table_definition.redb)?;
                    for result in table.iter()? {
                        let (key, value) = result?;
                        new_table.insert(key.value(), value.value())?;
                    }
                }
            }
        }
        w.commit()?;
        Ok(new_db)
    }
}
