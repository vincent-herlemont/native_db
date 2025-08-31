use assert_fs::TempDir;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;

mod common;
use common::{AllocationStats, AllocationTracker};

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
    let db = Builder::new().create(models, db_path).unwrap();
    (tmp_dir, db)
}

#[test]
fn test_allocation_pattern_basic_operations() {
    let tracker = AllocationTracker::new();
    let (_tmp_dir, db) = create_temp_db();

    // Reset tracker after DB creation
    tracker.reset();

    // Track insertions
    println!("\n=== Testing Insertion Allocations ===");
    for i in 0..10u32 {
        let initial_stats = tracker.get_stats();

        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: i,
            name: format!("test_{}", i),
            data: vec![0u8; 1024], // 1KB
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();

        let after_stats = tracker.get_stats();
        println!(
            "Insert #{}: allocations: +{}, current memory: {}",
            i,
            after_stats.allocations - initial_stats.allocations,
            AllocationStats::format_bytes(after_stats.current_bytes)
        );
    }

    // Track reads
    println!("\n=== Testing Read Allocations ===");
    let read_start_stats = tracker.get_stats();

    for i in 0..10u32 {
        let r = db.r_transaction().unwrap();
        let _model = r.get().primary::<TestModel>(i).unwrap();
        drop(r);
    }

    let read_end_stats = tracker.get_stats();
    println!(
        "Total read allocations: {}, memory delta: {}",
        read_end_stats.allocations - read_start_stats.allocations,
        AllocationStats::format_bytes(
            read_end_stats
                .current_bytes
                .saturating_sub(read_start_stats.current_bytes)
        )
    );

    // Track deletions
    println!("\n=== Testing Deletion Allocations ===");
    let delete_start_stats = tracker.get_stats();

    for i in 0..10u32 {
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(i).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();
    }

    let final_stats = tracker.get_stats();
    println!(
        "Total delete allocations: {}, final memory: {}",
        final_stats.allocations - delete_start_stats.allocations,
        AllocationStats::format_bytes(final_stats.current_bytes)
    );

    // Check for leaks
    println!("\n=== Final Statistics ===");
    println!("Total allocations: {}", final_stats.allocations);
    println!("Total deallocations: {}", final_stats.deallocations);
    println!(
        "Peak memory: {}",
        AllocationStats::format_bytes(final_stats.peak_bytes)
    );
    println!(
        "Current memory: {}",
        AllocationStats::format_bytes(final_stats.current_bytes)
    );

    // Note: We can't assert on exact values as the allocator internals vary
    // But we can check that memory is eventually released
    if final_stats.current_bytes > final_stats.peak_bytes / 2 {
        println!("Warning: More than half of peak memory still allocated");
    }
}

#[test]
fn test_transaction_rollback_allocations() {
    let tracker = AllocationTracker::new();
    let (_tmp_dir, db) = create_temp_db();

    tracker.reset();

    println!("\n=== Testing Transaction Rollback ===");

    // Successful transaction
    let success_start = tracker.get_stats();
    {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: 1,
            name: "success".to_string(),
            data: vec![0u8; 10240], // 10KB
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();
    }
    let success_end = tracker.get_stats();

    println!(
        "Successful transaction - allocations: {}, memory: {}",
        success_end.allocations - success_start.allocations,
        AllocationStats::format_bytes(success_end.current_bytes)
    );

    // Rolled back transaction
    let rollback_start = tracker.get_stats();
    {
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: 2,
            name: "rollback".to_string(),
            data: vec![0u8; 10240], // 10KB
        };
        rw.insert(model).unwrap();
        // Drop without commit - should rollback
        drop(rw);
    }
    let rollback_end = tracker.get_stats();

    println!(
        "Rolled back transaction - allocations: {}, memory delta: {}",
        rollback_end.allocations - rollback_start.allocations,
        AllocationStats::format_bytes(
            rollback_end
                .current_bytes
                .saturating_sub(rollback_start.current_bytes)
        )
    );

    // Memory after rollback should be similar to after successful transaction
    let memory_diff = rollback_end.current_bytes as isize - success_end.current_bytes as isize;
    println!(
        "Memory difference after rollback: {} bytes",
        memory_diff.abs()
    );
}

#[test]
fn test_concurrent_allocation_patterns() {
    let tracker = Arc::new(AllocationTracker::new());
    let (_tmp_dir, db) = create_temp_db();
    let db = Arc::new(db);

    tracker.reset();

    println!("\n=== Testing Concurrent Allocations ===");
    let start_stats = tracker.get_stats();

    let mut handles = vec![];

    for thread_id in 0..4u32 {
        let db_clone = Arc::clone(&db);
        let tracker_clone = Arc::clone(&tracker);

        let handle = thread::spawn(move || {
            for i in 0..25u32 {
                let id = thread_id * 100 + i;

                let rw = db_clone.rw_transaction().unwrap();
                let model = TestModel {
                    id,
                    name: format!("thread_{}_item_{}", thread_id, i),
                    data: vec![0u8; 1024],
                };
                rw.insert(model).unwrap();
                rw.commit().unwrap();

                if i % 10 == 0 {
                    let stats = tracker_clone.get_stats();
                    println!(
                        "Thread {} at iteration {}: current memory: {}",
                        thread_id,
                        i,
                        AllocationStats::format_bytes(stats.current_bytes)
                    );
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let end_stats = tracker.get_stats();
    println!("\nConcurrent test complete:");
    println!(
        "Total allocations: {}",
        end_stats.allocations - start_stats.allocations
    );
    println!(
        "Peak memory: {}",
        AllocationStats::format_bytes(end_stats.peak_bytes)
    );
    println!(
        "Current memory: {}",
        AllocationStats::format_bytes(end_stats.current_bytes)
    );
}

#[test]
fn test_large_object_allocation_deallocation() {
    let tracker = AllocationTracker::new();
    let (_tmp_dir, db) = create_temp_db();

    tracker.reset();

    println!("\n=== Testing Large Object Lifecycle ===");

    // Test various sizes
    let sizes = vec![
        (1024, "1KB"),
        (10 * 1024, "10KB"),
        (100 * 1024, "100KB"),
        (1024 * 1024, "1MB"),
    ];

    for (idx, (size, label)) in sizes.iter().enumerate() {
        let start_stats = tracker.get_stats();

        // Insert large object
        let rw = db.rw_transaction().unwrap();
        let model = TestModel {
            id: idx as u32,
            name: format!("large_{}", label),
            data: vec![0u8; *size],
        };
        rw.insert(model).unwrap();
        rw.commit().unwrap();

        let after_insert = tracker.get_stats();

        // Read it back
        let r = db.r_transaction().unwrap();
        let _model = r.get().primary::<TestModel>(idx as u32).unwrap();
        drop(r);

        let after_read = tracker.get_stats();

        // Delete it
        let rw = db.rw_transaction().unwrap();
        let model = rw.get().primary::<TestModel>(idx as u32).unwrap().unwrap();
        rw.remove(model).unwrap();
        rw.commit().unwrap();

        let after_delete = tracker.get_stats();

        println!("{} object lifecycle:", label);
        println!(
            "  After insert: {} allocations, {} memory",
            after_insert.allocations - start_stats.allocations,
            AllocationStats::format_bytes(after_insert.current_bytes)
        );
        println!(
            "  After read: {} allocations, {} memory",
            after_read.allocations - after_insert.allocations,
            AllocationStats::format_bytes(after_read.current_bytes)
        );
        println!(
            "  After delete: {} allocations, {} memory",
            after_delete.allocations - after_read.allocations,
            AllocationStats::format_bytes(after_delete.current_bytes)
        );
        println!(
            "  Memory freed: {}",
            AllocationStats::format_bytes(
                after_insert
                    .current_bytes
                    .saturating_sub(after_delete.current_bytes)
            )
        );
    }
}
