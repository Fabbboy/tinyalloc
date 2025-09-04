# MappedVector Decommit/Recommit Bug Analysis

## The Core Problem

When you clear all items from MappedVector and then push again, you get SIGSEGV. This happens because during the clearing process (popping 10k items), the shrinking logic triggers memory decommit operations, but the regrowth logic doesn't properly recommit the memory before accessing it.

## The Smoking Gun Evidence

1. **Decommit logs appear**: "Decommit AND protect result: 0" shows in test output
2. **Segfault is consistent**: Always happens when clearing completely then repushing
3. **Partial success**: Small pushes (10-1000 items) work, large ones (10k) segfault
4. **The pattern**: Works fine until growth needs to access previously decommitted memory

## Why This Happens (Memory Management Flow)

1. **Initial state**: Push 10k items → large committed memory pages
2. **Shrinking phase**: Pop items one by one → `maybe_shrink()` called each time
3. **Critical moment**: When usage drops below 25%, `shrink_active()` creates new smaller pages
4. **Memory state corruption**: Old pages get decommitted/dropped, but some memory pointers (`self.data`) still reference the decommitted regions
5. **Segfault trigger**: When growing again, `grow()` tries to copy from decommitted memory → SIGSEGV

## The Real Issue Location

The bug is in `mvec.rs` in the interaction between:

* `shrink_active()` - creates new pages, drops old ones (which get decommitted)
* `grow()` - assumes existing capacity means accessible memory, but doesn't check if memory was decommitted

## Key Code Locations to Fix

1. **`grow()` function**: Line \~70-110 - needs to verify memory is committed before using it
2. **`shrink_active()` function**: Line \~120-145 - might be leaving `self.data` pointing to decommitted memory
3. **State consistency**: The relationship between `self.data`, `self.capacity`, `self.active_elements` and actual committed memory state

## Current Shrinking Logic (The Culprit)

```rust
// In maybe_shrink() - called on every pop()
if usage_ratio < MVEC_DECOMMIT_THRESHOLD { // 0.25 = 25%
    let needed_elements = (self.len * 2).max(self.initial_capacity());
    if needed_elements < self.active_elements {
        self.shrink_active(needed_elements)?; // Creates new page, drops old one
    }
}
```

The problem: `shrink_active()` creates completely new pages and drops old ones. The dropping triggers decommit, but `self.data` might still point to the decommitted memory region.

## The Fix Strategy

**Don't avoid the problem, fix it properly:**

1. **In `grow()`**: Before assuming capacity is sufficient, check if the underlying page is committed
2. **Add recommit logic**: If memory exists but is decommitted, recommit it before use
3. **Fix pointer consistency**: Ensure `self.data` always points to committed, accessible memory

⚠️ **Important additional constraint:**
The **master page** can and should be deallocated **only if it is actually no longer used**. The `grow()` method must always ensure that the relevant page (whether it is page 0, page 10, or even page 1,000,000) is mapped or committed before any write.

## Test Case That Must Pass

```rust
fn test_clear_and_repush_behavior() {
    let mut mvec: MappedVector<i32, 1> = MappedVector::new(MAPPER);
    
    for i in 0..10000 { mvec.push(i).unwrap(); }
    for _ in 0..10000 { mvec.pop(); }  // This triggers decommit
    assert_eq!(mvec.len(), 0);
    
    // This MUST work without segfault:
    for i in 0..10000 { mvec.push(i).unwrap(); }
}
```

## What NOT To Do

* Don't change the test to avoid the problem
* Don't disable decommit behavior (that's the whole point of memory efficiency)
* Don't create workarounds that mask the real issue

## The Correct Approach

The decommit behavior is GOOD - it returns memory to the OS. The bug is that we're not properly recommitting when we need the memory again. Fix the recommit logic, don't break the decommit logic.

## Implementation Notes for Future Me

* The `Page` type already has working `decommit()` and `commit()` methods
* The logs prove decommit/commit operations work correctly
* The issue is in the MappedVector's state management, not in the underlying Page operations
* Look at the `grow()` function first - that's where the segfault happens during memory access

---

## Additional Notes

### Low Priority

The `MappedQueue` implementation currently never shrinks. There is no shrinking logic in the internal heap. **This is not considered a bug report yet, just a note** for future optimization. Current priority is low.

