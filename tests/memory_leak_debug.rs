use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
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
fn debug_memory_pattern() {
    println!("\n=== Memory Pattern Debug Test ===");
    let memory_tracker = MemoryTracker::new();
    
    // Print initial memory
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!("Initial - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }
    
    let (_tmp_dir, db) = create_temp_db();
    
    // Print after DB creation
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!("After DB creation - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }
    
    const BATCH_SIZE: u32 = 100;
    const DATA_SIZE: usize = 1024 * 10; // 10KB per record
    
    // Insert in batches and monitor memory
    for batch in 0..10 {
        let start_idx = batch * BATCH_SIZE;
        let end_idx = start_idx + BATCH_SIZE;
        
        for i in start_idx..end_idx {
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
            println!("After batch {} ({} records) - Physical: {}, Virtual: {}",
                batch + 1,
                end_idx,
                AllocationStats::format_bytes(stats.physical),
                AllocationStats::format_bytes(stats.virtual_mem)
            );
        }
    }
    
    println!("\n--- Starting deletion phase ---");
    
    // Delete in batches and monitor memory
    for batch in 0..10 {
        let start_idx = batch * BATCH_SIZE;
        let end_idx = start_idx + BATCH_SIZE;
        
        for i in start_idx..end_idx {
            let rw = db.rw_transaction().unwrap();
            let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
            rw.remove(model).unwrap();
            rw.commit().unwrap();
        }
        
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("After deleting batch {} ({} records deleted) - Physical: {}, Virtual: {}",
                batch + 1,
                end_idx,
                AllocationStats::format_bytes(stats.physical),
                AllocationStats::format_bytes(stats.virtual_mem)
            );
        }
    }
    
    // Try to trigger cleanup
    println!("\n--- Attempting cleanup ---");
    drop(db);
    
    if let Some(stats) = MemoryTracker::get_current_memory() {
        println!("After dropping database - Physical: {}, Virtual: {}",
            AllocationStats::format_bytes(stats.physical),
            AllocationStats::format_bytes(stats.virtual_mem)
        );
    }
    
    // Final check
    match memory_tracker.check_memory_growth(50.0) {
        Ok(()) => println!("\nMemory usage within acceptable limits"),
        Err(msg) => println!("\nWARNING: {}", msg),
    }
}