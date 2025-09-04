# TinyAlloc Roadmap

## 1. Title & Scope
This document is the authoritative project plan for the TinyAlloc memory allocator. It is maintained by AI agents and focuses exclusively on design and planning; no code changes are included here.

## 2. Executive Summary
Current status: foundational crates for virtual memory, basic containers, and size classes exist, but an integrated allocator pipeline is not yet implemented.

High-level architecture:
- **System/Page Allocator** – OS-facing, each map/unmap/commit/decommit is a direct syscall.
- **Chunk/Region Manager** – reserves large virtual memory regions (e.g., 1 MB–1 GB) without immediately committing pages.
- **User Allocator** – sits on top, uses predefined size classes; per-allocation header currently zero bytes but may be introduced.

## 3. Design Constraints (Non-Negotiable)
- Single-threaded MVP1 only; no NUMA, thread-local caches, atomics, or locks.
- A global heap is provided for default `malloc`/`free`; additional heaps can be created and are fully isolated.
- No dynamic data structures (allocator-independent). Internal structures should use singly linked lists.
- Existing size classes are fixed; only optional headers may be added.
- C FFI must expose `malloc`, `free`, `realloc`, `calloc`, plus APIs to create/destroy heaps and allocate/free within them.

## 4. Repository Map
- `/src/lib.rs` – re-exports system modules for external consumption.
- `/crates/tinyalloc-sys/src/vm.rs` – `Mapper` trait describing virtual memory operations【F:crates/tinyalloc-sys/src/vm.rs†L1-L29】
- `/crates/tinyalloc-sys/src/page.rs` – `Page` struct with RAII mapping/commit/protect semantics【F:crates/tinyalloc-sys/src/page.rs†L1-L93】
- `/crates/tinyalloc-sys/src/system/posix.rs` – POSIX implementation of `Mapper` using `mmap`/`mprotect`/`munmap`.
- `/crates/tinyalloc-sys/src/system/windows.rs` – Windows implementation via `VirtualAlloc`/`VirtualFree`.
- `/crates/tinyalloc-sys/src/size.rs` – page size detection and alignment helpers【F:crates/tinyalloc-sys/src/size.rs†L1-L12】
- `/crates/tinyalloc-bitmap/src/bitmap.rs` – bitmap structure for allocation tracking.
- `/crates/tinyalloc-bitmap/src/numeric.rs` – numeric trait helpers for bitmaps.
- `/crates/tinyalloc-alloc/src/classes.rs` – size class definitions and lookup【F:crates/tinyalloc-alloc/src/classes.rs†L1-L100】
- `/crates/tinyalloc-container/src/mvec.rs` – `MappedVector` backed by virtual pages【F:crates/tinyalloc-container/src/mvec.rs†L1-L113】
- `/crates/tinyalloc-container/src/mqueue.rs` – `MappedQueue` implementing a singly linked list over mapped pages【F:crates/tinyalloc-container/src/mqueue.rs†L1-L119】

## 5. Current Issues & Risks
- **Size class cutoff test failure**  
  - Evidence/Location: failing `test_find_size_class_cutoff` in `/crates/tinyalloc-alloc/src/classes.rs`【F:crates/tinyalloc-alloc/src/classes.rs†L150-L155】
  - Impact: incorrect size-class lookup at boundary; could misallocate large requests.
  - Status: Known (test reproduces).

## 6. Unused / Defined-But-Unused Inventory
- `crates/tinyalloc-container/src/mqueue.rs` – `QUEUE_NODE_ALIGNMENT` constant【F:crates/tinyalloc-container/src/mqueue.rs†L16-L17】
- `crates/tinyalloc-container/src/mqueue.rs` – `QUEUE_PAGE_MULTIPLIER` constant【F:crates/tinyalloc-container/src/mqueue.rs†L16-L17】

## 7. Edit Policy for This File
Future AI agents may update this roadmap to:
- Add/remove issues or adjust their status.
- Move items between MVP phases.
- Add/remove entries in the Unused Inventory as code evolves.
- Update architecture notes without violating the design constraints above.

## 8. MVP1: Scope, Criteria, and Plan
### Scope
- System/Page Allocator implemented for map/unmap/commit/decommit.
- Chunk/Region Manager reserving large VM ranges (configurable, e.g., 100 MB–1 GB) without committing.
- Allocator within Region allocator handing out Spans (prefered word: slab, slice, slot) for fixed size classes.
- User Allocator layered above, using existing size classes; optional small header allowed if needed (wastes potentially memory on boundary hits).
- Global heap plus APIs to create/destroy isolated heaps; default `malloc`/`free` target the global heap.
- C bindings exposing `malloc`, `free`, `realloc`, `calloc`, and heap management; delivered as static or dynamic library with header.
- No threading features or NUMA; correctness and minimal fragmentation only.

### Out-of-scope (MVP2+)
- Synchronization primitives, thread-local caches, NUMA awareness.
- Background scavenging, advanced telemetry, tunables, high-level abstractions.

### Success Criteria
- A C program can link against the library, perform large allocation/free cycles without leaks or prolonged stalls.
- After frees, virtual memory usage returns to reasonable levels without severe fragmentation.
- Memory is reused sensibly; allocator avoids egregious waste.

### Validation Plan
- **Pattern A:** Many small allocations/frees in tight loops → expect size-class slots to recycle; committed pages stable.
- **Pattern B:** Mixed small/large allocations with intermittent frees → expect region manager to commit/decommit appropriately and reuse freed slots.
- **Pattern C:** Large burst allocations followed by complete free and reallocation → expect regions reserved once, pages decommitted then recommitted without leaks.
- Metrics to observe: bytes allocated, bytes reserved, bytes committed, internal free-list lengths, external address space consumption.

## 9. MVP2: Scope Preview
- Thread safety via locks or lock-free structures.
- Thread-local caches and NUMA-aware allocation.
- Advanced defragmentation and scavenging strategies.
- Expanded telemetry and configurable tunables.
- Broader FFI surface.

## 10. Implementation Blueprint (No Code)
- **Dataflow:** API call → size class selection → slab/run/slot acquisition → region sub-allocation → system map/commit as needed.
- **Internal Structures:** Use singly linked lists for free lists, partial/full slab lists, and region lists; no dynamic containers.
- **Region Lifetime:** reserve large region → commit pages on demand → decommit when freed → optional unmap when region idle.
- **Configuration Knobs:** compile-time constants for default region reservation size (e.g., 100 MB or 1 GB), assumed page size, optional per-allocation header size.
- **Multiple Heaps:** each heap maintains its own metadata and free lists; no sharing across heaps to ensure isolation.

## 11. Open Questions / Unknowns
- Optimal default region reservation size (100 MB vs 1 GB) for diverse workloads.
- Whether per-allocation headers are necessary for free-list linking and how large they should be.
- Exact layout and metadata required for a custom heap structure.
- Strategy for returning fully unused regions to the OS (thresholds for unmap vs. decommit).

## 12. Appendix
### Glossary
- **Region:** large reserved virtual memory range from the OS.
- **Slab/Run:** subdivision of a region used for allocations of a particular size class.
- **Size class:** predefined allocation size bucket.
- **Commit vs Reserve:** reserve allocates address space without physical memory; commit assigns physical memory.
- **Decommit vs Unmap:** decommit releases physical memory but keeps address space; unmap releases both.

### AI-Updated Files
- `/ROADMAP.md`
- `/CLAUDE.md`
