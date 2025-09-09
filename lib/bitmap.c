#include "tinyalloc/tinyalloc-internal.h"
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#define TA_WORD_BITS TA_BITS_OF(TA_BITMAP_TYPE)
#define TA_WORD_SIZE sizeof(TA_BITMAP_TYPE)
#define TA_WORD_INDEX(i) ((i) / TA_WORD_BITS)
#define TA_BIT_POS(i) ((i) % TA_WORD_BITS)
#define TA_BIT_MASK(i) ((TA_BITMAP_TYPE)1 << TA_BIT_POS(i))
#define TA_WORDS_REQ(n) (((n) + TA_WORD_BITS - 1) / TA_WORD_BITS)
#define TA_BYTE_ZERO 0x00
#define TA_BYTE_ONE 0xFF

static inline size_t ta_last_word_mask(size_t bits) {
  size_t r = bits % TA_WORD_BITS;
  return r ? (((size_t)1 << r) - 1) : SIZE_MAX;
}

static bool ta_bitmap_fits(size_t bit_count, size_t buffer_size) {
  return TA_WORDS_REQ(bit_count) <= buffer_size / TA_WORD_SIZE;
}

size_t ta_bitmap_require(size_t bit_count) {
  TA_CHECK_RET(TA_IS_ZERO(bit_count), 0);
  return TA_WORDS_REQ(bit_count);
}

static inline size_t ta_bitmap_word_index(size_t index) {
  return TA_WORD_INDEX(index);
}

static inline size_t ta_bitmap_bit_position(size_t index) {
  return TA_BIT_POS(index);
}

void ta_bitmap_clear(ta_bitmap_t *bitmap, size_t index) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), );
  TA_CHECK_RET(index >= bitmap->bit_count, );
  bitmap->bits[TA_WORD_INDEX(index)] &= ~TA_BIT_MASK(index);
}

void ta_bitmap_set(ta_bitmap_t *bitmap, size_t index) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), );
  TA_CHECK_RET(index >= bitmap->bit_count, );
  bitmap->bits[TA_WORD_INDEX(index)] |= TA_BIT_MASK(index);
}

bool ta_bitmap_zero(ta_bitmap_t *bitmap) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), false);
  size_t words = ta_bitmap_require(bitmap->bit_count);
  memset(bitmap->bits, TA_BYTE_ZERO, words * TA_WORD_SIZE);
  return true;
}

bool ta_bitmap_one(ta_bitmap_t *bitmap) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), false);
  size_t words = ta_bitmap_require(bitmap->bit_count);
  memset(bitmap->bits, TA_BYTE_ONE, words * TA_WORD_SIZE);
  if (words)
    bitmap->bits[words - 1] &= ta_last_word_mask(bitmap->bit_count);
  return true;
}

bool ta_bitmap_init(ta_bitmap_t *bitmap, uint8_t *bits, size_t length,
                    size_t bit_count) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), false);
  TA_CHECK_RET(TA_IS_NULLPTR(bits), false);
  TA_CHECK_RET(TA_IS_ZERO(length) || TA_IS_ZERO(bit_count), false);
  TA_CHECK_RET(!ta_bitmap_fits(bit_count, length), false);
  bitmap->bits = (TA_BITMAP_TYPE *)bits;
  bitmap->bit_count = bit_count;
  return ta_bitmap_zero(bitmap);
}
