use tinyalloc_alloc::queue::Queue;
use tinyalloc_config::classes::{class_init, CLASSES};

fn main() {
    let _classes = class_init(|_| ());
    let class = &CLASSES[7]; // 64-byte class (same as benchmark)

    let mut queue = Queue::new(class);

    println!("Class info: size={}, align={}, id={}",
             class.size.0, class.align.0, class.id);

    println!("Initial state:");
    println!("  has_available: {}", queue.has_available());

    // Do many allocations to see if we reuse segments
    let mut ptrs = Vec::new();
    for i in 1..=1000 {
        let ptr = queue.allocate();
        if let Some(p) = ptr {
            ptrs.push(p);
        }
        if i % 100 == 0 {
            println!("After {} allocations: has_available={}", i, queue.has_available());
        }
    }

    println!("Allocated {} pointers", ptrs.len());
}