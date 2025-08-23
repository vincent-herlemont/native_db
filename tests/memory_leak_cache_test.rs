use assert_fs::TempDir;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

mod common;
use common::{AllocationStats, MemoryTracker};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct TestModel {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
    data: Vec<u8>,
}

fn test_with_cache_size(cache_size: Option<usize>, test_name: &str) {
    println!("\n=== {} ===", test_name);
    let memory_tracker = MemoryTracker::new();

    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");

    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();

    let mut builder = Builder::new();
    if let Some(size) = cache_size {
        builder.set_cache_size(size);
        println!(
            "Cache size set to: {} bytes ({:.2} MB)",
            size,
            size as f64 / 1024.0 / 1024.0
        );
    } else {
        println!("Using default cache size");
    }

    let db = builder.create(models, db_path).unwrap();

    // Print memory after DB creation
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "After DB creation - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    const ITERATIONS: u32 = 1000;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record

    // Insert data
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; DATA_SIZE],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
    }

    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "After insertions - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Delete all data
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();
    }

    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "After deletions - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Try explicit compaction if available
    // Note: redb might not expose direct compaction API

    // Force drop
    drop(db);
    thread::sleep(Duration::from_millis(100));

    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "After drop - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("Memory usage within acceptable limits"),
        Err(msg) => println!("WARNING: {}", msg),
    }
}

#[test]
fn test_memory_with_different_cache_sizes() {
    // Test with minimal cache
    test_with_cache_size(Some(1024 * 1024), "Test with 1MB cache");

    // Give time for memory to be released
    thread::sleep(Duration::from_secs(1));

    // Test with moderate cache
    test_with_cache_size(Some(10 * 1024 * 1024), "Test with 10MB cache");

    thread::sleep(Duration::from_secs(1));

    // Test with default (no cache size set)
    test_with_cache_size(None, "Test with default cache");
}

#[test]
fn test_compaction_behavior() {
    println!("\n=== Testing Compaction Behavior ===");
    let memory_tracker = MemoryTracker::new();

    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");

    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();

    // Use small cache to force more disk operations
    let mut builder = Builder::new();
    builder.set_cache_size(1024 * 1024); // 1MB

    let db = builder.create(models, &db_path).unwrap();

    // Insert and delete in alternating pattern
    for cycle in 0..5 {
        println!("\n--- Cycle {} ---", cycle + 1);

        // Insert 200 records
        for i in 0..200 {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: cycle * 1000 + i,
                name: format!("cycle_{}_test_{}", cycle, i),
                data: vec![0u8; 1024 * 10], // 10KB
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
        }

        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!(
                "After inserts - Physical: {}",
                AllocationStats::format_bytes(stats.physical)
            );
        }

        // Delete half of them
        for i in 0..100 {
            let rw = db.rw_transaction().unwrap();
            if let Some(model) = rw.get().primary::<TestModel>(cycle * 1000 + i).unwrap() {
                rw.remove(model).unwrap();
            }
            rw.commit().unwrap();
        }

        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!(
                "After partial delete - Physical: {}",
                AllocationStats::format_bytes(stats.physical)
            );
        }
    }

    // Close and reopen to test if compaction happens
    drop(db);

    println!("\n--- Reopening database ---");
    let db = builder.open(models, &db_path).unwrap();

    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "After reopen - Physical: {}",
            AllocationStats::format_bytes(stats.physical)
        );
    }

    // Count remaining records
    let r = db.r_transaction().unwrap();
    let count = r.len().primary::<TestModel>().unwrap();
    println!("Remaining records: {}", count);

    drop(r);
    drop(db);

    match memory_tracker.check_memory_growth(100.0) {
        Ok(()) => println!("\nMemory usage within acceptable limits"),
        Err(msg) => println!("\nWARNING: {}", msg),
    }
}
