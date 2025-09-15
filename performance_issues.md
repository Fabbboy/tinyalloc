# Tinyalloc Performance Findings

## Benchmark Recap
- `cargo bench` (release build) on this machine shows **~19 ns** for `tinyalloc_small_allocations` versus **~5 ns** for `libc_small_allocations`.
- Mixed workload benchmark reports **~148 ns** for tinyalloc versus **~101 ns** for libc.
- Large allocation benchmark takes **~6.79 µs** under tinyalloc versus **~13.7 ns** with libc.
- `ini_parse_tiny` bench runs at **~22.23 µs**, slightly slower than the std allocator version at **~21.36 µs** (~4% gap).

## Identified Issues

### 1. Large allocations always round-trip through the kernel
- Implementation (`crates/tinyalloc-alloc/src/large.rs:31-54`) creates a fresh `Region` (mmap + mprotect) for every request and drops it on deallocation.
- Result: every large allocation/deallocation pair incurs two syscalls, leading to ~6–7 µs latency vs nanoseconds for glibc, which caches slabs.
- Optimization ideas: cache/recycle `Region`s per size bucket, or delay `munmap` via a free list to amortize syscall overhead.

### 2. Segment provisioning requires nested locks and global scanning
- `allocate_segment` (`crates/tinyalloc-alloc/src/static_.rs:61-85`) grabs a global `RwLock`, iterates all arenas, then each arena acquires its own `Mutex` via `Arena::has_space`/`Arena::allocate` (`arena.rs:185-187`, `arena.rs:116`).
- Even in single-threaded workloads this adds coarse-grained locking and repeated lock/unlock cycles.
- Optimization ideas: maintain per-thread/per-class arena pools, avoid the preliminary `has_space` call, or adopt lock-free freelists for available segments.

### 3. Deallocation scans multiple lists linearly to find the owning segment
- `Queue::deallocate` (`crates/tinyalloc-alloc/src/queue.rs:76-111`) walks `free`, `partial`, then `full` lists using iterators that traverse every segment, checking `contains_ptr`.
- As the allocator accumulates segments, free/dealloc operations scale linearly, hurting tight loops of small allocations.
- Optimization ideas: encode the segment pointer alongside each allocation (e.g., prefix metadata or segregated object headers) so the queue can look it up in O(1) without traversing lists.

### 4. Memory commitment on every segment allocation
- When allocating a segment, `Arena::allocate` (`arena.rs:136-143`) calls `Region::partial` to `mprotect` the segment into read/write state immediately.
- This incurs an extra syscall per segment growth compared with allocators that lazily commit pages.
- Optimization ideas: batch or defer `mprotect` calls, or rely on demand paging where feasible to reduce syscall pressure.

## Suggested Next Steps
1. Introduce a large-allocation cache or slab reuse strategy to eliminate redundant `mmap/munmap` pairs.
2. Restructure arena management to avoid global locks (per-thread caches, lock-free queues, or reduced locking scope).
3. Add metadata that lets `deallocate` derive the owning segment directly, eliminating list scans.
4. Revisit when pages are committed to reduce the number of `mprotect`/`madvise` calls under load.
