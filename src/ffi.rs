/*
C11 (N1570 Committee Draft — freely accessible)

All in §7.22.3 Memory management functions:

General rules — order/contiguity of allocations is unspecified; successful results are “suitably aligned” for any type with fundamental alignment; zero-size requests have implementation-defined behavior (may return null or a unique pointer); lifetime runs from allocation to explicit deallocation. 
ISO 9899

aligned_alloc — alignment must be a valid alignment; size must be an integral multiple of alignment; returns null or a pointer. §7.22.3.1. 
ISO 9899

calloc — allocates nmemb * size, storage is all-bits-zero (not necessarily numeric zero for all types); returns null or a pointer. §7.22.3.2. 
ISO 9899

free — deallocates; free(NULL) is a no-op; freeing a non-allocated or already-freed pointer is UB. §7.22.3.3. 
ISO 9899

malloc — allocates size bytes, value is indeterminate; returns null or a pointer. §7.22.3.4. 
ISO 9899

realloc — returns a new object of size, preserving min(old,new) bytes; if ptr == NULL, acts like malloc(size); if it fails, the old object is unchanged; using an invalid/previously-freed pointer is UB. §7.22.3.5. 
ISO 9899

(You can read N1570 online: it’s the widely cited, freely hosted C11 committee draft.) 
ISO 9899

*/

// this is rust edition 2024 so #[no_mangle] is an error
// new api: #[unsafe(no_mangle)] just accept it
