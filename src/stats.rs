#[derive(Debug)]
pub struct Stats {
    pub stats_tables: Vec<StatsTable>,
}

#[derive(Debug)]
pub struct StatsTable {
    pub name: String,
    pub num_raw: usize,
}
