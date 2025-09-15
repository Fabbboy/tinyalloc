use criterion::{
  Criterion,
  criterion_group,
  criterion_main,
};
use std::{
  alloc::Layout,
  hint::black_box,
  ptr::NonNull,
};
use tinyalloc_alloc::heap::Heap;

fn small_allocations(c: &mut Criterion) {
  c.bench_function("small_allocations", |b| {
    let mut heap = Heap::new();

    b.iter(|| {
      let layout = Layout::from_size_align(64, 8).unwrap();
      let ptr = black_box(heap.allocate(layout).unwrap());
      heap
        .deallocate(
          unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
          layout,
        )
        .unwrap();
    });
  });
}

fn medium_allocations(c: &mut Criterion) {
  c.bench_function("medium_allocations", |b| {
    let mut heap = Heap::new();

    b.iter(|| {
      let layout = Layout::from_size_align(4096, 8).unwrap();
      let ptr = black_box(heap.allocate(layout).unwrap());
      heap
        .deallocate(
          unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
          layout,
        )
        .unwrap();
    });
  });
}

fn large_allocations(c: &mut Criterion) {
  c.bench_function("large_allocations", |b| {
    let mut heap = Heap::new();

    b.iter(|| {
      let layout = Layout::from_size_align(1024 * 1024, 8).unwrap();
      let ptr = black_box(heap.allocate(layout).unwrap());
      heap
        .deallocate(
          unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
          layout,
        )
        .unwrap();
    });
  });
}

fn mixed_workload(c: &mut Criterion) {
  c.bench_function("mixed_workload", |b| {
    let mut heap = Heap::new();
    let mut allocations = Vec::new();

    b.iter(|| {
      // Allocate various sizes
      for size in [32, 128, 512, 2048, 8192].iter() {
        let layout = Layout::from_size_align(*size, 8).unwrap();
        if let Ok(ptr) = heap.allocate(layout) {
          allocations.push((ptr, layout));
        }
      }

      // Deallocate half
      for _ in 0..allocations.len() / 2 {
        if let Some((ptr, layout)) = allocations.pop() {
          let _ = heap.deallocate(
            unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
            layout,
          );
        }
      }

      // Clean up remaining
      for (ptr, layout) in allocations.drain(..) {
        let _ = heap.deallocate(
          unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
          layout,
        );
      }
    });
  });
}

fn size_class_distribution(c: &mut Criterion) {
  c.bench_function("size_class_distribution", |b| {
    let mut heap = Heap::new();

    b.iter(|| {
      let mut ptrs = Vec::new();

      // Test all size classes
      for size in [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096].iter() {
        let layout = Layout::from_size_align(*size, 8).unwrap();
        if let Ok(ptr) = heap.allocate(layout) {
          ptrs.push((ptr, layout));
        }
      }

      // Deallocate all
      for (ptr, layout) in ptrs {
        let _ = heap.deallocate(
          unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
          layout,
        );
      }
    });
  });
}

criterion_group!(
  benches,
  small_allocations,
  medium_allocations,
  large_allocations,
  mixed_workload,
  size_class_distribution
);
criterion_main!(benches);
