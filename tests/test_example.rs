#[cfg(test)]
mod tests {
    use native_db::DatabaseBuilder;

    #[test]
    fn test_example() {
        // This is just a simple example test
        let db = DatabaseBuilder::new()
            .memory()
            .build()
            .expect("Failed to create database");

        assert!(db.is_memory());
    }
}
