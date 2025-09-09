#include <tinyalloc/tinyalloc-internal.h>

bool ta_segment_init(ta_segment_t *segment, size_t size, ta_mapper_t mapper) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), false);
  TA_CHECK_RET(TA_IS_ZERO(size), false);
  TA_VALIDATE_MAPPER(mapper, false);

  segment->next = NULL;
  return ta_page_init(&segment->page, size, mapper);
}

void ta_segment_deinit(ta_segment_t *segment) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), );

  ta_page_deinit(&segment->page);
  segment->next = NULL;
}

bool ta_segment_iter(ta_segment_t *segment, ta_segment_t **next) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), false);
  *next = NULL;
  TA_CHECK_RET(TA_IS_NULLPTR(next), false);
  *next = segment->next;
  return segment->next != NULL;
}

void ta_segment_space(ta_segment_t *segment, size_t *size, uint8_t **ptr) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment) || TA_IS_NULLPTR(size) ||
                   TA_IS_NULLPTR(ptr), );
  *size = segment->page.size;
  *ptr = segment->page.ptr;
}

void ta_segment_next(ta_segment_t *segment, ta_segment_t *next) {
  TA_CHECK_RET(TA_IS_NULLPTR(segment), );
  segment->next = next;
}