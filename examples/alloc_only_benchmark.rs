use std::{
  alloc::Layout,
  time::Instant,
};
use tinyalloc_alloc::heap::Heap;
use tinyalloc_config::metrics::{start_summary, print_summary};

const ITERATIONS: usize = 10_000;

fn main() {
  println!("TinyAlloc ALLOC-ONLY Analysis with {} iterations", ITERATIONS);

  // Start collecting metrics
  start_summary();

  let mut heap = Heap::new();
  let start_time = Instant::now();
  let mut ptrs = Vec::new();

  // Test: Only allocations, no deallocations
  for _ in 0..ITERATIONS {
    let layout = Layout::from_size_align(64, 8).unwrap();
    if let Ok(ptr) = heap.allocate(layout) {
      ptrs.push(ptr);
    }
  }

  let elapsed = start_time.elapsed();
  let ops_per_sec = ITERATIONS as f64 / elapsed.as_secs_f64();

  println!("Completed {} allocations in {:.3}s", ITERATIONS, elapsed.as_secs_f64());
  println!("Performance: {:.0} operations/second", ops_per_sec);

  // Print detailed metrics
  print_summary();

  println!("Allocated {} total objects", ptrs.len());
}