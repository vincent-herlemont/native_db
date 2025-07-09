use native_db::db_type::Error;
use native_db::upgrade::UpgradeResultExt;
use std::io;

#[test]
fn test_upgrade_context_conversion() {
    // Test basic error conversion
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let result: Result<(), io::Error> = Err(io_error);

    let converted = result.upgrade_context("opening database");
    assert!(converted.is_err());

    match converted.unwrap_err() {
        Error::UpgradeMigration { context, source } => {
            assert_eq!(context, "opening database");
            assert_eq!(source.to_string(), "file not found");
        }
        _ => panic!("Expected UpgradeMigration error"),
    }
}

#[test]
fn test_upgrade_with_item_conversion() {
    #[derive(Debug)]
    struct TestItem {
        id: u32,
        name: String,
    }

    let item = TestItem {
        id: 42,
        name: "test".to_string(),
    };

    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let result: Result<(), io::Error> = Err(io_error);

    let converted = result.upgrade_with_item("processing", &item);
    assert!(converted.is_err());

    match converted.unwrap_err() {
        Error::UpgradeMigration { context, source } => {
            assert!(context.contains("processing item:"));
            assert!(context.contains("TestItem"));
            assert!(context.contains("id: 42"));
            assert!(context.contains("name: \"test\""));
            assert_eq!(source.to_string(), "access denied");
        }
        _ => panic!("Expected UpgradeMigration error"),
    }
}

#[test]
fn test_upgrade_context_preserves_ok_values() {
    let result: Result<i32, io::Error> = Ok(42);
    let converted = result.upgrade_context("some operation");
    assert!(converted.is_ok());
    assert_eq!(converted.unwrap(), 42);
}
