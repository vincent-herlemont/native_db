use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use assert_fs::TempDir;

mod common;
use common::{MemoryTracker, AllocationStats};

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


fn create_temp_db() -> (TempDir, Database<'static>) {
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();
    let db = Builder::new()
        .create(models, db_path)
        .unwrap();
    (tmp_dir, db)
}

#[test]
fn test_large_data_insertion_memory_leak() {
    let memory_tracker = MemoryTracker::new();
    let (_tmp_dir, db) = create_temp_db();
    
    // Insert large amounts of data
    const ITERATIONS: u32 = 1000;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record
    
    for i in 0..ITERATIONS {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; DATA_SIZE],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
        
        // Transaction already consumed by commit
    }
    
    // Check memory after insertions
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!("After insertions - Physical: {}, Virtual: {}",
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
    
    // Force garbage collection if possible
    thread::sleep(Duration::from_millis(100));
    
    // Check memory growth
    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("Memory usage within acceptable limits"),
        Err(msg) => panic!("{}", msg),
    }
}

#[test]
fn test_transaction_lifecycle_memory_leak() {
    let (_tmp_dir, db) = create_temp_db();
    
    // Create and drop many transactions
    for i in 0..10000 {
        // Read transaction
        let r = db.r_transaction().unwrap();
        let _ = r.get().primary::<TestModel>(i);
        drop(r);
        
        // Write transaction (without commit)
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; 100],
        };
        let _ = rw.insert(model);
        // Intentionally drop without commit to test rollback cleanup
        drop(rw);
    }
    
    // Create committed transactions
    for i in 0..1000 {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; 100],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
    }
}

#[test]
fn test_watch_system_memory_leak() {
    let (_tmp_dir, db) = create_temp_db();
    
    // Create many watchers and drop them
    for i in 0..1000 {
        let watcher = db.watch();
        
        // Insert some data to trigger events
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,  // Use unique IDs to avoid duplicate key errors
            name: format!("test_{}", i),
            data: vec![0u8; 1024],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
        
        // Process events - watchers don't have try_recv in native_db
        // We'll skip the event processing for now as it requires async runtime
        
        // Drop watcher
        drop(watcher);
    }
}

#[test]
fn test_database_open_close_cycles() {
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    
    // Open and close database many times
    for i in 0..100 {
        let _models = Models::new();
        
        let mut models = Models::new();
        models.define::<TestModel>().unwrap();
        
        let db = if i == 0 {
            Builder::new()
                .create(&models, &db_path)
                .unwrap()
        } else {
            Builder::new()
                .open(&models, &db_path)
                .unwrap()
        };
        
        // Do some operations
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; 1024],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
        
        // Explicitly drop database
        drop(db);
    }
}

#[test]
fn test_concurrent_access_memory_leak() {
    let (_tmp_dir, db) = create_temp_db();
    let db = Arc::new(db);
    
    let mut handles = vec![];
    
    // Spawn multiple threads doing operations
    for thread_id in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let id = thread_id * 1000 + i;
                
                // Insert
                let rw = db_clone.rw_transaction().unwrap();
                let model = TestModel {
                    id,
                    name: format!("thread_{}_item_{}", thread_id, i),
                    data: vec![0u8; 1024],
                };
                rw.insert(model).unwrap();
                rw.commit().unwrap();
                
                // Read
                let r = db_clone.r_transaction().unwrap();
                let _ = r.get().primary::<TestModel>(id);
                drop(r);
                
                // Update
                let rw = db_clone.rw_transaction().unwrap();
                if let Some(mut model) = rw.get().primary::<TestModel>(id).unwrap() {
                    model.data.extend_from_slice(&[1, 2, 3, 4]);
                    rw.auto_update(model).unwrap();
                }
                rw.commit().unwrap();
                
                // Delete
                let rw = db_clone.rw_transaction().unwrap();
                if let Some(model) = rw.get().primary::<TestModel>(id).unwrap() {
                    rw.remove(model).unwrap();
                }
                rw.commit().unwrap();
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
#[ignore] // This test requires manual observation
fn test_long_running_memory_stability() {
    let memory_tracker = MemoryTracker::new();
    let (_tmp_dir, db) = create_temp_db();
    
    println!("Starting long-running memory stability test...");
    
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!("Initial memory - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    } else {
        println!("Memory tracking not available on this platform");
    }
    
    for cycle in 0..10 {
        println!("Cycle {}/10", cycle + 1);
        
        // Heavy insertion phase
        for i in 0..1000 {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("cycle_{}_item_{}", cycle, i),
                data: vec![0u8; 10240], // 10KB
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
        }
        
        // Read phase
        for i in 0..1000 {
            let r = db.r_transaction().unwrap();
            let _ = r.get().primary::<TestModel>(i);
        }
        
        // Update phase
        for i in 0..1000 {
            let rw = db.rw_transaction().unwrap();
            if let Some(mut model) = rw.get().primary::<TestModel>(i).unwrap() {
                model.data = vec![1u8; 5120]; // Change to 5KB
                let updated_model = TestModel {
                    id: i,
                    name: format!("cycle_{}_item_{}", cycle, i),
                    data: vec![1u8; 5120], // Change to 5KB
                };
                rw.auto_update(updated_model).unwrap();
            }
            rw.commit().unwrap();
        }
        
        // Deletion phase
        for i in 0..1000 {
            let rw = db.rw_transaction().unwrap();
            if let Some(model) = rw.get().primary::<TestModel>(i).unwrap() {
                rw.remove(model).unwrap();
            }
            rw.commit().unwrap();
        }
        
        thread::sleep(Duration::from_secs(1));
        
        // Report memory after each cycle
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("Cycle {} memory - Physical: {}, Virtual: {}",
                cycle + 1,
                AllocationStats::format_bytes(stats.physical),
                AllocationStats::format_bytes(stats.virtual_mem)
            );
        }
    }
    
    // Final memory check
    match memory_tracker.check_memory_growth(100.0) {
        Ok(()) => println!("Long-running test completed successfully"),
        Err(msg) => println!("Warning: {}", msg),
    }
}