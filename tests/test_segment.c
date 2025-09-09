#include "../lib/segment.c"
#include "tinyalloc/tinyalloc-internal.h"
#include "unity.h"

void test_segment_init_success(void) {
  ta_segment_t segment;
  ta_mapper_t mapper = ta_mapper();

  bool result = ta_segment_init(&segment, 4096, mapper);

  TEST_ASSERT_TRUE(result);
  TEST_ASSERT_NOT_NULL(segment.page.ptr);
  TEST_ASSERT_EQUAL_size_t(4096, segment.page.size);
  TEST_ASSERT_NOT_NULL(segment.page.mapper.map);
  TEST_ASSERT_NOT_NULL(segment.page.mapper.unmap);
  TEST_ASSERT_NULL(segment.next);

  ta_segment_deinit(&segment);
}

void test_segment_iterator(void) {
  ta_mapper_t mapper = ta_mapper();
  ta_segment_t segment1, segment2, segment3;

  TEST_ASSERT_TRUE(ta_segment_init(&segment1, 4096, mapper));
  TEST_ASSERT_TRUE(ta_segment_init(&segment2, 4096, mapper));
  TEST_ASSERT_TRUE(ta_segment_init(&segment3, 4096, mapper));

  ta_segment_next(&segment1, &segment2);
  ta_segment_next(&segment2, &segment3);
  ta_segment_next(&segment3, NULL);

  ta_segment_t *current = &segment1;
  ta_segment_t *next;
  int count = 0;

  while (current != NULL) {
    count++;
    bool has_next = ta_segment_iter(current, &next);
    ta_segment_deinit(current);
    current = next;

    if (count == 1)
      TEST_ASSERT_TRUE(has_next);
    if (count == 2)
      TEST_ASSERT_TRUE(has_next);
    if (count == 3)
      TEST_ASSERT_FALSE(has_next);
  }

  TEST_ASSERT_EQUAL_INT(3, count);
}

void test_segment_write(void) {
  ta_mapper_t mapper = ta_mapper();
  ta_segment_t segment;

  TEST_ASSERT_TRUE(ta_segment_init(&segment, 4096, mapper));

  size_t size;
  uint8_t *ptr;
  ta_segment_space(&segment, &size, &ptr);

  TEST_ASSERT_NOT_NULL(ptr);
  TEST_ASSERT_EQUAL_size_t(4096, size);

  // Write some data to verify the segment is writable
  for (int i = 0; i < 100; i++) {
    ptr[i] = (uint8_t)(i % 256);
  }

  // Verify the written data
  for (int i = 0; i < 100; i++) {
    TEST_ASSERT_EQUAL_UINT8((uint8_t)(i % 256), ptr[i]);
  }

  ta_segment_deinit(&segment);
}

int main(void) {
  UNITY_BEGIN();
  
  RUN_TEST(test_segment_init_success);
  RUN_TEST(test_segment_iterator);
  RUN_TEST(test_segment_write);
  
  return UNITY_END();
}