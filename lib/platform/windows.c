
#ifdef TA_PLATFORM_WINDOWS
#include "page.c"
#include "tinyalloc/tinyalloc-internal.h"
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <windows.h>

const uint16_t TA_OS_PERM_RW = PAGE_READWRITE;
const uint16_t TA_OS_FLAGS = MEM_COMMIT | MEM_RESERVE;

size_t ta_get_page_size(void) {
  size_t ps = atomic_load_explicit(&page_size, memory_order_acquire);
  TA_CHECK_RET(!TA_IS_ZERO(ps), ps);

  SYSTEM_INFO si;
  GetSystemInfo(&si);
  size_t computed = (size_t)si.dwPageSize;

  if (computed == 0) {
    computed = 4096;
  }

  size_t expected = 0;

  if (atomic_compare_exchange_strong_explicit(&page_size, &expected, computed,
                                              memory_order_release,
                                              memory_order_relaxed)) {
    return computed;
  }
  return expected;
}

void os_unmap(uint8_t *ptr, size_t size) {
  TA_CHECK_RET(TA_IS_NULLPTR(ptr) || TA_IS_ZERO(size), );

  VirtualFree(ptr, 0, MEM_RELEASE);
}

bool os_map(uint8_t **ptr, size_t size) {
  TA_CHECK_RET(TA_IS_NULLPTR(ptr) || TA_IS_ZERO(size), false);

  void *p = VirtualAlloc(NULL, size, TA_OS_FLAGS, TA_OS_PERM_RW);
  TA_CHECK_RET(p == NULL, false);

  *ptr = (uint8_t *)p;
  return true;
}

ta_mapper_t ta_mapper(void) {
  ta_mapper_t mapper = {0};
  mapper.map = os_map;
  mapper.unmap = os_unmap;
  return mapper;
}
#endif