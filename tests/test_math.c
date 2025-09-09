#include "../lib/math.c"
#include "unity.h"

void setUp(void) {
  // Setup for each test
}

void tearDown(void) {
  // Cleanup after each test
}

void test_next_power_of_2_basic(void) {
  TEST_ASSERT_EQUAL_size_t(1, ta_next_power_of_2(0));
  TEST_ASSERT_EQUAL_size_t(1, ta_next_power_of_2(1));
  TEST_ASSERT_EQUAL_size_t(2, ta_next_power_of_2(2));
  TEST_ASSERT_EQUAL_size_t(4, ta_next_power_of_2(3));
  TEST_ASSERT_EQUAL_size_t(8, ta_next_power_of_2(5));
  TEST_ASSERT_EQUAL_size_t(16, ta_next_power_of_2(9));
  TEST_ASSERT_EQUAL_size_t(64, ta_next_power_of_2(33));
}

void test_prev_power_of_2_basic(void) {
  TEST_ASSERT_EQUAL_size_t(0, ta_prev_power_of_2(0));
  TEST_ASSERT_EQUAL_size_t(1, ta_prev_power_of_2(1));
  TEST_ASSERT_EQUAL_size_t(2, ta_prev_power_of_2(2));
  TEST_ASSERT_EQUAL_size_t(2, ta_prev_power_of_2(3));
  TEST_ASSERT_EQUAL_size_t(4, ta_prev_power_of_2(5));
  TEST_ASSERT_EQUAL_size_t(8, ta_prev_power_of_2(9));
  TEST_ASSERT_EQUAL_size_t(32, ta_prev_power_of_2(33));
}

void test_is_power_of_2_basic(void) {
  TEST_ASSERT_FALSE(ta_is_power_of_2(0));
  TEST_ASSERT_TRUE(ta_is_power_of_2(1));
  TEST_ASSERT_TRUE(ta_is_power_of_2(2));
  TEST_ASSERT_FALSE(ta_is_power_of_2(3));
  TEST_ASSERT_TRUE(ta_is_power_of_2(4));
  TEST_ASSERT_FALSE(ta_is_power_of_2(5));
  TEST_ASSERT_TRUE(ta_is_power_of_2(8));
  TEST_ASSERT_TRUE(ta_is_power_of_2(16));
  TEST_ASSERT_FALSE(ta_is_power_of_2(33));
}

void test_align_up_basic(void) {
  TEST_ASSERT_EQUAL_size_t(0, ta_align_up(0, 4));
  TEST_ASSERT_EQUAL_size_t(4, ta_align_up(1, 4));
  TEST_ASSERT_EQUAL_size_t(4, ta_align_up(4, 4));
  TEST_ASSERT_EQUAL_size_t(8, ta_align_up(5, 4));
  TEST_ASSERT_EQUAL_size_t(16, ta_align_up(9, 8));
  TEST_ASSERT_EQUAL_size_t(16, ta_align_up(16, 8));
}

void test_align_down_basic(void) {
  TEST_ASSERT_EQUAL_size_t(0, ta_align_down(0, 4));
  TEST_ASSERT_EQUAL_size_t(0, ta_align_down(1, 4));
  TEST_ASSERT_EQUAL_size_t(4, ta_align_down(4, 4));
  TEST_ASSERT_EQUAL_size_t(4, ta_align_down(7, 4));
  TEST_ASSERT_EQUAL_size_t(8, ta_align_down(15, 8));
  TEST_ASSERT_EQUAL_size_t(16, ta_align_down(16, 8));
}

void test_next_power_of_2_edge_cases(void) {
  TEST_ASSERT_EQUAL_size_t(0, ta_next_power_of_2(SIZE_MAX));
  TEST_ASSERT_EQUAL_size_t(0, ta_next_power_of_2((SIZE_MAX >> 1) + 2));

  size_t large_power = (SIZE_MAX >> 1) + 1;
  TEST_ASSERT_EQUAL_size_t(large_power, ta_next_power_of_2(large_power));
}

void test_prev_power_of_2_edge_cases(void) {
  TEST_ASSERT_EQUAL_size_t((SIZE_MAX >> 1) + 1, ta_prev_power_of_2(SIZE_MAX));

  size_t large_power = (SIZE_MAX >> 1) + 1;
  TEST_ASSERT_EQUAL_size_t(large_power, ta_prev_power_of_2(large_power));
}

void test_is_power_of_2_edge_cases(void) {
  TEST_ASSERT_FALSE(ta_is_power_of_2(SIZE_MAX));
  TEST_ASSERT_FALSE(ta_is_power_of_2(SIZE_MAX - 1));

  size_t large_power = (SIZE_MAX >> 1) + 1;
  TEST_ASSERT_TRUE(ta_is_power_of_2(large_power));
}

void test_align_up_edge_cases(void) {
  TEST_ASSERT_EQUAL_size_t(42, ta_align_up(42, 0));
  TEST_ASSERT_EQUAL_size_t(1, ta_align_up(1, 1));

  TEST_ASSERT_EQUAL_size_t(SIZE_MAX & ~1, ta_align_up(SIZE_MAX - 2, 2));
  // For SIZE_MAX - 10, aligning up to 4-byte boundary gives this result
  TEST_ASSERT_EQUAL_size_t(18446744073709551608UL,
                           ta_align_up(SIZE_MAX - 10, 4));
}

void test_align_down_edge_cases(void) {
  TEST_ASSERT_EQUAL_size_t(42, ta_align_down(42, 0));
  TEST_ASSERT_EQUAL_size_t(1, ta_align_down(1, 1));

  TEST_ASSERT_EQUAL_size_t(SIZE_MAX & ~1, ta_align_down(SIZE_MAX, 2));
  TEST_ASSERT_EQUAL_size_t(SIZE_MAX & ~3, ta_align_down(SIZE_MAX, 4));
}

void test_alignment_consistency(void) {
  for (size_t alignment = 1; alignment <= 64; alignment <<= 1) {
    for (size_t value = 0; value < 100; value++) {
      size_t aligned_up = ta_align_up(value, alignment);
      size_t aligned_down = ta_align_down(value, alignment);

      TEST_ASSERT_TRUE(aligned_up >= value);
      TEST_ASSERT_TRUE(aligned_down <= value);
      TEST_ASSERT_EQUAL_size_t(0, aligned_up % alignment);
      TEST_ASSERT_EQUAL_size_t(0, aligned_down % alignment);
    }
  }
}

int main(void) {
  UNITY_BEGIN();

  RUN_TEST(test_next_power_of_2_basic);
  RUN_TEST(test_prev_power_of_2_basic);
  RUN_TEST(test_is_power_of_2_basic);
  RUN_TEST(test_align_up_basic);
  RUN_TEST(test_align_down_basic);
  RUN_TEST(test_next_power_of_2_edge_cases);
  RUN_TEST(test_prev_power_of_2_edge_cases);
  RUN_TEST(test_is_power_of_2_edge_cases);
  RUN_TEST(test_align_up_edge_cases);
  RUN_TEST(test_align_down_edge_cases);
  RUN_TEST(test_alignment_consistency);

  return UNITY_END();
}