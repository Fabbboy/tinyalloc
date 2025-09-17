use tinyalloc::TinyAlloc;
use std::thread;
use std::sync::mpsc;

#[global_allocator]
static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

const ALLOCATION_SIZE: usize = 1024 * 1024; // 1MB

fn main() {
    println!("Testing same-thread allocation/deallocation with debug output...");

    // Simple same-thread test first
    println!("=== Single thread test ===");
    let memory: Vec<u8> = vec![42; 1000];
    let ptr = memory.as_ptr();
    println!("Allocated at {:p}", ptr);
    drop(memory);
    println!("Freed at {:p}", ptr);

    // Multi-threaded same-thread test
    println!("=== Multi-thread same-thread test ===");
    let mut handles = Vec::new();

    for thread_id in 0..2 {
        let handle = thread::spawn(move || {
            for i in 0..3 {
                println!("Thread {} - Allocation {}", thread_id, i);
                let memory: Vec<u8> = vec![42; 1000];
                let ptr = memory.as_ptr();
                println!("Thread {} - Allocated {} at {:p}", thread_id, i, ptr);
                drop(memory);
                println!("Thread {} - Freed {} at {:p}", thread_id, i, ptr);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("Test completed");
}
