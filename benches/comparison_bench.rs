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

fn libc_small_allocations(c: &mut Criterion) {
  c.bench_function("libc_small_allocations", |b| {
    b.iter(|| {
      let ptr = unsafe { libc::malloc(64) };
      black_box(ptr);
      unsafe { libc::free(ptr) };
    });
  });
}

fn tinyalloc_small_allocations(c: &mut Criterion) {
  c.bench_function("tinyalloc_small_allocations", |b| {
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

fn libc_medium_allocations(c: &mut Criterion) {
  c.bench_function("libc_medium_allocations", |b| {
    b.iter(|| {
      let ptr = unsafe { libc::malloc(4096) };
      black_box(ptr);
      unsafe { libc::free(ptr) };
    });
  });
}

fn tinyalloc_medium_allocations(c: &mut Criterion) {
  c.bench_function("tinyalloc_medium_allocations", |b| {
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

fn libc_large_allocations(c: &mut Criterion) {
  c.bench_function("libc_large_allocations", |b| {
    b.iter(|| {
      let ptr = unsafe { libc::malloc(1024 * 1024) };
      black_box(ptr);
      unsafe { libc::free(ptr) };
    });
  });
}

fn tinyalloc_large_allocations(c: &mut Criterion) {
  c.bench_function("tinyalloc_large_allocations", |b| {
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

fn libc_mixed_workload(c: &mut Criterion) {
  c.bench_function("libc_mixed_workload", |b| {
    b.iter(|| {
      let mut ptrs = Vec::new();

      for size in [32, 128, 512, 2048, 8192].iter() {
        let ptr = unsafe { libc::malloc(*size) };
        if !ptr.is_null() {
          ptrs.push(ptr);
        }
      }

      for _ in 0..ptrs.len() / 2 {
        if let Some(ptr) = ptrs.pop() {
          unsafe { libc::free(ptr) };
        }
      }

      for ptr in ptrs {
        unsafe { libc::free(ptr) };
      }
    });
  });
}

fn tinyalloc_mixed_workload(c: &mut Criterion) {
  c.bench_function("tinyalloc_mixed_workload", |b| {
    let mut heap = Heap::new();

    b.iter(|| {
      let mut allocations = Vec::new();

      for size in [32, 128, 512, 2048, 8192].iter() {
        let layout = Layout::from_size_align(*size, 8).unwrap();
        if let Ok(ptr) = heap.allocate(layout) {
          allocations.push((ptr, layout));
        }
      }

      for _ in 0..allocations.len() / 2 {
        if let Some((ptr, layout)) = allocations.pop() {
          let _ = heap.deallocate(
            unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
            layout,
          );
        }
      }

      for (ptr, layout) in allocations {
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
  libc_small_allocations,
  tinyalloc_small_allocations,
  libc_medium_allocations,
  tinyalloc_medium_allocations,
  libc_large_allocations,
  tinyalloc_large_allocations,
  libc_mixed_workload,
  tinyalloc_mixed_workload
);
criterion_main!(benches);
