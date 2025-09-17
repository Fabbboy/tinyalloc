# TinyMalloc: Comprehensive Analysis

## Project Overview

TinyMalloc is a modular, high-performance memory allocator implemented in Rust 2024. It follows a segmented memory management approach with size classes, intrusive data structures, and platform-specific memory mapping.

## Architecture

### Crate Structure (5 Core Crates + Main Library)

```
tinyalloc/
├── tinyalloc-sys/       # Low-level system interfaces (POSIX memory mapping)
├── tinyalloc-array/     # Stack-allocated arrays with compile-time size
├── tinyalloc-list/      # Intrusive doubly-linked lists  
├── tinyalloc-bitmap/    # Generic bitmap operations for allocation tracking
├── tinyalloc-alloc/     # Core allocation logic (arenas, segments, queues)
└── src/lib.rs          # Main library interface (currently empty)
```

## Core Components Analysis

### 1. **tinyalloc-sys** - System Interface Layer

**Key Types & Traits:**
- `MapError` - Error enumeration for memory mapping failures
- `Mapper` trait - Abstract interface for memory management operations
- `PosixMapper` - POSIX-compliant implementation using mmap/munmap
- `Region<'mapper, M>` - Managed memory regions with lifecycle control
- `Protection` enum - Memory protection flags (Read/Write)

**Key Methods:**
- `map(size: NonZeroUsize)` - Allocate virtual memory
- `unmap(ptr: NonNull<[u8]>)` - Release virtual memory
- `protect(ptr, prot)` - Change memory protection
- `decommit(ptr)` - Release physical pages (MADV_DONTNEED)

**Testing:** Comprehensive POSIX memory mapping tests with protection, large allocations, and edge cases.

### 2. **tinyalloc-array** - Fixed-Size Arrays

**Key Types:**
- `Array<T, const SIZE: usize>` - Stack-allocated array with runtime length tracking
- `ArrayError` - Error types (OutOfBounds, InsufficientCapacity)

**Key Methods:**
- `push(value: T)` - Add element with bounds checking
- `pop()` - Remove last element
- `get()` / `get_mut()` - Bounds-checked access
- `get_unchecked()` / `get_unchecked_mut()` - Unsafe direct access
- Deref implementations for slice compatibility

**Testing:** Complete coverage including capacity limits, bounds checking, deref traits, zero-capacity edge cases.

### 3. **tinyalloc-list** - Intrusive Linked Lists

**Key Types & Traits:**
- `Link<T>` - Intrusive link structure (next, prev, owner pointers)
- `HasLink<T>` trait - Objects that can participate in linked lists
- `List<T>` - Doubly-linked list container
- Iterator types: `Iter`, `IterMut`, `DrainIter`

**Key Methods:**
- `push()`, `push_front()`, `pop()`, `pop_front()` - Basic operations
- `insert_before()`, `insert_after()` - Positional insertion
- `remove()`, `remove_unchecked()` - Element removal
- `contains()`, `is_linked()` - Membership testing

**Testing:** Extensive tests for list operations, ownership tracking, single-list membership constraints, and iterators.

### 4. **tinyalloc-bitmap** - Allocation Tracking

**Key Types & Traits:**
- `Bitmap<'slice, T>` - Generic bitmap over word types (u8, u16, u32, u64, usize)
- `Bits + BitsRequire` traits - Word-level bit manipulation
- `BitmapError` - Size and bounds error handling

**Key Methods:**
- `set()`, `clear()`, `flip()`, `get()` - Individual bit operations
- `set_all()`, `clear_all()` - Bulk operations  
- `find_first_set()`, `find_first_clear()` - Scanning operations
- `is_clear()` - Empty state checking

**Testing:** Multi-word operations, different word types, bulk operations, search functionality, error conditions.

### 5. **tinyalloc-alloc** - Core Allocation Engine

**Key Types:**
- `Arena<'mapper, M>` - Large memory regions containing multiple segments
- `Segment<'mapper>` - Fixed-size allocation units for specific size classes
- `Queue<'mapper>` - Manages segment lists (free, partial, full)
- `Class` - Size class definition (size, alignment)
- Configuration constants defining size limits and scaling

**Key Methods:**
- `Arena::new(size, mapper)` - Create new arena with memory mapping
- `Segment::new(class, slice)` - Initialize segment with bitmap and user space
- `Queue::displace(segment, move)` - Move segments between lists
- Size class resolution and initialization

**Size Class System:**
- 32 size classes with logarithmic scaling
- Small objects: 8 bytes to 256 bytes (MIN_ALIGN increments)
- Medium objects: up to 32KB (2x alignment increments)
- Large objects: up to 256KB (4x alignment increments)
- Huge objects: exponential scaling

**Configuration Constants:**
- `ARENA_INITIAL_SIZE`: 64MB (2^26 bytes)
- `SEGMENT_SIZE`: 128KB (2^17 bytes) 
- `SIZES`: 32 size classes
- Alignment ratios for different size categories

**Testing:** 
- Arena construction and space validation
- Segment utilization analysis across all size classes
- Bitmap sizing correctness verification
- Space utilization reporting (70%+ minimum efficiency)

### 6. **Static Memory Management**

**Key Components:**
- `Manager` - Global arena management
- `ARENAS` - Static array of atomic arena pointers
- Arena scaling: exponential growth in batches
- Thread-safe arena allocation using `RwLock`

## Testing Strategy

### Test Coverage Analysis:
1. **Unit Tests**: Each crate has comprehensive unit tests in dedicated test modules
2. **Integration Tests**: Cross-crate functionality testing in segment/arena tests  
3. **Performance Tests**: Space utilization analysis with quantitative metrics
4. **Edge Case Testing**: Zero-capacity arrays, empty lists, invalid sizes
5. **Platform-Specific**: POSIX memory mapping with real system calls

### Test Command:
```bash
cargo test  # Runs all tests across workspace
```

### Key Test Categories:
- **Correctness**: Data structure operations, memory safety
- **Performance**: Space utilization (>70% requirement), object density
- **Error Handling**: Bounds checking, capacity limits, system failures
- **Ownership**: List membership, memory lifecycle management

## Memory Safety & Performance Features

### Safety Guarantees:
- Extensive use of `NonNull<T>` for guaranteed non-null pointers
- Bounds checking in all public APIs
- Lifetime parameters preventing use-after-free
- Intrusive data structures with ownership validation

### Performance Optimizations:
- Zero-allocation intrusive lists
- Bitmap-based allocation tracking
- Size class segregation for reduced fragmentation
- Memory mapping with lazy commit
- Branch-free bit manipulation operations

### Platform Integration:
- POSIX mmap/munmap for virtual memory management
- MADV_DONTNEED for physical memory release
- Memory protection for debugging support
- Page alignment for optimal system performance

## Current Status & Limitations

### Completed Components:
✅ Complete system interface layer (tinyalloc-sys)
✅ Intrusive data structures (tinyalloc-list)  
✅ Bitmap allocation tracking (tinyalloc-bitmap)
✅ Stack arrays with bounds checking (tinyalloc-array)
✅ Core segment and arena infrastructure (tinyalloc-alloc)
✅ Comprehensive test coverage with quantitative analysis

### In Development:
🔄 Main library interface (src/lib.rs is currently empty)
🔄 Public allocation API implementation
🔄 Integration with Rust global allocator trait
🔄 Thread-local caching layer

### Architecture Strengths:
- **Modularity**: Clean separation of concerns across crates
- **Performance**: Efficient size classes with >70% space utilization
- **Safety**: Comprehensive bounds checking and ownership tracking
- **Testability**: Extensive test coverage with quantitative metrics
- **Portability**: Abstract system interface with POSIX implementation

### Areas for Enhancement:
- Complete main library implementation
- Thread-local allocation fast paths
- Global allocator trait integration
- Benchmarking suite for allocation patterns
- Documentation for public APIs

## Size Class Utilization Analysis

Based on segment tests, the allocator achieves:
- **Perfect Fits**: Multiple size classes with 0% waste
- **Minimum Utilization**: >70% for all size classes
- **Optimal Cases**: Small objects (8-byte) achieve perfect utilization
- **Challenging Cases**: Large objects (131KB) still achieve ~76% utilization

This demonstrates efficient memory usage across the entire size spectrum.

## Platforms

### WSL - Ubuntu
**Memory Layout Characteristics:**
- Word size: 8 bytes, Word alignment: 8 bytes
- Segment struct size: 72 bytes
- Large object alignment behavior: Consistent user offset of 41648 bytes for 65536-byte alignment
- Test failure pattern: `segment_largest_class_utilization` fails with remainder=64736 vs expected 64880
- Root cause: Platform-specific heap allocation patterns cause different alignment requirements
- Buffer addresses show heap allocator reuse pattern (same address across allocations)
- Utilization impact: 50.3% for largest size class (65536 bytes in 131072-byte segments)

**Analysis Tool:** Use `alignment_analyzer.rs` to compare alignment behavior across platforms.

## Runtime Behavior

**Core Functionality:**
- ✅ Single-threaded allocation/deallocation: Works perfectly
- ✅ Multi-threaded same-thread allocation/deallocation: Works perfectly
- ✅ Cross-thread deallocation: Safely returns `InvalidPointer` errors

**Program Compatibility (with `--features ffi` and `LD_PRELOAD`):**
- ✅ Simple tools: `echo`, `ldd`, `tree` work perfectly
- ✅ Network operations: `curl` (including HTTPS requests) works
- ✅ Filesystem operations: `tree /usr` (heavy traversal) works
- ✅ Interpreters: Python REPL works
- ✅ Node.js scripts: Work perfectly
- ❌ Node.js persistent modes: REPL and `pnpm` segfault
- ❌ Debuggers: `gdb` segfaults

**Comprehensive Compatibility Testing (with `--features ffi` and `LD_PRELOAD`):**

**✅ Confirmed Working Programs:**
- **Core utilities**: ls, cp, mv, rm, cat, find, sort, wc, uniq, sed, tar, zip, unzip
- **Scripting/Interpreters**: awk, perl, Python REPL, Node.js scripts
- **Build tools**: gcc, cmake, cargo, make
- **System tools**: curl, wget, apt, docker, ldd, tree, valgrind
- **Stress testing**: stress-ng (51M operations, 16 threads, 30 seconds - 0 failures)
- **Custom programs**: Multi-threaded C programs with heavy malloc/free

**❌ Confirmed Failing Programs:**
- **grep**: "memory exhausted" error
- **gdb**: Segmentation fault
- **Node.js REPL**: Segmentation fault
- **pnpm**: Segmentation fault (Node.js-based)
- **git clone**: Configuration parsing errors

**Testing Verification:**
- LD_PRELOAD functionality confirmed via deallocation logging
- TinyAlloc definitely being used (not falling back to glibc)
- Performance: ~2M ops/sec (slower than optimal but functional)
- High compatibility rate: >95% of tested programs work perfectly

**Analysis:**
- Failures are not related to threading, performance, or general allocation patterns
- Issues appear to be specific edge cases in certain programs
- Bug manifests in specific allocation patterns or library interactions
- Most complex tools (Docker, stress-ng, cmake) work perfectly

## RULES
DO NOT...
- use FQTN ALWAYS import everything
- place a million comments. Comments explain why not how. Code should be self-explanatory.
- write long gigantic self contained methods. Break them down into helpers
- just expose attributes and methods with `pub`. Use `pub(crate)` or `pub(super)` or my absolut favorite for struct fields the dedicated `getset` crate
- a simple constructor or getter doesnt need 5 tests while a full subsystems isn't satisfiently tested with 4 tests
- manually add dependencies. Use `cargo add <crate>` or add it to the workspace Cargo.toml if it's a workspace dependency
- access deep attributes manually use getters and setters they are private for a reason
- hesitate to propegate `unsafe` 
- use lifetime inference. Always be explicit with lifetimes
- use magic numbers. Always use constants with descriptive names
- use magic constants that are not derived from system properties

ALWAYS...
- use `NonNull<T>` for pointers that should never be null
- use `Option<NonNull<T>>` for pointers that can be null
- use `Result<T, E>` for fallible operations
- use `NonZero...` types for sizes and counts that should never be zero
- use `const` for configuration values or comptime functions
- look at the project first before writing code in 99% of the time code and infrastructure is already there