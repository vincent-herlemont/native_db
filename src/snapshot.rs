use crate::db_type::Result;
use crate::{Builder, Database, Models};
use redb::{ReadableDatabase, ReadableMultimapTable, ReadableTable};
use std::path::Path;

impl Database<'_> {
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
