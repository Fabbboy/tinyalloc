# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build System

This project uses CMake with automatic build system detection:

- **Build command**: `./build.sh` - Automatically detects and uses Ninja or Make
- **Clean build**: `./build.sh --clean` - Removes build directory and rebuilds
- **Manual build**: From the build directory, run `ninja` or `make -j$(nproc)`

The build script automatically:
- Detects the best available build system (Ninja preferred, Make fallback)
- Configures CMake with appropriate generator
- Builds the static library `libtinyalloc.a`
- Uses parallel builds when possible

## Architecture

TinyAlloc is a minimal C library providing mathematical utility functions for memory allocation systems.

### Project Structure

- `include/tinyalloc/math.h` - Public API header with mathematical utility functions
- `lib/math.c` - Implementation of mathematical utilities
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