#include "tinyalloc/tinyalloc-internal.h"
#include "unity.h"
#include <stdio.h>

void test_segment_init_success(void) {
  ta_segment_t *segment;
  ta_mapper_t mapper = ta_mapper();

  bool result = ta_segment_init(&segment, 4096, mapper);

  TEST_ASSERT_TRUE(result);
  TEST_ASSERT_NOT_NULL(segment->page.ptr);
  TEST_ASSERT_EQUAL_size_t(4096, segment->page.size);
  TEST_ASSERT_NOT_NULL(segment->page.mapper.map);
  TEST_ASSERT_NOT_NULL(segment->page.mapper.unmap);
  TEST_ASSERT_NULL(segment->item.next);

  ta_segment_deinit(segment);
}

void test_segment_write(void) {
  ta_mapper_t mapper = ta_mapper();
  ta_segment_t *segment;

  TEST_ASSERT_TRUE(ta_segment_init(&segment, 4096, mapper));

  size_t size;
  uint8_t *ptr;
  ta_segment_space(segment, &size, &ptr);

  TEST_ASSERT_NOT_NULL(ptr);
  TEST_ASSERT_EQUAL(size, 4096 - sizeof(ta_segment_t));
  size_t write_size = size < 100 ? size : 100;
  for (size_t i = 0; i < write_size; i++) {
    ptr[i] = (uint8_t)(i % 256);
  }

  for (size_t i = 0; i < write_size; i++) {
    TEST_ASSERT_EQUAL_UINT8((uint8_t)(i % 256), ptr[i]);
  }

  ta_segment_deinit(segment);
}

int main(void) {
  UNITY_BEGIN();

  RUN_TEST(test_segment_init_success);
  RUN_TEST(test_segment_write);

  return UNITY_END();
}