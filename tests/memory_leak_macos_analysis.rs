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
fn analyze_macos_memory_behavior() {
    println!("\n=== macOS Memory Behavior Analysis ===");
    println!("Testing if memory is actually leaked or just not released to OS");

    // First pass - establish baseline
    {
        println!("\n--- First Pass (Baseline) ---");
        let memory_tracker = MemoryTracker::new();

        let tmp_dir = TempDir::new().unwrap();
        let db_path = tmp_dir.path().join("test.db");
        let models = Box::leak(Box::new(Models::new()));
        models.define::<TestModel>().unwrap();

        let mut builder = Builder::new();
        builder.set_cache_size(5 * 1024 * 1024); // 5MB cache
        let db = builder.create(models, db_path).unwrap();

        // Insert and delete 500 records
        for i in 0..500u32 {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("test_{}", i),
                data: vec![0u8; 10 * 1024], // 10KB
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
        }

        for i in 0..500u32 {
            let rw = db.rw_transaction().unwrap();
            let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
            rw.remove(model).unwrap();
            rw.commit().unwrap();
        }

        drop(db);
        drop(tmp_dir);

        if let Some((growth_bytes, growth_percentage)) = memory_tracker.get_memory_growth() {
            println!(
                "First pass growth: {} bytes ({:.2}%)",
                growth_bytes, growth_percentage
            );
        }
    }

    // Wait a bit
    thread::sleep(Duration::from_secs(2));

    // Second pass - see if memory is reused
    {
        println!("\n--- Second Pass (Memory Reuse Test) ---");
        let memory_tracker = MemoryTracker::new();

        let tmp_dir = TempDir::new().unwrap();
        let db_path = tmp_dir.path().join("test2.db");
        let models = Box::leak(Box::new(Models::new()));
        models.define::<TestModel>().unwrap();

        let mut builder = Builder::new();
        builder.set_cache_size(5 * 1024 * 1024); // 5MB cache
        let db = builder.create(models, db_path).unwrap();

        // Insert and delete 500 records again
        for i in 0..500u32 {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("test_{}", i),
                data: vec![0u8; 10 * 1024], // 10KB
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
        }

        for i in 0..500u32 {
            let rw = db.rw_transaction().unwrap();
            let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
            rw.remove(model).unwrap();
            rw.commit().unwrap();
        }

        drop(db);
        drop(tmp_dir);

        if let Some((growth_bytes, growth_percentage)) = memory_tracker.get_memory_growth() {
            println!(
                "Second pass growth: {} bytes ({:.2}%)",
                growth_bytes, growth_percentage
            );

            // If second pass has minimal growth, memory is being reused
            if growth_percentage < 50.0 {
                println!("✅ Memory is being reused - not a true leak");
            } else {
                println!("❌ Memory continues to grow - possible leak");
            }
        }
    }
}

#[test]
fn test_very_small_cache() {
    println!("\n=== Test with Very Small Cache (1MB) ===");

    let memory_tracker = MemoryTracker::new();

    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();

    let mut builder = Builder::new();
    builder.set_cache_size(1 * 1024 * 1024); // Only 1MB cache
    let db = builder.create(models, db_path).unwrap();

    // Insert only 100 records to avoid overwhelming small cache
    const ITERATIONS: u32 = 100;
    const DATA_SIZE: usize = 1024; // Only 1KB per record

    println!("Inserting {} small records with 1MB cache...", ITERATIONS);
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

    println!("Deleting all records...");
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();
    }

    drop(db);
    drop(tmp_dir);

    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("✅ Memory usage within acceptable limits"),
        Err(msg) => println!("❌ {}", msg),
    }
}

#[test]
fn test_memory_with_forced_cleanup() {
    println!("\n=== Test with Forced Cleanup ===");

    let memory_tracker = MemoryTracker::new();

    {
        let tmp_dir = TempDir::new().unwrap();
        let db_path = tmp_dir.path().join("test.db");
        let models = Box::leak(Box::new(Models::new()));
        models.define::<TestModel>().unwrap();

        let mut builder = Builder::new();
        builder.set_cache_size(2 * 1024 * 1024); // 2MB cache
        let db = builder.create(models, db_path).unwrap();

        // Insert in small batches with cleanup between
        for batch in 0..10u32 {
            // Insert 100 records
            for i in 0..100u32 {
                let id = batch * 100 + i;
                let rw = db.rw_transaction().unwrap();
                let model = TestModel {
                    id,
                    name: format!("test_{}", id),
                    data: vec![0u8; 5 * 1024], // 5KB
                };
                rw.insert(model).unwrap();
                rw.commit().unwrap();
            }

            // Delete them immediately
            for i in 0..100u32 {
                let id = batch * 100 + i;
                let rw = db.rw_transaction().unwrap();
                let model = rw.get().primary::<TestModel>(id).unwrap().unwrap();
                rw.remove(model).unwrap();
                rw.commit().unwrap();
            }

            // Small pause between batches
            thread::sleep(Duration::from_millis(10));
        }

        // Explicitly drop everything
        drop(db);
        drop(tmp_dir);
    } // Extra scope to ensure everything is dropped

    // Force a pause for OS cleanup
    thread::sleep(Duration::from_millis(500));

    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("✅ Memory usage within acceptable limits"),
        Err(msg) => println!("❌ {}", msg),
    }
}
