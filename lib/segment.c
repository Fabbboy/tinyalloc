#include "math.c"
#include <stdalign.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <tinyalloc/tinyalloc-internal.h>

bool ta_segment_init(ta_segment_t **segment, size_t size, ta_mapper_t mapper) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment) || TA_IS_ZERO(size), false);
  TA_VALIDATE_MAPPER(mapper, false);

  ta_page_t page;
  TA_CHECK_RET(!ta_page_init(&page, size, mapper), false);

  uint8_t *base_aligned =
      (uint8_t *)ta_align_up((size_t)page.ptr, alignof(ta_segment_t));
  uint8_t *user_ptr = base_aligned + sizeof(ta_segment_t);
  size_t overhead = user_ptr - page.ptr;

  ta_segment_t *seg = (ta_segment_t *)base_aligned;
  seg->page = page;
  seg->next = NULL;
  seg->data = user_ptr;
  seg->usable = page.size - overhead;
  *segment = seg;
  return true;
}

void ta_segment_deinit(ta_segment_t *segment) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), );
  ta_page_t page = segment->page;
  ta_page_deinit(&page);
}

bool ta_segment_iter(ta_segment_t *segment, ta_segment_t **next) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment) || TA_IS_NULLPTR(next), false);
  return (*next = segment->next) != NULL;
}

void ta_segment_space(ta_segment_t *segment, size_t *size, uint8_t **ptr) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment) || TA_IS_NULLPTR(size) ||
                   TA_IS_NULLPTR(ptr), );
  *size = segment->usable;
  *ptr = segment->data;
}

void ta_segment_next(ta_segment_t *segment, ta_segment_t *next) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), );
  segment->next = next;
}

void ta_segment_prev(ta_segment_t *segment, ta_segment_t *prev) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), );
  segment->prev = prev;
}