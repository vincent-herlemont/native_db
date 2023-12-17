#[derive(Debug)]
pub struct Stats {
    pub primary_tables: Vec<StatsTable>,
    pub secondary_tables: Vec<StatsTable>,
}

#[derive(Debug)]
pub struct StatsTable {
    pub name: String,
    pub n_entries: Option<u64>,
}
