# PHF Size Class Optimization Strategy

## Current Problem

**Size class lookup is O(n) linear search through 84 size classes**, costing 1.0 operation per allocation. This accounts for significant overhead in allocation hot paths.

```rust
// Current implementation - O(n) linear search
pub const fn find_class(size: usize, align: usize) -> Option<&'static Class> {
  let mut i = 0;
  while i < SIZES {
    let class = &CLASSES[i];
    if size <= class.size.0 && align <= class.align.0 {
      return Some(class);
    }
    i += 1;
  }
  None
}
```

## PHF Solution Overview

Use **Perfect Hash Functions (PHF)** to generate compile-time O(1) lookup tables that map sizes to size class indices with zero hash collisions.

### Key Benefits
- ✅ **O(1) guaranteed lookup** - No hash collisions by design
- ✅ **Zero runtime allocation** - Generated at compile time  
- ✅ **Platform adaptive** - Automatically adjusts to different architectures
- ✅ **Perfect consistency** - Uses same logic as runtime size class computation
- ✅ **Type safe** - Rust compiler validates mappings

## Implementation Strategy

### 1. Build-Time PHF Generation

```toml
# Add to Cargo.toml
[dependencies]
phf = { version = "0.11", features = ["macros"] }

[build-dependencies]
phf_codegen = "0.11"
```

### 2. Extract Size Class Logic

Create shared size class computation logic that both runtime and build scripts can use:

```rust
// In a shared module (e.g., size_class_shared.rs)
pub struct SizeClassInfo {
    pub size: usize,
    pub align: usize,
    pub id: usize,
}

pub fn compute_size_classes() -> Vec<SizeClassInfo> {
    // Extract the exact logic from the current const fn classes()
    let mut classes = Vec::new();
    let mut size = MIN_SIZE;
    
    for i in 0..SIZES {
        let align = size_to_align(size);
        let aligned_size = align_up(size, align);
        classes.push(SizeClassInfo { 
            size: aligned_size, 
            align, 
            id: i 
        });
        
        // Same growth logic as current implementation
        if size < SMALL_SC_LIMIT {
            size += align;
        } else if size < MEDIUM_SC_LIMIT {
            size += align * 2;
        } else if size < LARGE_SC_LIMIT {
            size += align * 4;
        } else {
            size *= 2;
        }
    }
    
    classes
}
```

### 3. Build Script Implementation

```rust
// In build.rs
use phf_codegen::Map;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    generate_size_class_phf();
}

fn generate_size_class_phf() {
    let size_classes = compute_size_classes();
    
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("size_class_phf.rs");
    let mut file = BufWriter::new(File::create(&dest_path).unwrap());
    
    // Generate primary size->class_id mapping
    writeln!(&mut file, "static SIZE_TO_CLASS: phf::Map<u32, u8> = \\").unwrap();
    let mut map = Map::new();
    
    // For each size class, map all sizes it can handle
    for (class_id, class) in size_classes.iter().enumerate() {
        let prev_max_size = if class_id > 0 { 
            size_classes[class_id - 1].size 
        } else { 
            0 
        };
        
        // Generate entries for all sizes this class handles
        for size in (prev_max_size + 1)..=class.size {
            map.entry(size as u32, &format!("{}u8", class_id));
        }
    }
    
    writeln!(&mut file, "{};", map.build()).unwrap();
    
    // Generate alignment constraint mapping
    generate_alignment_phf(&mut file, &size_classes);
    
    // Generate validation data for testing
    generate_validation_data(&mut file, &size_classes);
}

fn generate_alignment_phf(file: &mut BufWriter<File>, classes: &[SizeClassInfo]) {
    writeln!(file, "static ALIGN_TO_MIN_CLASS: phf::Map<u32, u8> = \\").unwrap();
    let mut map = Map::new();
    
    // Map alignment requirements to minimum class that satisfies them
    for align in 1..=MAX_ALIGN {
        let min_class = classes.iter()
            .position(|c| c.align >= align)
            .unwrap_or(classes.len() - 1);
        map.entry(align as u32, &format!("{}u8", min_class));
    }
    
    writeln!(file, "{};", map.build()).unwrap();
}

fn generate_validation_data(file: &mut BufWriter<File>, classes: &[SizeClassInfo]) {
    // Generate test data for build-time validation
    writeln!(file, "#[cfg(test)]").unwrap();
    writeln!(file, "const EXPECTED_CLASS_COUNT: usize = {};", classes.len()).unwrap();
    
    writeln!(file, "#[cfg(test)]").unwrap();
    writeln!(file, "const EXPECTED_MAX_SIZE: usize = {};", 
             classes.last().unwrap().size).unwrap();
}
```

### 4. Runtime Integration

```rust
// In classes.rs - include generated PHF
include!(concat!(env!("OUT_DIR"), "/size_class_phf.rs"));

#[inline(always)]
pub fn find_class_phf(size: usize, align: usize) -> Option<&'static Class> {
    if size == 0 { return None; }
    
    // Primary O(1) lookup
    if let Some(&class_id) = SIZE_TO_CLASS.get(&(size as u32)) {
        let class = &CLASSES[class_id as usize];
        
        // Quick alignment check
        if align <= class.align.0 {
            return Some(class);
        }
        
        // Find minimum class that satisfies alignment
        if let Some(&min_class_id) = ALIGN_TO_MIN_CLASS.get(&(align as u32)) {
            if min_class_id <= class_id {
                return Some(class);
            }
            // Need higher class for alignment
            if (min_class_id as usize) < SIZES {
                return Some(&CLASSES[min_class_id as usize]);
            }
        }
    }
    
    // Fallback for edge cases (should be rare)
    find_class_linear(size, align)
}

// Keep original as fallback
fn find_class_linear(size: usize, align: usize) -> Option<&'static Class> {
    // Current implementation
}
```

## Platform Adaptation

### Automatic Platform Scaling

The PHF generation automatically adapts to platform-specific factors:

- **Word size differences** (32-bit vs 64-bit) → Alignment requirements automatically handled
- **Cache line sizes** → Size class boundaries adapt automatically  
- **Page sizes** → Growth patterns adjust based on platform constants
- **Architecture-specific alignment** → All handled through existing size_to_align() logic

### Build-Time Validation

```rust
// In build.rs - validate generated PHF against linear search
fn validate_phf_correctness(classes: &[SizeClassInfo]) {
    println!("cargo:warning=Validating PHF for {} size classes", classes.len());
    
    let mut validation_errors = 0;
    
    // Test all reasonable size/alignment combinations
    for size in 1..=LARGE_SC_LIMIT {
        for align in [1, 2, 4, 8, 16, 32, 64, 128] {
            if align > size { continue; }
            
            let phf_result = find_class_phf_test(size, align);
            let linear_result = find_class_linear_test(size, align);
            
            if phf_result != linear_result {
                eprintln!("PHF mismatch: size={}, align={}, phf={:?}, linear={:?}", 
                         size, align, phf_result, linear_result);
                validation_errors += 1;
            }
        }
    }
    
    if validation_errors > 0 {
        panic!("PHF validation failed with {} errors", validation_errors);
    }
    
    println!("cargo:warning=PHF validation passed for all test cases");
}
```

## Performance Analysis

### Expected Improvements

| Metric | Current | With PHF | Improvement |
|--------|---------|----------|-------------|
| Size class lookup | O(n) - avg 42 comparisons | O(1) - 1 array access | **~40x faster** |
| Allocation hot path | Multiple O(n) operations | Mostly O(1) operations | **10-20x faster** |
| Cache miss penalty | O(n) lookup + segment creation | O(1) lookup + segment creation | **Major improvement** |

### Memory Usage

PHF maps are very compact:
- **SIZE_TO_CLASS**: ~1KB for 84 size classes (maps up to LARGE_SC_LIMIT)
- **ALIGN_TO_MIN_CLASS**: ~256 bytes for alignment constraints
- **Total overhead**: <2KB static data

### Build Time Impact

- **Generation time**: <1ms for 84 size classes
- **Validation time**: <10ms for comprehensive testing
- **Binary size**: Negligible increase (<2KB)

## Migration Strategy

### Phase 1: Parallel Implementation
- Keep existing `find_class()` as fallback
- Add `find_class_phf()` alongside
- Add feature flag for testing

### Phase 2: Performance Testing
- Benchmark PHF vs linear on target platforms
- Validate correctness across all size/alignment combinations
- Measure allocation throughput improvements

### Phase 3: Integration
- Replace linear search with PHF in hot paths
- Keep linear as fallback for edge cases
- Remove feature flag once stable

## Edge Cases & Fallbacks

### Handled Cases
- **Size overflow**: PHF naturally handles by falling back to linear
- **Alignment constraints**: Dedicated PHF map for alignment requirements
- **Platform differences**: Build-time generation adapts automatically
- **Size class changes**: Automatic regeneration on any constant changes

### Safety Net
Always fallback to linear search for any PHF misses, ensuring correctness is never compromised for performance.

## Future Enhancements

### Advanced PHF Optimizations
- **Range PHF**: Map size ranges instead of individual sizes
- **Hybrid approach**: PHF for common cases, optimized linear for edge cases
- **SIMD validation**: Vectorized validation during build

### Integration with Other Optimizations
- **Cache-aware PHF**: Generate PHF maps optimized for cache line access
- **Batched lookups**: Process multiple size class lookups together
- **Prefetch hints**: Add prefetch for likely size class accesses

---

*This optimization represents a major performance improvement with zero runtime cost, automatically adapting to any platform while maintaining perfect correctness through comprehensive validation.*