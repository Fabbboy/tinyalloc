#ifndef TINYALLOC_INTERNAL_H
#define TINYALLOC_INTERNAL_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define TA_IS_NULLPTR(p) ((p) == NULL)
#define TA_IS_ZERO(s) ((s) == 0)
#define TA_CHECK_RET(expr, ret)                                                \
  if (expr) {                                                                  \
    return ret;                                                                \
  }
#define TA_BITS_OF(T) (sizeof(T) * 8)

typedef struct {
  bool (*map)(uint8_t **ptr, size_t size);
  void (*unmap)(uint8_t *ptr, size_t size);
  // add protect decommit and commit -> MVP 2
} ta_mapper_t;

ta_mapper_t ta_mapper(void);

#define TA_VALIDATE_MAPPER(mapper, ret)                                        \
  TA_CHECK_RET(TA_IS_NULLPTR(mapper.map) || TA_IS_NULLPTR(mapper.unmap), ret)

typedef struct {
  uint8_t *ptr;
  size_t size;
  ta_mapper_t mapper;
} ta_page_t;

bool ta_page_init(ta_page_t *page, size_t size, ta_mapper_t mapper);
void ta_page_deinit(ta_page_t *page);

#define TA_BITMAP_TYPE uint64_t

typedef struct {
  TA_BITMAP_TYPE *bits;
  size_t bit_count;
} ta_bitmap_t;

size_t ta_bitmap_require(size_t bit_count);
bool ta_bitmap_init(ta_bitmap_t *bitmap, uint8_t *bits, size_t length,
                    size_t bit_count);
void ta_bitmap_clear(ta_bitmap_t *bitmap, size_t index);
void ta_bitmap_set(ta_bitmap_t *bitmap, size_t index);
bool ta_bitmap_zero(ta_bitmap_t *bitmap);
bool ta_bitmap_one(ta_bitmap_t *bitmap);
size_t ta_bitmap_find_first_set(ta_bitmap_t *bitmap);
size_t ta_bitmap_find_first_clear(ta_bitmap_t *bitmap);

typedef struct ta_segment_t {
  struct ta_segment_t *next;
  struct ta_segment_t *prev;
  ta_page_t page;
  uint8_t *data;
  size_t usable;
} ta_segment_t;

bool ta_segment_init(ta_segment_t **segment, size_t size, ta_mapper_t mapper);
void ta_segment_next(ta_segment_t *segment, ta_segment_t *next);
void ta_segment_prev(ta_segment_t *segment, ta_segment_t *prev);
void ta_segment_space(ta_segment_t *segment, size_t *size, uint8_t **ptr);
bool ta_segment_iter(ta_segment_t *segment, ta_segment_t **next);
void ta_segment_deinit(ta_segment_t *segment);

#endif