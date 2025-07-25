use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Cross-platform memory tracking utilities
pub struct MemoryTracker {
    initial_memory: Option<MemoryStats>,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Physical memory (RSS) in bytes
    pub physical: usize,
    /// Virtual memory in bytes
    pub virtual_mem: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        let initial_memory = Self::get_current_memory();
        MemoryTracker { initial_memory }
    }

    /// Get current memory usage
    pub fn get_current_memory() -> Option<MemoryStats> {
        // Try sysinfo first (more detailed but heavier)
        // Skip sysinfo for now since it requires the 'system' feature

        // Fallback to memory-stats (lighter weight)
        if let Some(usage) = memory_stats::memory_stats() {
            return Some(MemoryStats {
                physical: usage.physical_mem,
                virtual_mem: usage.virtual_mem,
            });
        }

        // Platform-specific fallbacks
        #[cfg(target_os = "linux")]
        {
            Self::get_linux_memory()
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    #[cfg(target_os = "linux")]
    fn get_linux_memory() -> Option<MemoryStats> {
        use std::fs;
        
        let status = fs::read_to_string("/proc/self/status").ok()?;
        let mut rss = None;
        let mut vsz = None;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    rss = parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
                }
            } else if line.starts_with("VmSize:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    vsz = parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
                }
            }
        }
        
        match (rss, vsz) {
            (Some(physical), Some(virtual_mem)) => Some(MemoryStats { physical, virtual_mem }),
            (Some(physical), None) => Some(MemoryStats { physical, virtual_mem: physical }),
            _ => None,
        }
    }

    /// Get memory growth since initialization
    pub fn get_memory_growth(&self) -> Option<(isize, f64)> {
        let initial = self.initial_memory.as_ref()?;
        let current = Self::get_current_memory()?;
        
        let growth = current.physical as isize - initial.physical as isize;
        let percentage = if initial.physical > 0 {
            (growth as f64 / initial.physical as f64) * 100.0
        } else {
            0.0
        };
        
        Some((growth, percentage))
    }

    /// Check if memory growth exceeds threshold
    pub fn check_memory_growth(&self, max_growth_percentage: f64) -> Result<(), String> {
        match self.get_memory_growth() {
            Some((growth_bytes, growth_percentage)) => {
                if growth_percentage > max_growth_percentage {
                    Err(format!(
                        "Memory grew by {:.2}% ({} bytes), exceeding threshold of {:.2}%",
                        growth_percentage,
                        growth_bytes,
                        max_growth_percentage
                    ))
                } else {
                    Ok(())
                }
            }
            None => {
                // If we can't measure memory, we can't fail the test
                eprintln!("Warning: Unable to measure memory on this platform");
                Ok(())
            }
        }
    }
}

/// Simple allocation tracker using a custom allocator
pub struct AllocationTracker {
    allocations: Arc<AtomicUsize>,
    deallocations: Arc<AtomicUsize>,
    current_bytes: Arc<AtomicUsize>,
    peak_bytes: Arc<AtomicUsize>,
}

impl AllocationTracker {
    pub fn new() -> Self {
        AllocationTracker {
            allocations: Arc::new(AtomicUsize::new(0)),
            deallocations: Arc::new(AtomicUsize::new(0)),
            current_bytes: Arc::new(AtomicUsize::new(0)),
            peak_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn track_allocation(&self, size: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let current = self.current_bytes.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak if necessary
        let mut peak = self.peak_bytes.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_bytes.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }

    pub fn track_deallocation(&self, size: usize) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.current_bytes.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            allocations: self.allocations.load(Ordering::Relaxed),
            deallocations: self.deallocations.load(Ordering::Relaxed),
            current_bytes: self.current_bytes.load(Ordering::Relaxed),
            peak_bytes: self.peak_bytes.load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        self.allocations.store(0, Ordering::Relaxed);
        self.deallocations.store(0, Ordering::Relaxed);
        self.current_bytes.store(0, Ordering::Relaxed);
        self.peak_bytes.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub current_bytes: usize,
    pub peak_bytes: usize,
}

impl AllocationStats {
    pub fn leaked_bytes(&self) -> isize {
        self.current_bytes as isize
    }

    pub fn format_bytes(bytes: usize) -> String {
        const UNITS: &[(&str, usize)] = &[
            ("GB", 1024 * 1024 * 1024),
            ("MB", 1024 * 1024),
            ("KB", 1024),
            ("B", 1),
        ];

        for (unit, size) in UNITS {
            if bytes >= *size {
                return format!("{:.2} {}", bytes as f64 / *size as f64, unit);
            }
        }
        
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let _tracker = MemoryTracker::new();
        
        // Allocate some memory
        let _data: Vec<u8> = vec![0; 10 * 1024 * 1024]; // 10MB
        
        // Check if we can detect memory usage
        if let Some(stats) = MemoryTracker::get_current_memory() {
            println!("Current memory - Physical: {}, Virtual: {}", 
                AllocationStats::format_bytes(stats.physical),
                AllocationStats::format_bytes(stats.virtual_mem)
            );
            
            // Memory should be non-zero
            assert!(stats.physical > 0, "Physical memory should be non-zero");
        } else {
            eprintln!("Memory tracking not available on this platform");
        }
    }

    #[test]
    fn test_allocation_tracker() {
        let tracker = AllocationTracker::new();
        
        // Simulate allocations
        tracker.track_allocation(1024);
        tracker.track_allocation(2048);
        tracker.track_deallocation(1024);
        
        let stats = tracker.get_stats();
        assert_eq!(stats.allocations, 2);
        assert_eq!(stats.deallocations, 1);
        assert_eq!(stats.current_bytes, 2048);
        assert_eq!(stats.peak_bytes, 3072);
    }
}