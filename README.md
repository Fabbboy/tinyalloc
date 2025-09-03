# TinyAlloc

A minimalist memory allocator designed for simplicity and cross-platform compatibility.

## Roadmap

### Phase 1: Foundation ✓
- [x] Virtual memory abstraction layer
- [x] Platform-specific memory mapping (POSIX, Windows)
- [x] Page management with RAII semantics
- [x] Basic utilities (alignment, page size detection)

## Design Principles

- **Simplicity**: Minimal code complexity while maintaining functionality
- **Portability**: Works across different operating systems and architectures  
- **Modularity**: Clean separation between platform code and algorithms
- **Interoperability**: FFI-first design for multi-language support
- **Performance**: Competitive with system allocators for common use cases

## Architecture Goals

- Language-agnostic core with thin language bindings
- Pluggable allocation strategies
- Zero-overhead abstractions where possible
- Minimal external dependencies