#include "platform/page.c"
#include "tinyalloc/tinyalloc-internal.h"
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef TA_PLATFORM_UNIX
#include <sys/mman.h>
#include <unistd.h>

const uint16_t TA_OS_PERM_RW = PROT_READ | PROT_WRITE;
const uint16_t TA_OS_FLAGS = MAP_PRIVATE | MAP_ANONYMOUS;

size_t ta_get_page_size(void) {
  size_t ps = atomic_load_explicit(&page_size, memory_order_acquire);
  TA_CHECK_RET(!TA_IS_ZERO(ps), ps);

  long val = 0;
#if defined(_SC_PAGESIZE)
  val = sysconf(_SC_PAGESIZE);
#elif defined(_SC_PAGE_SIZE)
  val = sysconf(_SC_PAGE_SIZE);
#endif
  if (val <= 0) {
    val = 4096;
  }

  size_t computed = (size_t)val;
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

  munmap(ptr, size);
}
bool os_map(uint8_t **ptr, size_t size) {
  TA_CHECK_RET(TA_IS_NULLPTR(ptr) || TA_IS_ZERO(size), false);
  void *p = mmap(NULL, size, TA_OS_PERM_RW, TA_OS_FLAGS, -1, 0);
  TA_CHECK_RET(p == MAP_FAILED, false);
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