#include "tinyalloc/tinyalloc-internal.h"
#include "unity.h"
#include <stdio.h>

void test_arena_init_success(void) {
  ta_arena_t *arena;
  ta_mapper_t mapper = ta_mapper();

  bool result = ta_arena_init(&arena, 4096, mapper);

  TEST_ASSERT_TRUE(result);
  TEST_ASSERT_NOT_NULL(arena->page.ptr);
  TEST_ASSERT_EQUAL_size_t(4096, arena->page.size);
  TEST_ASSERT_NOT_NULL(arena->page.mapper.map);
  TEST_ASSERT_NOT_NULL(arena->page.mapper.unmap);
  TEST_ASSERT_NULL(arena->item.next);

  ta_arena_deinit(arena);
}

void test_arena_write(void) {
  ta_mapper_t mapper = ta_mapper();
  ta_arena_t *arena;

  TEST_ASSERT_TRUE(ta_arena_init(&arena, 4096, mapper));

  size_t size;
  uint8_t *ptr;
  ta_arena_space(arena, &size, &ptr);

  TEST_ASSERT_NOT_NULL(ptr);
  TEST_ASSERT_EQUAL(size, 4096 - sizeof(ta_arena_t));
  size_t write_size = size < 100 ? size : 100;
  for (size_t i = 0; i < write_size; i++) {
    ptr[i] = (uint8_t)(i % 256);
  }

  for (size_t i = 0; i < write_size; i++) {
    TEST_ASSERT_EQUAL_UINT8((uint8_t)(i % 256), ptr[i]);
  }

  ta_arena_deinit(arena);
}

int main(void) {
  UNITY_BEGIN();

  RUN_TEST(test_arena_init_success);
  RUN_TEST(test_arena_write);

  return UNITY_END();
}