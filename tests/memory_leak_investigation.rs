use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use assert_fs::TempDir;
use std::thread;
use std::time::Duration;

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

#[test]
fn test_redb_default_cache_behavior() {
    println!("\n=== Testing redb Default Cache Behavior ===");
    println!("According to redb docs, default cache is 1GB");
    
    let memory_tracker = MemoryTracker::new();
    
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();
    
    // Test 1: Default cache (should be 1GB according to redb)
    {
        println!("\n--- Test 1: Default cache ---");
        let db = Builder::new().create(models, &db_path).unwrap();
        
        // Insert 100MB of data
        const RECORDS: u32 = 10_000;
        const DATA_SIZE: usize = 10 * 1024; // 10KB per record = 100MB total
        
        for i in 0..RECORDS {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("test_{}", i),
                data: vec![0u8; DATA_SIZE],
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
            
            if i % 1000 == 0 {
                if let Some(stats) = MemoryTracker::get_current_memory() {
                    println!("After {} records - Physical: {}", 
                        i, AllocationStats::format_bytes(stats.physical));
                }
            }
        }
        
        drop(db);
        
        // Remove database file
        std::fs::remove_file(&db_path).unwrap();
    }
    
    thread::sleep(Duration::from_secs(1));
    
    // Test 2: Small cache (1MB)
    {
        println!("\n--- Test 2: 1MB cache ---");
        let mut builder = Builder::new();
        builder.set_cache_size(1024 * 1024); // 1MB
        let db = builder.create(models, &db_path).unwrap();
        
        // Insert same amount of data
        const RECORDS: u32 = 10_000;
        const DATA_SIZE: usize = 10 * 1024; // 10KB per record = 100MB total
        
        for i in 0..RECORDS {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("test_{}", i),
                data: vec![0u8; DATA_SIZE],
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
            
            if i % 1000 == 0 {
                if let Some(stats) = MemoryTracker::get_current_memory() {
                    println!("After {} records - Physical: {}", 
                        i, AllocationStats::format_bytes(stats.physical));
                }
            }
        }
        
        drop(db);
    }
    
    match memory_tracker.check_memory_growth(200.0) {
        Ok(()) => println!("\nMemory usage within acceptable limits"),
        Err(msg) => println!("\nWARNING: {}", msg),
    }
}

#[test]
fn test_memory_release_patterns() {
    println!("\n=== Testing Memory Release Patterns ===");
    
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();
    
    // Use very small cache to force disk operations
    let mut builder = Builder::new();
    builder.set_cache_size(512 * 1024); // 512KB
    
    // Test pattern: Insert, delete, check memory after various operations
    {
        let memory_tracker = MemoryTracker::new();
        let db = builder.create(models, &db_path).unwrap();
        
        println!("\n1. Inserting 500 records...");
        for i in 0..500 {
            let rw = db.rw_transaction().unwrap();
            let model = TestModel {
                id: i,
                name: format!("test_{}", i),
                data: vec![0u8; 10 * 1024], // 10KB
            };
            rw.insert(model).unwrap();
            rw.commit().unwrap();
        }
        
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("After inserts - Physical: {}", AllocationStats::format_bytes(stats.physical));
        }
        
        println!("\n2. Deleting all records...");
        for i in 0..500 {
            let rw = db.rw_transaction().unwrap();
            if let Some(model) = rw.get().primary::<TestModel>(i).unwrap() {
                rw.remove(model).unwrap();
            }
            rw.commit().unwrap();
        }
        
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("After deletes - Physical: {}", AllocationStats::format_bytes(stats.physical));
        }
        
        println!("\n3. Creating read transaction to check empty DB...");
        let r = db.r_transaction().unwrap();
        let count = r.len().primary::<TestModel>().unwrap();
        println!("Records in DB: {}", count);
        drop(r);
        
        println!("\n4. Dropping database...");
        drop(db);
        
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("After drop - Physical: {}", AllocationStats::format_bytes(stats.physical));
        }
        
        match memory_tracker.check_memory_growth(50.0) {
            Ok(()) => println!("Memory usage within acceptable limits"),
            Err(msg) => println!("WARNING: {}", msg),
        }
    }
    
    thread::sleep(Duration::from_secs(1));
    
    // Test reopen
    {
        println!("\n5. Reopening database...");
        let memory_tracker = MemoryTracker::new();
        
        let db = builder.open(models, &db_path).unwrap();
        
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("After reopen - Physical: {}", AllocationStats::format_bytes(stats.physical));
        }
        
        let r = db.r_transaction().unwrap();
        let count = r.len().primary::<TestModel>().unwrap();
        println!("Records after reopen: {}", count);
        drop(r);
        
        drop(db);
        
        match memory_tracker.check_memory_growth(50.0) {
            Ok(()) => println!("Memory after reopen within acceptable limits"),
            Err(msg) => println!("WARNING after reopen: {}", msg),
        }
    }
}

#[test]
fn test_file_size_after_deletions() {
    println!("\n=== Testing File Size After Deletions ===");
    
    let tmp_dir = TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    
    let models = Box::leak(Box::new(Models::new()));
    models.define::<TestModel>().unwrap();
    
    let mut builder = Builder::new();
    builder.set_cache_size(1024 * 1024); // 1MB
    
    let db = builder.create(models, &db_path).unwrap();
    
    // Insert data
    println!("Inserting 1000 records...");
    for i in 0..1000 {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; 10 * 1024], // 10KB
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
    }
    
    // Check file size
    let metadata = std::fs::metadata(&db_path).unwrap();
    let size_after_insert = metadata.len();
    println!("File size after inserts: {} bytes ({:.2} MB)", 
        size_after_insert, size_after_insert as f64 / 1024.0 / 1024.0);
    
    // Delete all records
    println!("Deleting all records...");
    for i in 0..1000 {
        let rw = db.rw_transaction().unwrap();
        if let Some(model) = rw.get().primary::<TestModel>(i).unwrap() {
            rw.remove(model).unwrap();
        }
        rw.commit().unwrap();
    }
    
    drop(db);
    
    // Check file size after deletions
    let metadata = std::fs::metadata(&db_path).unwrap();
    let size_after_delete = metadata.len();
    println!("File size after deletes: {} bytes ({:.2} MB)", 
        size_after_delete, size_after_delete as f64 / 1024.0 / 1024.0);
    
    let size_ratio = size_after_delete as f64 / size_after_insert as f64;
    println!("File size ratio (after/before): {:.2}", size_ratio);
    
    // Reopen and check
    println!("\nReopening database...");
    let db = builder.open(models, &db_path).unwrap();
    
    let r = db.r_transaction().unwrap();
    let count = r.len().primary::<TestModel>().unwrap();
    println!("Records after reopen: {}", count);
    drop(r);
    drop(db);
    
    // Check final file size
    let metadata = std::fs::metadata(&db_path).unwrap();
    let final_size = metadata.len();
    println!("Final file size: {} bytes ({:.2} MB)", 
        final_size, final_size as f64 / 1024.0 / 1024.0);
}