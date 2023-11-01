/// Schema of the Item. Returned by the [`<your_item>::struct_db_schema()`](crate::SDBItem::struct_db_schema) method.
#[derive(Clone, Debug)]
pub struct Schema {
    pub table_name: &'static str,
    pub primary_key: &'static str,
    pub secondary_tables_name: std::collections::HashSet<&'static str>,
}
