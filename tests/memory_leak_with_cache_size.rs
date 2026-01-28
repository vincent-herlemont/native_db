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

fn create_temp_db_with_cache(cache_size: usize) -> (TempDir, Database<'static>) {
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();

    let mut builder = Builder::new();
    builder.set_cache_size(cache_size);

    let db = builder.create(models, db_path).unwrap();
    (tmp_dir, db)
}

#[test]
fn test_large_data_insertion_with_small_cache() {
    println!("\n=== Test with 5MB Cache Size ===");
    let memory_tracker = MemoryTracker::new();

    // Use 5MB cache instead of default 1GB
    let (_tmp_dir, db) = create_temp_db_with_cache(5 * 1024 * 1024);

    println!("Initial memory:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Insert large amounts of data (same as original test)
    const ITERATIONS: u32 = 1000;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record

    println!(
        "\nInserting {} records ({} KB each)...",
        ITERATIONS,
        DATA_SIZE / 1024
    );
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
    println!("\nDeleting all {} records...", ITERATIONS);
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

    // Force garbage collection if possible
    thread::sleep(Duration::from_millis(100));

    println!("\nAfter sleep:");
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!(
            "  Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }

    // Check memory growth
    println!("\nMemory growth check (threshold: 50%):");
    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("✅ Memory usage within acceptable limits"),
        Err(msg) => println!("❌ {}", msg),
    }
}

#[test]
fn test_comparison_different_cache_sizes() {
    println!("\n=== Comparing Different Cache Sizes ===");

    // Test configurations
    let cache_configs = vec![
        (1 * 1024 * 1024, "1MB"),
        (5 * 1024 * 1024, "5MB"),
        (10 * 1024 * 1024, "10MB"),
        (50 * 1024 * 1024, "50MB"),
    ];

    const ITERATIONS: u32 = 500;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record

    for (cache_size, cache_name) in cache_configs {
        println!("\n--- Testing with {} cache ---", cache_name);
        let memory_tracker = MemoryTracker::new();

        let (_tmp_dir, db) = create_temp_db_with_cache(cache_size);

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

        let after_insert = MemoryTracker::get_current_memory();

        // Delete all data
        for i in 0..ITERATIONS {
            let rw = db.rw_transaction().unwrap();
            let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
            rw.remove(model).unwrap();
            rw.commit().unwrap();
        }

        let after_delete = MemoryTracker::get_current_memory();

        // Drop database
        drop(db);
        thread::sleep(Duration::from_millis(100));

        let after_drop = MemoryTracker::get_current_memory();

        // Print results
        if let (Some(insert), Some(delete), Some(drop_stats)) =
            (after_insert, after_delete, after_drop)
        {
            println!(
                "  After insert: {}",
                AllocationStats::format_bytes(insert.physical)
            );
            println!(
                "  After delete: {}",
                AllocationStats::format_bytes(delete.physical)
            );
            println!(
                "  After drop:   {}",
                AllocationStats::format_bytes(drop_stats.physical)
            );
        }

        match memory_tracker.check_memory_growth(50.0) {
            Ok(()) => println!("  Result: ✅ Within 50% growth"),
            Err(msg) => {
                if let Some((growth_bytes, growth_percentage)) = memory_tracker.get_memory_growth()
                {
                    println!(
                        "  Result: ❌ Growth: {:.1}% ({} bytes)",
                        growth_percentage, growth_bytes
                    );
                }
            }
        }

        // Clean up between tests
        thread::sleep(Duration::from_millis(500));
    }
}

#[test]
fn test_minimal_cache_memory_behavior() {
    println!("\n=== Test with Minimal Cache (256KB) ===");
    let memory_tracker = MemoryTracker::new();

    // Use very small cache to force disk operations
    let (_tmp_dir, db) = create_temp_db_with_cache(256 * 1024); // 256KB

    const ITERATIONS: u32 = 1000;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record

    println!("Inserting {} records with minimal cache...", ITERATIONS);

    // Track memory during insertion
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; DATA_SIZE],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();

        if i % 250 == 249 {
            if let Some(stats) = MemoryTracker::get_current_memory() {
                println!(
                    "  After {} records: {}",
                    i + 1,
                    AllocationStats::format_bytes(stats.physical)
                );
            }
        }
    }

    println!("\nDeleting all records with minimal cache...");

    // Track memory during deletion
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();

        if i % 250 == 249 {
            if let Some(stats) = MemoryTracker::get_current_memory() {
                println!(
                    "  After {} deletions: {}",
                    i + 1,
                    AllocationStats::format_bytes(stats.physical)
                );
            }
        }
    }

    drop(db);
    thread::sleep(Duration::from_millis(100));

    println!("\nFinal memory check:");
    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("✅ Memory usage within acceptable limits"),
        Err(msg) => println!("❌ {}", msg),
    }
}
