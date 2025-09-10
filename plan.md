# TinyAlloc Project Plan

## Architecture Overview

TinyAlloc uses a five-tier memory hierarchy: **Global Arenas → Segments → Queue → Slabs → Heap Object**

The architecture is designed with caching and bitmap-based lookups at every layer for performance.

### Current Implementation Status

- ✅ Arena management with configurable growth
- ✅ Segment allocation within arenas
- ✅ Size class bitmap tracking
- ✅ Platform-specific configuration (32/64-bit)
- 🔄 Block allocation within segments (in progress)
- ❌ Multiple heap support
- ❌ Large allocation handling (>128KiB)

## Architecture Details

### 1. Global Arenas Layer

**Lifecycle**: NEVER destructed - grow as needed, live forever
**Purpose**: Global memory pools shared across all heaps

**Key Features**:

- **Growing Strategy**: Create new arenas as memory demand increases
- **Alternative Option (TBD)**: Dedicated arenas for specific purposes
- **Shared Resource**: All heaps allocate segments from global arenas
- **Persistent**: Once created, arenas remain until process termination
- **Caching**: Bitmap-based tracking for fast arena selection
- **Memory Management**: Page-aligned allocation with commit-on-demand

### 2. Segments Layer (Allocator Specific)

**Lifecycle**: Can be destructed when empty OR moved to another freelist
**Purpose**: Size class specific memory regions within arenas

**Key Features**:

- **Size Classes**: Each segment handles specific allocation sizes
- **Allocator Specific**: Belongs to a specific allocator instance
- **Dynamic Lifecycle**:
  - Destructible when completely empty
  - Can be appended to another segment's freelist (TBD implementation)
- **Bitmap Tracking**: Fast lookup for available segments
- **Caching**: Per-size-class caches for hot allocation paths

### 3. Queue Layer

**Purpose**: Transparent data structure for slab lifecycle management
**Responsibility**: Create and destroy slabs, manage freelist connections

**Key Features**:

- **Transparent Operation**: Invisible to higher-level allocation logic
- **Slab Creation**: Dynamically creates new slabs as needed
- **Slab Destruction**: Removes empty slabs and returns memory
- **Freelist Management**: Appends slabs to appropriate freelists
- **Allocation Unit Wrapping**: Manages smaller allocation units within slabs
- **Bitmap Optimization**: Uses bitmaps for efficient slab tracking

### 4. Slabs Layer

**Purpose**: Actual blocks of memory containing allocatable units
**Role**: The fundamental memory containers

**Key Features**:

- **Memory Blocks**: Contiguous regions of allocatable memory
- **Fixed Size Units**: All allocations within a slab are identical size
- **Bitmap Tracking**: Allocation state tracking for individual units
- **Freelist Integration**: Connected via queue layer to segment freelists
- **Caching**: Hot slab caching for performance

### 5. Heap Object Layer

**Purpose**: Uniform interface wrapping the entire allocation system
**Role**: Provides standard alloc/dealloc methods

**Key Features**:

- **Uniform Interface**: Standard `alloc()` and `dealloc()` methods
- **System Integration**: Manages interaction between all lower layers
- **Large Allocation Handling**: Direct management of large allocations
- **Caching**: Top-level allocation caches for frequent operations
- **Bitmap Coordination**: Coordinates bitmap usage across all layers

### 6. Large Allocations (Special Case)

**Management**: Handled directly by heap object, bypasses lower layers
**Implementation**: Simple linked list of memory blocks

**Key Features**:

- **Direct System Calls**: Bypasses segment/slab system entirely
- **Simple Tracking**: Basic linked list for large block management
- **Size Threshold**: Configurable cutoff (typically >128KiB)
- **Minimal Overhead**: No complex metadata or fragmentation management

## Size Class Strategy

### Current Configuration

- **Total Size Classes**: 44 (`TA_SCS = 44`)
- **Range**: Up to 128KiB cutoff
- **Problem**: Segment size limits maximum allocatable size
  - 32-bit: 2KB segment - header ≈ 1.9KB usable
  - 64-bit: 4KB segment - header ≈ 3.9KB usable

### Size Class Distribution (Needs Definition)

- **Small**: 8, 16, 24, 32, 48, 64, 80, 96, 112, 128 bytes
- **Medium**: Powers of 2 with increments up to segment limit
- **Large**: Direct system allocation for >128KiB
- **Segment Limit**: Largest size class must fit in segment after header

## Development Phases

### Phase 1: Complete Core Implementation (Current)

**Priority**: Critical

**Remaining Work**:

1. **Block Management**: Implement within-segment allocation

   - Free list management
   - Bitmap operations for block tracking
   - Block allocation/deallocation functions

2. **Size Class Definition**: Define actual 44 size classes

   - Calculate max size that fits in segment
   - Optimize distribution for real-world usage
   - Handle the 128KiB boundary issue

3. **Heap Integration**: Complete `ta_heap_t` implementation

   - Connect heaps to arena system
   - Per-heap segment tracking
   - Independent allocation contexts

4. **Large Allocation**: System allocator bypass
   - Direct malloc/free for >128KiB
   - Simple tracking structure
   - Integration with heap interface

### Phase 2: Optimization & Polish

**Priority**: High

**Components**:

- Allocation cache for hot paths
- Pointer masking for O(1) lookups
- Memory usage optimization
- Performance benchmarking

### Phase 3: Advanced Features

**Priority**: Medium

**Components**:

- Optional thread safety layer
- Global malloc/free interface
- Memory debugging tools
- Statistics collection

## Current Technical Specifications

### Arena Growth Formula (from `ta_arena_size()`)

```c
// Every batch_size arenas (default 8), multiply by 2^shift
size_t multiplier_shift = ta_clamp(arena_count / batch_size, 0, max_multiplier_shift);
size_t multiplier = 1ULL << multiplier_shift;
size_t arena_size = initial_size * multiplier;
```

**Example Growth (64-bit)**:

- Arenas 0-7: 4MB each
- Arenas 8-15: 8MB each
- Arenas 16-23: 16MB each
- Arenas 24-31: 32MB each

### Memory Layout

```
Global Arena (4MB on 64-bit, persistent):
├── Arena metadata
├── Segment 0 (4KB) [allocator A, size class X]
├── Segment 1 (4KB) [allocator B, size class Y]
├── ...
├── Segment N (4KB)
└── Unused space (uncommitted)

Segment (4KB, destructible when empty):
├── Segment header + bitmap tracking
├── Queue management metadata
├── Slab 0 → [allocation units] → freelist
├── Slab 1 → [allocation units] → freelist
└── ...

Slab (within segment):
├── Slab metadata + bitmap
├── Allocation unit 1 (size class specific)
├── Allocation unit 2 (size class specific)
├── ...
└── Freelist pointers

Heap Object:
├── Uniform alloc/dealloc interface
├── Per-layer caching systems
├── Bitmap coordination
├── Large allocation list → [direct system blocks]
└── References to global arenas
```

## Key Architectural Decisions to Finalize

### 1. Arena Specialization (TBD)

**Decision**: Global growing arenas vs. dedicated specialized arenas
**Options**:

- **Growing**: Create new arenas as needed (current direction)
- **Dedicated**: Specialized arenas for different allocation patterns
  **Impact**: Affects memory locality and fragmentation patterns

### 2. Segment Freelist Sharing (TBD)

**Decision**: How segments move between freelists when empty
**Considerations**:

- Cross-allocator segment sharing for memory efficiency
- Ownership tracking and lifecycle management
- Impact on caching and bitmap coordination

### 3. Queue Layer Implementation

**Requirements**: Design transparent slab lifecycle management
**Components**:

- Slab creation/destruction algorithms
- Freelist append/remove operations
- Integration with segment-level bitmaps
- Allocation unit wrapping strategies

### 4. Multi-Layer Caching Strategy

**Challenge**: Implement caching at every layer without conflicts
**Layers needing caches**:

- Global arena selection
- Segment allocation within arenas
- Slab allocation within segments
- Allocation unit allocation within slabs
- Heap-level allocation patterns

### 5. Bitmap Coordination

**Challenge**: Coordinate bitmaps across all five layers
**Requirements**:

- Fast lookup at each layer
- Consistent state across layer boundaries
- Efficient updates during allocation/deallocation
- Memory overhead optimization

## Testing Strategy

### Missing Tests

- Block allocation within segments
- Multiple heap instances
- Large allocation handling
- Size class distribution
- Memory commit/decommit cycles

---

**Next Immediate Steps**:

1. Resolve segment size vs size class issue
2. Implement block allocation within segments
3. Complete heap integration
4. Add large allocation bypass
