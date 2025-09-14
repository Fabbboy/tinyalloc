# TinyAlloc MVP Roadmap

> **Design Inspiration**: This project is inspired by [mimalloc](https://github.com/microsoft/mimalloc) and follows its core design principles including segmented memory management, size classes, and free list organization. The architecture maintains mimalloc's approach to arena-based allocation while adapting it to Rust's ownership model and safety guarantees.

## Current Implementation Status

### ✅ Completed Infrastructure (5/5 Core Crates)
- **tinyalloc-sys**: POSIX memory mapping with `PosixMapper`, `Region`, error handling
- **tinyalloc-list**: Intrusive doubly-linked lists with `List<T>`, `Link<T>`, `HasLink<T>`  
- **tinyalloc-bitmap**: Generic bitmap operations over word types with scanning/bulk ops
- **tinyalloc-array**: Stack arrays with bounds checking and deref traits
- **tinyalloc-alloc**: Core allocation primitives - `Arena`, `Segment`, `Queue`, `Class`

### 🔄 Partially Complete Components
- **Heap structure**: `crates/tinyalloc-alloc/src/heap.rs:8-34` - struct defined, allocation methods stubbed
- **Large allocations**: `crates/tinyalloc-alloc/src/large.rs:12-51` - `Large<M>` struct with region management
- **Size classes**: All 32 classes defined with >70% space utilization verified

## MVP Requirements & Implementation Tasks

### 1. **Global Mapper Integration - CRITICAL**
**Status**: 🔴 Missing - All components hardcoded to generic mapper  
**Requirements**: Complete mapper agnosticism across entire codebase

**Tasks**:
- [ ] Refactor all components to use global `&'static dyn Mapper` reference
- [ ] Remove generic `<M: Mapper>` parameters from `Heap`, `Arena`, `Segment`, `Large`
- [ ] Implement mapper selection at startup (POSIX vs Windows)
- [ ] Global mapper initialization in `static_.rs`

### 2. **Windows Mapper Implementation**
**Status**: 🔴 Missing - POSIX only  
**Requirements**: Full Windows support using **winapi crate** (NOT windows-sys)

**Files**: `crates/tinyalloc-sys/src/`

**Tasks**:
- [ ] Add `winapi` crate dependency (workspace level)
- [ ] Implement `WindowsMapper` struct in new module
- [ ] VirtualAlloc/VirtualFree operations with proper error mapping
- [ ] VirtualProtect for memory protection
- [ ] VirtualAlloc with MEM_RESET for decommit operations
- [ ] Conditional compilation for Windows/POSIX mapper selection

### 3. **Core Heap Allocation API** 
**Status**: 🔴 Critical - Missing Implementation  
**Files**: `crates/tinyalloc-alloc/src/heap.rs:28-33`

**Tasks**:
- [ ] Implement `Heap::allocate(layout: Layout) -> Option<NonNull<[u8]>>`
- [ ] Implement `Heap::deallocate(ptr: NonNull<u8>, layout: Layout)`
- [ ] Size class resolution and Queue management
- [ ] Integration with global mapper for large allocations

### 4. **Main Library Interface**
**Status**: 🔴 Critical - Completely Missing  
**Files**: `src/lib.rs:1` (currently only contains FFI comment)

**Tasks**:
- [ ] Re-export core types without generic mapper parameters
- [ ] Implement heap factory functions using global mapper
- [ ] Proper `Layout` handling with alignment validation
- [ ] Cross-platform initialization functions

### 5. **Error Handling & Layout Validation**
**Status**: 🟡 Partial - Scattered across crates

**Tasks**:
- [ ] Unified error type covering Windows and POSIX failures
- [ ] Layout validation: size limits, alignment requirements  
- [ ] Platform-specific error code mapping
- [ ] Remove all `todo!()` and `unwrap()` calls

### 6. **Arena Growth & Management**
**Status**: 🟡 Partial - Static management exists

**Tasks**:
- [ ] Dynamic arena allocation using global mapper
- [ ] Cross-platform decommit mechanism (MADV_DONTNEED/MEM_RESET)
- [ ] Multi-heap arena sharing
- [ ] Exponential growth strategy

### 7. **System Value Derivation**
**Status**: 🟡 Partial - Magic constants need replacement

**Tasks**:
- [ ] Windows: GetSystemInfo for page size
- [ ] POSIX: sysconf(_SC_PAGESIZE) 
- [ ] Cache line size detection (both platforms)
- [ ] Replace all magic constants with system-derived values

### 8. **Multi-Heap Support & Thread Safety**
**Status**: 🔴 Missing

**Tasks**:
- [ ] Multiple `Heap` instances sharing global arenas
- [ ] `Send` + `Sync` implementations where appropriate
- [ ] User-managed heap lifecycle (no internal locks)

### 9. **Performance Validation**
**Status**: 🔴 Missing

**Tasks**:
- [ ] Cross-platform benchmark suite vs ptmalloc/Windows heap
- [ ] Memory utilization validation (>70% maintained)
- [ ] Allocation latency measurements

## Implementation Priority Order

### Phase 1: Mapper Agnosticism (Week 1)
1. **CRITICAL**: Remove all generic `<M: Mapper>` parameters 
2. Implement global mapper infrastructure in `static_.rs`
3. Refactor `Heap`, `Arena`, `Large` to use global mapper reference

### Phase 2: Windows Support (Week 2)  
1. Add `winapi` workspace dependency
2. Implement complete `WindowsMapper` with VirtualAlloc/VirtualFree
3. Conditional compilation and cross-platform testing

### Phase 3: Core Functionality (Week 3)
1. Complete `Heap::allocate()` and `Heap::deallocate()` 
2. Main library interface without generic parameters
3. Cross-platform arena growth and decommit

### Phase 4: Production Readiness (Week 4)
1. System value derivation (both platforms)
2. Multi-heap support and thread safety markers
3. Cross-platform benchmark suite

## Success Criteria
- [ ] **Zero generic mapper parameters** - complete mapper agnosticism
- [ ] **Windows + POSIX support** using winapi crate (not windows-sys)
- [ ] Global mapper selection at startup
- [ ] All allocation paths functional on both platforms
- [ ] Benchmarks showing competitive performance vs platform allocators
- [ ] >70% space utilization maintained cross-platform

## Out of Scope (Post-MVP)
- Thread local caching or global heap optimizations
- Global allocator trait implementation  
- FFI bindings (as noted in `src/lib.rs:1`)
- Advanced large object optimizations
- Internal synchronization primitives (user-managed threading)

## Platform Requirements
- **Windows**: winapi crate for VirtualAlloc/VirtualFree/VirtualProtect
- **POSIX**: libc for mmap/munmap/mprotect/madvise
- **Both**: System-derived page sizes and cache line detection

## Terms
- **decommit**: Return physical pages to OS while preserving virtual address reservation (MADV_DONTNEED/MEM_RESET, not munmap/VirtualFree)