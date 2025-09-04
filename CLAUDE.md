# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

DO NOT USE VEC OR BOX THIS IS AN INDEPENDENT MEMORY ALLOCATOR THAT DOES **NOT** RELY ON MALLOC/FREE

This is a Rust workspace implementing a tiny memory allocator with the following architecture:

- **Root crate (`tinyalloc`)**: Main library that re-exports components from workspace crates
- **`tinyalloc-sys`**: Low-level system abstractions and platform-specific implementations
  - `vm.rs`: Defines the `Mapper` trait for virtual memory operations
  - `page.rs`: `Page` struct that manages memory pages with RAII semantics
  - `size.rs`: Page size utilities and alignment functions
  - `system/`: Platform-specific implementations
    - `posix.rs`: POSIX implementation using `mmap`/`munmap`
    - `windows.rs`: Windows implementation using Win32 APIs
- **`tinyalloc-bitmap`**: Bitmap data structure for tracking allocation state
  - `bitmap.rs`: Core bitmap implementation
  - `numeric.rs`: Numeric utilities for bitmap operations
  - `error.rs`: Error types for bitmap operations
- **`tinyalloc-alloc`**: High-level allocator implementation

The design follows a trait-based architecture where `Mapper` defines the interface for virtual memory operations (map, unmap, commit, decommit, protect), and platform-specific implementations provide the actual system calls.

## Development Commands

- **Build**: `cargo build`
- **Build with release optimizations**: `cargo build --release`
- **Run tests**: `cargo test`
- **Run tests in specific crate**: `cargo test -p tinyalloc-sys`, `cargo test -p tinyalloc-bitmap`, or `cargo test -p tinyalloc-alloc`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`

## Toolchain

This project uses Rust nightly toolchain as specified in `rust-toolchain.toml`.

## Key Dependencies

- `libc`: Low-level POSIX system calls
- `windows-sys`: Windows system API bindings
- `page_size`: Cross-platform page size detection
- `getset`: Automatic getter generation
- `enumset`: Efficient enum set implementation