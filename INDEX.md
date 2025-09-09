# TinyAlloc Project Index

This file serves as a fast-path lookup for agents and developers to quickly find definitions, implementations, and key concepts in the project.

## Core Components

### Mathematical Utilities
- **Location**: `lib/math.c`, `include/tinyalloc/math.h`
- **Functions**:
  - `ta_next_power_of_2()` - Get next power of 2
  - `ta_prev_power_of_2()` - Get previous power of 2  
  - `ta_is_power_of_2()` - Check if value is power of 2
  - `ta_align_up()` - Align value up to boundary
  - `ta_align_down()` - Align value down to boundary
- **Implementation**: Bit manipulation based for performance
- **Features**: Overflow protection for edge cases

### Build System
- **Primary**: Just with CMake (`justfile`)
- **Commands**: 
  - `just` / `just build` - Build with Ninja
  - `just clean` - Clean rebuild
  - `just test` - Run tests
  - `just rebuild` - Clean and rebuild
- **Output**: `build/libtinyalloc.a` (static library)

### Testing
- **Framework**: Unity testing framework
- **Location**: `tests/` directory
- **Execution**: `just test` or run `build/tests/*` directly

## Vendor Dependencies

### mimalloc
- **Location**: `vendor/mimalloc/`
- **Purpose**: Reference implementation for advanced memory allocation
- **Key Concepts**:
  - **Arenas**: Fixed OS memory areas (32MiB), multi-thread shared, atomic bitmap managed
  - **Segments**: Large memory blocks from arenas/OS, thread-local, contain pages
  - **Pages**: 64KiB-512KiB allocation units, size-class specific
  - **Hierarchy**: Arena → Segment → Pages → Blocks

#### mimalloc Key Files
- `vendor/mimalloc/include/mimalloc/types.h` - Core type definitions
- `vendor/mimalloc/src/arena.c` - Arena management (thread-safe large block allocation)
- `vendor/mimalloc/src/segment.c` - Segment management (thread-local containers)
- `vendor/mimalloc/src/page.c` - Page management
- `vendor/mimalloc/src/alloc.c` - Main allocation routines
- `vendor/mimalloc/src/free.c` - Deallocation routines

#### mimalloc Architecture Notes
- **tcache**: Thread-local caching (look in `src/heap.c`, `src/page-queue.c`)
- **Free list sharding**: Per-page free lists for reduced contention
- **Multi-sharding**: Thread-local + concurrent free lists per page
- **Arena allocation**: Atomic bitmap allocation from fixed OS memory areas
- **Segment ownership**: Thread-local segments with cross-thread freeing support

## Project Structure
```
include/tinyalloc/     - Public API headers
lib/                   - Implementation files
vendor/mimalloc/       - Microsoft mimalloc source tree
tests/                 - Unity-based test suite
build/                 - CMake build output
CMakeLists.txt         - Build configuration
justfile               - Build commands
CLAUDE.md              - Agent instructions
INDEX.md               - This fast-lookup file
```

## Quick Reference Patterns

### Finding Implementations
- Math functions → `lib/math.c`
- Public APIs → `include/tinyalloc/*.h` 
- Build config → `CMakeLists.txt`
- Tests → `tests/*.c`
- mimalloc internals → `vendor/mimalloc/src/*.c`

### Common Queries
- "How does X work?" → Check this INDEX first, then relevant source files
- "Where is Y defined?" → Use grep/search tools, update INDEX with findings
- "Build failing?" → Check `justfile`, `CMakeLists.txt`
- "Test issues?" → Look in `tests/` directory

## Update Instructions

**Agents should update this INDEX.md when:**
- Adding new functions or components
- Discovering important implementation details
- Finding answers to complex queries
- Locating key concepts in vendor code

**Format for updates:**
```markdown
### New Component
- **Location**: file/path
- **Purpose**: brief description  
- **Key Details**: important notes
```

---
*Last updated: Initial creation*