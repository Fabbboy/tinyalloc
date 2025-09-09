#include "math.c"
#include <stdalign.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <tinyalloc/tinyalloc-internal.h>

bool ta_arena_init(ta_arena_t **arena, size_t size, ta_mapper_t mapper) {
  TA_CHECK_RET(TA_IS_NULLPTR(arena) || TA_IS_ZERO(size), false);
  TA_VALIDATE_MAPPER(mapper, false);

  ta_page_t page;
  TA_CHECK_RET(!ta_page_init(&page, size, mapper), false);

  uint8_t *base_aligned =
      (uint8_t *)ta_align_up((size_t)page.ptr, alignof(ta_arena_t));
  uint8_t *user_ptr = base_aligned + sizeof(ta_arena_t);
  size_t overhead = user_ptr - page.ptr;

  ta_arena_t *seg = (ta_arena_t *)base_aligned;
  seg->page = page;
  seg->data = user_ptr;
  seg->usable = page.size - overhead;

  ta_item_t item = {0};
  item.next = NULL;
  item.prev = NULL;
  item.ptr = seg;

  seg->item = item;
  *arena = seg;
  return true;
}

void ta_arena_deinit(ta_arena_t *arena) {
  TA_CHECK_RET(TA_IS_NULLPTR(arena), );
  ta_page_t page = arena->page;
  ta_page_deinit(&page);
}

void ta_arena_space(ta_arena_t *arena, size_t *size, uint8_t **ptr) {
  TA_CHECK_RET(TA_IS_NULLPTR(arena) || TA_IS_NULLPTR(size) ||
                   TA_IS_NULLPTR(ptr), );
  *size = arena->usable;
  *ptr = arena->data;
}