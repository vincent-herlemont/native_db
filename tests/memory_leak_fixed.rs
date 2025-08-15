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

#[test]
fn test_large_data_insertion_memory_leak_fixed() {
    println!("\n=== Fixed Memory Leak Test with 10MB Cache ===");

    // Create a fresh memory tracker for this test
    let memory_tracker = MemoryTracker::new();

    println!("Initial memory state:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();

    // Create database with 10MB cache
    let mut builder = Builder::new();
    builder.set_cache_size(10 * 1024 * 1024); // 10MB cache
    let db = builder.create(models, db_path).unwrap();

    println!("\nAfter DB creation (10MB cache):");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Insert large amounts of data
    const ITERATIONS: u32 = 1000;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record

    println!("\nInserting {} records...", ITERATIONS);
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

    println!("\nAfter insertions:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Delete all data
    println!("\nDeleting all records...");
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();
    }

    println!("\nAfter deletions:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Force cleanup
    drop(db);
    drop(tmp_dir);
    thread::sleep(Duration::from_millis(100));

    println!("\nAfter cleanup:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Check memory growth
    println!("\nMemory growth analysis:");
    match memory_tracker.get_memory_growth() {
        Some((growth_bytes, growth_percentage)) => {
            println!(
                "  Growth: {} bytes ({:.2}%)",
                growth_bytes, growth_percentage
            );

            if growth_percentage > 50.0 {
                panic!(
                    "Memory grew by {:.2}% ({} bytes), exceeding threshold of 50.00%",
                    growth_percentage, growth_bytes
                );
            } else {
                println!("  ✅ Memory usage within acceptable limits");
            }
        }
        None => println!("  Could not measure memory growth"),
    }
}
