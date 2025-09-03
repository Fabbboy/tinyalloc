# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

This is a Rust workspace implementing a tiny memory allocator with the following architecture:

- **Root crate (`tinyalloc`)**: Main library that re-exports components from workspace crates
- **`tinyalloc-core`**: Core abstractions and interfaces
  - `vm.rs`: Defines the `Mapper` trait for virtual memory operations
  - `page.rs`: `Page` struct that manages memory pages with RAII semantics
  - `size.rs`: Page size utilities and alignment functions
- **`tinyalloc-sys`**: Platform-specific implementations
  - `posix.rs`: POSIX implementation of `Mapper` trait using `mmap`/`munmap`

The design follows a trait-based architecture where `Mapper` defines the interface for virtual memory operations (map, unmap, commit, decommit, protect), and platform-specific implementations like `PosixMapper` provide the actual system calls.

## Development Commands

- **Build**: `cargo build`
- **Build with release optimizations**: `cargo build --release`
- **Run tests**: `cargo test`
- **Run tests in specific crate**: `cargo test -p tinyalloc-core` or `cargo test -p tinyalloc-sys`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`

## Toolchain

This project uses Rust nightly toolchain as specified in `rust-toolchain.toml`.

## Key Dependencies

- `libc`: Low-level POSIX system calls
- `page_size`: Cross-platform page size detection
- `getset`: Automatic getter generation