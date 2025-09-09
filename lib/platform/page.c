#ifndef PLATFORM_PAGE_C
#define PLATFORM_PAGE_C

#include "tinyalloc/tinyalloc-internal.h"
#include <stdbool.h>
#include <stddef.h>

static _Atomic size_t page_size = 0;

size_t ta_get_page_size(void);

bool ta_page_init(ta_page_t *page, size_t size, ta_mapper_t mapper) {
  TA_CHECK_RET(TA_IS_NULLPTR(page) || TA_IS_ZERO(size), false);
  TA_VALIDATE_MAPPER(mapper, false);

  uint8_t *ptr = NULL;
  TA_CHECK_RET(!mapper.map(&ptr, size), false);
  page->ptr = ptr;
  page->size = size;
  page->mapper = mapper;
  return true;
}

void ta_page_deinit(ta_page_t *page){
    TA_CHECK_RET(TA_IS_NULLPTR(page) || TA_IS_NULLPTR(page->ptr) ||
                     TA_IS_ZERO(page->size),
                 );
    
    page->mapper.unmap(page->ptr, page->size);
    page->ptr = NULL;
    page->size = 0;
}

#endif