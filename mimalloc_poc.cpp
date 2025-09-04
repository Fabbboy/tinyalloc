/*
 * Mimalloc-inspired PoC: Large Memory Container Management
 * 
 * This demonstrates how mimalloc manages large memory containers (segments) 
 * without using malloc/free. Key insights from mimalloc source:
 * 
 * 1. **mi_segment_t** (types.h:465-500) - The biggest data structure after mmap
 * 2. **_mi_arena_alloc_aligned** (arena.c:855) - OS abstraction layer that calls mmap
 * 3. **Segments are 32MiB** on 64-bit systems (types.h:192)
 * 4. **Metadata stored in-band**: segment metadata at start of each segment
 * 5. **Linked list built using metadata area**: `next` pointer at offset 0 (types.h:479)
 * 6. **Bootstrap problem solved**: First segment metadata lives at segment start
 */

#include <iostream>
#include <cstdint>
#include <cstring>
#include <sys/mman.h>
#include <unistd.h>
#include <cassert>

// Mimalloc-inspired constants
const size_t SEGMENT_SIZE = 32 * 1024 * 1024; // 32 MiB
const size_t SEGMENT_ALIGN = SEGMENT_SIZE;     // Segments are aligned to their size
const size_t PAGE_SIZE = 4096;

// Forward declaration
struct Segment;

// Mimalloc-style memory ID for tracking provenance
struct MemId {
    void* base;
    size_t size;
    bool initially_committed;
    bool initially_zero;
    bool is_pinned;
};

// Large memory container (inspired by mi_segment_t)
// This is the "biggest data structure after mmap" that actually calls OS abstraction
struct Segment {
    // CRITICAL: `next` must be FIRST field (offset 0) for linked list management
    // This is the key insight - mimalloc stores the linked list pointer 
    // at the very start of each segment's metadata area
    Segment* next;                    // Next segment in free list
    
    // Memory management metadata  
    MemId memid;                     // Track how this segment was allocated
    bool allow_decommit;             // Can we decommit parts of this segment?
    bool allow_purge;                // Can we purge (reset/decommit) this segment?
    size_t segment_size;             // Total size of this segment
    
    // Commit tracking (simplified - mimalloc uses bitmasks)
    uint64_t commit_mask;            // Which parts are committed (simplified)
    uint64_t purge_mask;             // Which parts can be purged
    
    // Thread ownership and validation
    uint64_t thread_id;              // Which thread owns this segment
    uintptr_t cookie;                // Security cookie for validation
    
    // Usage tracking
    size_t used;                     // Number of pages in use
    size_t abandoned;                // Number of abandoned pages
    
    // Data area starts after metadata
    // In real mimalloc, this contains mi_slice_t array for page management
    uint8_t data[];                  // Flexible array member - actual allocatable space
};

// Global segment cache - simple linked list
struct SegmentCache {
    Segment* free_list;              // Head of free segment list
    size_t count;                    // Number of cached segments
    size_t peak_count;               // Peak number of segments
};

static SegmentCache g_segment_cache = {nullptr, 0, 0};

// OS abstraction layer - this is where actual mmap happens
// Inspired by _mi_arena_alloc_aligned and mi_os_prim_alloc
void* os_alloc_aligned(size_t size, size_t alignment, bool commit, MemId* memid) {
    // Ensure size is properly aligned
    size = (size + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
    
    // Use mmap to get aligned memory (Linux/Unix-style)
    int flags = MAP_PRIVATE | MAP_ANONYMOUS;
    if (!commit) {
        flags |= MAP_NORESERVE; // Don't commit immediately
    }
    
    void* ptr = mmap(nullptr, size, 
                     commit ? (PROT_READ | PROT_WRITE) : PROT_NONE,
                     flags, -1, 0);
    
    if (ptr == MAP_FAILED) {
        std::cerr << "Failed to allocate " << size << " bytes aligned to " << alignment << std::endl;
        return nullptr;
    }
    
    // Check alignment (mmap usually gives page-aligned memory)
    if (reinterpret_cast<uintptr_t>(ptr) % alignment != 0) {
        std::cerr << "Warning: allocated memory not properly aligned" << std::endl;
        munmap(ptr, size);
        return nullptr;
    }
    
    // Fill memory ID
    memid->base = ptr;
    memid->size = size;
    memid->initially_committed = commit;
    memid->initially_zero = true;  // mmap gives zero-filled memory
    memid->is_pinned = false;
    
    std::cout << "Allocated segment: " << ptr << " size: " << size << " committed: " << commit << std::endl;
    return ptr;
}

// Free OS memory
void os_free(void* ptr, size_t size, const MemId& memid) {
    if (ptr && size > 0) {
        munmap(ptr, size);
        std::cout << "Freed segment: " << ptr << " size: " << size << std::endl;
    }
}

// Allocate a new segment from OS - this is the "biggest data structure after mmap"
Segment* segment_alloc_from_os() {
    MemId memid;
    
    // Allocate a full segment from OS - this calls mmap internally
    void* ptr = os_alloc_aligned(SEGMENT_SIZE, SEGMENT_ALIGN, true, &memid);
    if (!ptr) {
        return nullptr;
    }
    
    // Initialize segment metadata at the start of the allocated region
    // This is the crucial insight: metadata lives IN the allocated region
    Segment* segment = static_cast<Segment*>(ptr);
    
    // Zero initialize metadata (mmap gives us zero memory anyway)
    memset(segment, 0, sizeof(Segment));
    
    // Set up segment metadata
    segment->next = nullptr;              // Not in any list yet
    segment->memid = memid;
    segment->allow_decommit = !memid.is_pinned;
    segment->allow_purge = segment->allow_decommit;
    segment->segment_size = SEGMENT_SIZE;
    segment->commit_mask = ~0ULL;         // All committed for simplicity
    segment->purge_mask = 0;              // Nothing needs purging yet
    segment->thread_id = reinterpret_cast<uintptr_t>(pthread_self());
    segment->cookie = reinterpret_cast<uintptr_t>(segment) ^ 0xDEADBEEF; // Simple cookie
    segment->used = 0;
    segment->abandoned = 0;
    
    std::cout << "Created new segment with metadata at: " << segment 
              << " data starts at: " << segment->data << std::endl;
    
    return segment;
}

// Add segment to free list - demonstrate linked list management
void segment_cache_push(Segment* segment) {
    assert(segment != nullptr);
    assert(segment->next == nullptr); // Should not already be in a list
    
    // Insert at head of free list
    // This is the critical part - we're using the `next` field that lives
    // at the start of each segment's metadata to build our linked list
    segment->next = g_segment_cache.free_list;
    g_segment_cache.free_list = segment;
    g_segment_cache.count++;
    
    if (g_segment_cache.count > g_segment_cache.peak_count) {
        g_segment_cache.peak_count = g_segment_cache.count;
    }
    
    std::cout << "Cached segment " << segment << " (cache size: " << g_segment_cache.count << ")" << std::endl;
}

// Get segment from free list or allocate new one
Segment* segment_alloc() {
    // Try to reuse from cache first
    if (g_segment_cache.free_list) {
        Segment* segment = g_segment_cache.free_list;
        g_segment_cache.free_list = segment->next;  // Remove from list
        segment->next = nullptr;                    // Clear the link
        g_segment_cache.count--;
        
        std::cout << "Reused cached segment " << segment << " (cache size: " << g_segment_cache.count << ")" << std::endl;
        return segment;
    }
    
    // No cached segments, allocate from OS
    return segment_alloc_from_os();
}

// Free segment (either cache or return to OS)
void segment_free(Segment* segment) {
    if (!segment) return;
    
    // Reset segment state
    segment->used = 0;
    segment->abandoned = 0;
    
    // Could add logic here to sometimes return to OS instead of caching
    // For now, always cache
    segment_cache_push(segment);
}

// Demonstrate the system
int main() {
    std::cout << "=== Mimalloc-inspired Large Memory Container Demo ===" << std::endl;
    std::cout << "Segment size: " << SEGMENT_SIZE / (1024*1024) << " MiB" << std::endl;
    std::cout << "Metadata size: " << sizeof(Segment) << " bytes" << std::endl;
    std::cout << "Data area per segment: " << (SEGMENT_SIZE - sizeof(Segment)) / (1024*1024) << " MiB" << std::endl;
    std::cout << std::endl;
    
    // Allocate several segments to demonstrate linked list management
    std::cout << "1. Allocating segments..." << std::endl;
    Segment* seg1 = segment_alloc();
    Segment* seg2 = segment_alloc(); 
    Segment* seg3 = segment_alloc();
    
    std::cout << "\n2. Freeing segments (adding to cache)..." << std::endl;
    segment_free(seg1);
    segment_free(seg2);
    segment_free(seg3);
    
    std::cout << "\n3. Re-allocating segments (from cache)..." << std::endl;
    Segment* reused1 = segment_alloc();
    Segment* reused2 = segment_alloc();
    
    std::cout << "\n4. Verifying reuse..." << std::endl;
    std::cout << "seg3 == reused1: " << (seg3 == reused1 ? "Yes" : "No") << std::endl;
    std::cout << "seg2 == reused2: " << (seg2 == reused2 ? "Yes" : "No") << std::endl;
    
    std::cout << "\n5. Final cleanup..." << std::endl;
    
    // Clean up remaining segments
    if (reused1) os_free(reused1, SEGMENT_SIZE, reused1->memid);
    if (reused2) os_free(reused2, SEGMENT_SIZE, reused2->memid);
    
    // Clean up any remaining cached segments
    while (g_segment_cache.free_list) {
        Segment* segment = g_segment_cache.free_list;
        g_segment_cache.free_list = segment->next;
        os_free(segment, SEGMENT_SIZE, segment->memid);
    }
    
    std::cout << "\nDemo complete! Key insights:" << std::endl;
    std::cout << "- Segments (32MiB) are the largest containers allocated from mmap" << std::endl;
    std::cout << "- Metadata stored at the START of each segment (in-band)" << std::endl;
    std::cout << "- Linked lists built using 'next' pointer at offset 0" << std::endl;
    std::cout << "- No malloc/free needed - metadata lives in mmap'd region" << std::endl;
    std::cout << "- Bootstrap problem solved: first segment contains its own metadata" << std::endl;
    
    return 0;
}