# TinyMalloc Segment Space Utilization Analysis

## Test Configuration
- **Segment size**: 131,072 bytes (128 KB)
- **Total size classes**: 32
- **Word size**: 8 bytes (64-bit architecture)

## Individual Class Metrics

| Class | Object Size | Max Objects | Wasted Bytes | Utilization | Bitmap Words |
|-------|-------------|-------------|--------------|-------------|--------------|
| 0     | 8           | 16,119      | 0            | 100.0%      | 256          |
| 1     | 16          | 8,123       | 8            | 100.0%      | 128          |
| 2     | 24          | 5,429       | 16           | 100.0%      | 86           |
| 3     | 32          | 4,077       | 24           | 100.0%      | 64           |
| 4     | 40          | 3,264       | 24           | 100.0%      | 52           |
| 5     | 48          | 2,722       | 0            | 100.0%      | 43           |
| 6     | 56          | 2,334       | 0            | 100.0%      | 37           |
| 7     | 64          | 2,042       | 56           | 100.0%      | 32           |
| 8     | 72          | 1,816       | 16           | 100.0%      | 29           |
| 9     | 136         | 962         | 40           | 100.0%      | 16           |
| 10    | 200         | 654         | 112          | 99.9%       | 11           |
| 11    | 264         | 495         | 256          | 99.8%       | 8            |
| 12    | 392         | 334         | 24           | 100.0%      | 6            |
| 13    | 520         | 251         | 448          | 99.7%       | 4            |
| 14    | 648         | 202         | 72           | 99.9%       | 4            |
| 15    | 776         | 168         | 608          | 99.5%       | 3            |
| 16    | 904         | 144         | 800          | 99.4%       | 3            |
| 17    | 1,032       | 126         | 952          | 99.3%       | 2            |
| 18    | 3,080       | 42          | 1,632        | 98.8%       | 1            |
| 19    | 5,128       | 25          | 2,792        | 97.9%       | 1            |
| 20    | 7,176       | 18          | 1,824        | 98.6%       | 1            |
| 21    | 9,224       | 14          | 1,856        | 98.6%       | 1            |
| 22    | 13,320      | 9           | 11,112       | 91.5%       | 1            |
| 23    | 17,416      | 7           | 9,080        | 93.1%       | 1            |
| 24    | 21,512      | 6           | 1,920        | 98.5%       | 1            |
| 25    | 25,608      | 5           | 2,952        | 97.7%       | 1            |
| 26    | 29,704      | 4           | 12,176       | 90.7%       | 1            |
| 27    | 33,800      | 3           | 29,592       | 77.4%       | 1            |
| 28    | 37,896      | 3           | 17,304       | 86.8%       | 1            |
| 29    | 41,992      | 3           | 5,016        | 96.2%       | 1            |
| 30    | 46,088      | 2           | 38,816       | 70.4%       | 1            |
| 31    | 50,184      | 2           | 30,624       | 76.6%       | 1            |

## Summary Statistics

- **Perfect fit classes**: 3 out of 32 (9.4%)
  - Classes 0, 5, and 6 have zero waste
- **Best utilization**: 100.0% (classes 0-9, 12)
- **Worst utilization**: 70.4% (class 30, 46,088 bytes)
- **Average utilization**: 95.5%
- **Utilization range**: 70.4% - 100.0%

## Key Observations

1. **Small objects (≤136 bytes)** achieve perfect or near-perfect utilization
2. **Medium objects (200-1,032 bytes)** maintain >99% utilization
3. **Large objects (>3,000 bytes)** show significant degradation in utilization
4. **Critical threshold** appears around 30,000 bytes where utilization drops below 90%
5. **Bitmap overhead** scales inversely with object size (more objects = more bitmap words needed)

## Potential Optimizations

Based on the data, classes with <90% utilization could benefit from:
- Different segment sizes for large objects
- Alternative allocation strategies for objects >30KB
- Dynamic segment sizing based on object size class

Classes with poor utilization:
- Class 22 (13,320 bytes): 91.5% utilization
- Class 26 (29,704 bytes): 90.7% utilization  
- Class 27 (33,800 bytes): 77.4% utilization
- Class 30 (46,088 bytes): 70.4% utilization

---

# Performance Optimizations

## Benchmark Analysis Results

Current performance vs libc malloc:

| Allocation Type | TinyAlloc | libc | Performance Gap |
|----------------|-----------|------|-----------------|
| Small (64B)    | 34.85 ns  | 5.88 ns | **6x slower** ❌ |
| Medium (4KB)   | 16.98 ns  | 38.43 ns | **2.3x faster** ✅ |
| Large (1MB)    | 2,689 ns  | 27.79 ns | **97x slower** ❌ |
| Mixed Workload | 180.78 ns | 213.21 ns | **1.2x faster** ✅ |

## Priority Optimizations

### 1. Pointer Masking for O(1) Deallocation Lookup

**Problem**: Currently deallocating requires O(n) search through queues/segments to find which one contains the pointer.

**Solution**: Use pointer masking to instantly identify the arena/segment:

```rust
// Arena-level masking (existing arenas are 64MB aligned)
const ARENA_MASK: usize = !(ARENA_INITIAL_SIZE - 1);
fn find_arena_from_ptr(ptr: NonNull<u8>) -> Option<NonNull<Arena>> {
    let arena_start = (ptr.as_ptr() as usize) & ARENA_MASK;
    // Validate and return arena pointer
}

// Segment-level masking (segments are 128KB aligned within arenas)
const SEGMENT_MASK: usize = !(SEGMENT_SIZE - 1);
fn find_segment_from_ptr(ptr: NonNull<u8>, arena: &Arena) -> Option<NonNull<Segment>> {
    let segment_offset = ((ptr.as_ptr() as usize) & SEGMENT_MASK) - arena.start();
    let segment_index = segment_offset / SEGMENT_SIZE;
    // Return segment at index
}
```

**Impact**: Reduces deallocation from O(n) to O(1), especially beneficial for mixed workloads.

### 2. Small Allocation Fast Path

**Problem**: Small allocations are 6x slower than libc due to size class lookup overhead.

**Solutions**:
- **Inline size class lookup**: Use compile-time lookup tables instead of loops
- **Thread-local segment cache**: Cache last-used segment per size class
- **Bump allocation fallback**: For very small objects, use simple bump allocator within segments
- **Size class pre-computation**: Store size class index in layout or use lookup table

```rust
// Fast lookup table for common sizes
const SIZE_CLASS_TABLE: [u8; 256] = precomputed_size_classes();

#[inline(always)]
fn fast_size_class(size: usize) -> Option<u8> {
    if size <= 256 {
        Some(SIZE_CLASS_TABLE[size - 1])
    } else {
        slow_size_class_lookup(size)
    }
}
```

### 3. Large Allocation Optimization

**Problem**: Large allocations are 97x slower due to mmap overhead per allocation.

**Solutions**:
- **Large object threshold adjustment**: Move threshold from 64KB to 1MB+ to use size classes more
- **Large object pooling**: Pre-allocate common large sizes and reuse
- **Batch mmap operations**: Group multiple large allocations into single mmap call
- **Virtual memory reservation**: Reserve large address space upfront, commit on demand

### 4. Memory Layout Optimizations

**Current Issues**:
- Segment headers cause cache misses
- Bitmap fragmentation
- Poor spatial locality

**Solutions**:
- **Header consolidation**: Store all segment headers together for better cache locality
- **Bitmap packing**: Use more efficient bitmap representations (e.g., hierarchical bitmaps)
- **Allocation ordering**: Allocate from lowest addresses first for better cache performance

### 5. Queue Management Optimizations

**Problems**:
- Linear search through free/partial/full lists
- Excessive segment state transitions
- Poor cache locality

**Solutions**:
- **Segregated queues**: Separate queues by fullness level (0-25%, 25-50%, etc.)
- **Queue hints**: Remember last allocation position within queues
- **Lazy state updates**: Batch segment state transitions
- **LIFO allocation**: Prefer recently freed segments for cache warmth

### 6. Compiler and Architecture Optimizations

**Code optimizations**:
- **Branch prediction hints**: Use `likely()/unlikely()` for common paths
- **Function inlining**: Force inline critical allocation paths
- **SIMD operations**: Use vector instructions for bitmap operations
- **Prefetching**: Add memory prefetch hints for predictable access patterns

**Memory alignment**:
- **Cache line alignment**: Align critical structures to cache line boundaries
- **NUMA awareness**: Consider NUMA topology for arena placement
- **Huge pages**: Use huge pages for arenas when available

### 7. Lock-Free Optimizations

**Current synchronization overhead**:
- RwLock contention on arena access
- Atomic operations in queues

**Solutions**:
- **Thread-local arenas**: Reduce contention with per-thread allocation
- **Lock-free queues**: Use atomic operations instead of locks
- **Hazard pointers**: Safe memory reclamation without locks
- **Work stealing**: Allow threads to steal work from others

## Implementation Priority

1. **High Impact, Low Effort**:
   - Pointer masking for deallocation (addresses O(n) lookups)
   - Size class lookup table optimization
   - Inline critical functions

2. **High Impact, Medium Effort**:
   - Thread-local segment caching
   - Large allocation threshold tuning
   - Queue management improvements

3. **High Impact, High Effort**:
   - Lock-free queue implementation
   - Large object pooling system
   - Complete memory layout redesign

4. **Medium Impact**:
   - SIMD bitmap operations
   - Memory prefetching
   - Cache line alignment

The pointer masking optimization should be implemented first as it directly addresses the O(n) deallocation problem and has clear performance benefits with relatively low implementation complexity.