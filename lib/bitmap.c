#include "tinyalloc/tinyalloc-internal.h"
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#ifdef TA_COMPILER_MSVC_LIKE
#include <intrin.h>
#endif

#define TA_WORD_BITS TA_BITS_OF(TA_BITMAP_TYPE)
#define TA_WORD_SIZE sizeof(TA_BITMAP_TYPE)
#define TA_WORD_INDEX(i) ((i) / TA_WORD_BITS)
#define TA_BIT_POS(i) ((i) % TA_WORD_BITS)
#define TA_BIT_MASK(i) ((TA_BITMAP_TYPE)1 << TA_BIT_POS(i))
#define TA_WORDS_REQ(n) (((n) + TA_WORD_BITS - 1) / TA_WORD_BITS)
#define TA_BYTE_ZERO 0x00
#define TA_BYTE_ONE 0xFF
#define TA_WORD_ALL_ONES (~(TA_BITMAP_TYPE)0)
#define TA_WORD_ALL_ZEROS ((TA_BITMAP_TYPE)0)

static inline TA_BITMAP_TYPE ta_make_mask(size_t bits) {
  if (bits == 0)
    return TA_WORD_ALL_ZEROS;
  if (bits >= TA_WORD_BITS)
    return TA_WORD_ALL_ONES;
  return ((TA_BITMAP_TYPE)1 << bits) - 1;
}

static inline size_t ta_last_word_mask(size_t bits) {
  size_t r = bits % TA_WORD_BITS;
  return r ? ta_make_mask(r) : SIZE_MAX;
}

static bool ta_bitmap_fits(size_t bit_count, size_t buffer_size) {
  return TA_WORDS_REQ(bit_count) <= buffer_size / TA_WORD_SIZE;
}

size_t ta_bitmap_require(size_t bit_count) {
  TA_CHECK_RET(TA_IS_ZERO(bit_count), 0);
  return TA_WORDS_REQ(bit_count);
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

static size_t ta_ctz(TA_BITMAP_TYPE word) {
  if (word == TA_WORD_ALL_ZEROS) 
    return TA_WORD_BITS;

#ifdef TA_COMPILER_POSIX_LIKE
  return __builtin_ctzll(word);
#elif defined(TA_COMPILER_MSVC_LIKE)
  unsigned long index;
  _BitScanForward64(&index, word);
  return index;
#else
  size_t count = 0;
  TA_BITMAP_TYPE mask = 1;
  while ((word & mask) == 0) {
    word >>= 1;
    count++;
  }
  return count;
#endif
}

size_t ta_bitmap_find_first_set(ta_bitmap_t *bitmap) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), SIZE_MAX);
  
  size_t words = TA_WORDS_REQ(bitmap->bit_count);
  for (size_t w = 0; w < words; w++) {
    if (bitmap->bits[w] == TA_WORD_ALL_ZEROS)
      continue;
    size_t bit = w * TA_WORD_BITS + ta_ctz(bitmap->bits[w]);
    if (bit >= bitmap->bit_count)
      return SIZE_MAX;
    return bit;
  }
  return SIZE_MAX;
}

size_t ta_bitmap_find_first_clear(ta_bitmap_t *bitmap) {
  TA_CHECK_RET(TA_IS_NULLPTR(bitmap), SIZE_MAX);
  
  size_t words = TA_WORDS_REQ(bitmap->bit_count);
  for (size_t w = 0; w < words; w++) {
    TA_BITMAP_TYPE word = bitmap->bits[w];
    
    if (w == words - 1) {
      size_t bits_in_last = bitmap->bit_count % TA_WORD_BITS;
      if (bits_in_last > 0) {
        TA_BITMAP_TYPE mask = ta_make_mask(bits_in_last);
        word |= ~mask;
      }
    }
    
    if (word == TA_WORD_ALL_ONES)
      continue;
    size_t bit = w * TA_WORD_BITS + ta_ctz(~word);
    if (bit >= bitmap->bit_count)
      return SIZE_MAX;
    return bit;
  }
  return SIZE_MAX;
}
