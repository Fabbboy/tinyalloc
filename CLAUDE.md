# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build System

This project uses Just with CMake:

- **Build command**: `just` or `just build` - Uses CMake with Ninja generator
- **Clean build**: `just clean` - Removes build directory and rebuilds
- **Run tests**: `just test` - Builds and runs all tests
- **Clean and rebuild**: `just rebuild` - Clean, then build

The justfile automatically:
- Uses CMake with configurable generator (default: Ninja)
- Builds the static library `libtinyalloc.a`
- Supports environment variable overrides for tools

## Architecture

TinyAlloc is a minimal C library providing mathematical utility functions for memory allocation systems.

### Project Structure

- `include/tinyalloc/math.h` - Public API header with mathematical utility functions
- `lib/math.c` - Implementation of mathematical utilities
- `vendor/mimalloc/` - Complete Microsoft mimalloc source tree for reference and integration
- `CMakeLists.txt` - Build configuration creating static library
- `build/` - CMake build directory (generated)

### Core Components

**Mathematical Utilities** (`lib/math.c`):
- Power-of-2 operations: `ta_next_power_of_2()`, `ta_prev_power_of_2()`, `ta_is_power_of_2()`
- Memory alignment functions: `ta_align_up()`, `ta_align_down()`
- Bit manipulation based implementations for performance
- Overflow protection for edge cases

The library follows a consistent naming convention with `ta_` prefix for all public functions.

### Design Philosophy

This project follows a unity build style architecture with minimal header files:

- `tinyalloc.h` - Main public API
- `tinyalloc-internal.h` - Internal interfaces 
- `tinyalloc-override.h` - Override mechanisms (if needed)

All implementation details should go in `.c` files, keeping headers lean with only essential interfaces. This approach minimizes compilation dependencies and keeps the API surface clean.

### Build Output

- Static library: `build/libtinyalloc.a`
- Compile commands: `build/compile_commands.json` (for IDE integration)
- C11 standard compliance

## Testing

- **Test framework**: Unity testing framework
- **Test command**: `just test` 
- **Manual test execution**: Run `build/tests/*` directly from build directory
- **Test location**: All tests are located in `tests/` directory
- **Coverage requirement**: Every major feature needs thorough testing

## Fast Lookup Reference

- **INDEX.md**: Contains fast-path lookups for components, functions, and concepts
  - Check INDEX.md first before searching codebase
  - Update INDEX.md when discovering new information
  - Use for quick reference on mimalloc internals, build commands, and project structure

## Important Build Notes

- Build output goes to `build/` directory
- Unity build style: Use C includes in other C files to minimize surface area
- Keep API surface small and clean