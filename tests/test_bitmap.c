#include "unity.h"
#include "tinyalloc/tinyalloc-internal.h"

void test_bitmap_find_first_set_empty(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  TEST_ASSERT_EQUAL_size_t(SIZE_MAX, ta_bitmap_find_first_set(&bitmap));
}

void test_bitmap_find_first_set_single(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  
  ta_bitmap_set(&bitmap, 5);
  TEST_ASSERT_EQUAL_size_t(5, ta_bitmap_find_first_set(&bitmap));
}

void test_bitmap_find_first_set_multiple(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  
  ta_bitmap_set(&bitmap, 10);
  ta_bitmap_set(&bitmap, 5);
  ta_bitmap_set(&bitmap, 20);
  
  TEST_ASSERT_EQUAL_size_t(5, ta_bitmap_find_first_set(&bitmap));
}

void test_bitmap_find_first_set_edge_cases(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  
  ta_bitmap_set(&bitmap, 0);
  TEST_ASSERT_EQUAL_size_t(0, ta_bitmap_find_first_set(&bitmap));
  
  ta_bitmap_zero(&bitmap);
  ta_bitmap_set(&bitmap, 63);
  TEST_ASSERT_EQUAL_size_t(63, ta_bitmap_find_first_set(&bitmap));
}

void test_bitmap_find_first_clear_empty(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  TEST_ASSERT_EQUAL_size_t(0, ta_bitmap_find_first_clear(&bitmap));
}

void test_bitmap_find_first_clear_partial(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  
  ta_bitmap_set(&bitmap, 0);
  TEST_ASSERT_EQUAL_size_t(1, ta_bitmap_find_first_clear(&bitmap));
  
  ta_bitmap_set(&bitmap, 1);
  ta_bitmap_set(&bitmap, 2);
  TEST_ASSERT_EQUAL_size_t(3, ta_bitmap_find_first_clear(&bitmap));
}

void test_bitmap_find_first_clear_full(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 64));
  
  ta_bitmap_one(&bitmap);
  TEST_ASSERT_EQUAL_size_t(SIZE_MAX, ta_bitmap_find_first_clear(&bitmap));
}

void test_bitmap_find_first_clear_odd_size(void) {
  uint8_t buffer[8];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 33));
  
  for (size_t i = 0; i < 32; i++) {
    ta_bitmap_set(&bitmap, i);
  }
  
  TEST_ASSERT_EQUAL_size_t(32, ta_bitmap_find_first_clear(&bitmap));
  
  ta_bitmap_set(&bitmap, 32);
  TEST_ASSERT_EQUAL_size_t(SIZE_MAX, ta_bitmap_find_first_clear(&bitmap));
}

void test_bitmap_cross_word_boundary(void) {
  uint8_t buffer[16];
  ta_bitmap_t bitmap;
  
  TEST_ASSERT_TRUE(ta_bitmap_init(&bitmap, buffer, sizeof(buffer), 128));
  
  for (size_t i = 0; i < 64; i++) {
    ta_bitmap_set(&bitmap, i);
  }
  
  TEST_ASSERT_EQUAL_size_t(0, ta_bitmap_find_first_set(&bitmap));
  TEST_ASSERT_EQUAL_size_t(64, ta_bitmap_find_first_clear(&bitmap));
  
  ta_bitmap_set(&bitmap, 70);
  TEST_ASSERT_EQUAL_size_t(64, ta_bitmap_find_first_clear(&bitmap));
}

int main(void) {
  UNITY_BEGIN();
  
  RUN_TEST(test_bitmap_find_first_set_empty);
  RUN_TEST(test_bitmap_find_first_set_single);
  RUN_TEST(test_bitmap_find_first_set_multiple);
  RUN_TEST(test_bitmap_find_first_set_edge_cases);
  RUN_TEST(test_bitmap_find_first_clear_empty);
  RUN_TEST(test_bitmap_find_first_clear_partial);
  RUN_TEST(test_bitmap_find_first_clear_full);
  RUN_TEST(test_bitmap_find_first_clear_odd_size);
  RUN_TEST(test_bitmap_cross_word_boundary);
  
  return UNITY_END();
}