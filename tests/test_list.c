#include "tinyalloc/tinyalloc-internal.h"
#include "unity.h"

void test_list_init(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  TEST_ASSERT_NULL(list.head);
  TEST_ASSERT_NULL(list.tail);
  TEST_ASSERT_TRUE(ta_list_empty(&list));
}

void test_list_push_single_item(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  ta_item_t item = {0};
  item.ptr = (void*)0x1234;
  
  ta_list_push(&list, &item);
  
  TEST_ASSERT_EQUAL_PTR(&item, list.head);
  TEST_ASSERT_EQUAL_PTR(&item, list.tail);
  TEST_ASSERT_FALSE(ta_list_empty(&list));
  TEST_ASSERT_NULL(item.next);
  TEST_ASSERT_NULL(item.prev);
}

void test_list_push_multiple_items(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  ta_item_t item1 = {0};
  ta_item_t item2 = {0};
  ta_item_t item3 = {0};
  item1.ptr = (void*)0x1;
  item2.ptr = (void*)0x2;
  item3.ptr = (void*)0x3;
  
  ta_list_push(&list, &item1);
  ta_list_push(&list, &item2);
  ta_list_push(&list, &item3);
  
  TEST_ASSERT_EQUAL_PTR(&item1, list.head);
  TEST_ASSERT_EQUAL_PTR(&item3, list.tail);
  
  TEST_ASSERT_EQUAL_PTR(&item2, item1.next);
  TEST_ASSERT_NULL(item1.prev);
  
  TEST_ASSERT_EQUAL_PTR(&item3, item2.next);
  TEST_ASSERT_EQUAL_PTR(&item1, item2.prev);
  
  TEST_ASSERT_NULL(item3.next);
  TEST_ASSERT_EQUAL_PTR(&item2, item3.prev);
}

void test_list_pop_single_item(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  ta_item_t item = {0};
  item.ptr = (void*)0x1234;
  
  ta_list_push(&list, &item);
  ta_item_t *popped = ta_list_pop(&list);
  
  TEST_ASSERT_EQUAL_PTR(&item, popped);
  TEST_ASSERT_NULL(list.head);
  TEST_ASSERT_NULL(list.tail);
  TEST_ASSERT_TRUE(ta_list_empty(&list));
  TEST_ASSERT_NULL(popped->next);
  TEST_ASSERT_NULL(popped->prev);
}

void test_list_pop_multiple_items(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  ta_item_t item1 = {0};
  ta_item_t item2 = {0};
  ta_item_t item3 = {0};
  
  ta_list_push(&list, &item1);
  ta_list_push(&list, &item2);
  ta_list_push(&list, &item3);
  
  ta_item_t *popped3 = ta_list_pop(&list);
  TEST_ASSERT_EQUAL_PTR(&item3, popped3);
  TEST_ASSERT_EQUAL_PTR(&item2, list.tail);
  TEST_ASSERT_NULL(item2.next);
  
  ta_item_t *popped2 = ta_list_pop(&list);
  TEST_ASSERT_EQUAL_PTR(&item2, popped2);
  TEST_ASSERT_EQUAL_PTR(&item1, list.tail);
  TEST_ASSERT_NULL(item1.next);
  
  ta_item_t *popped1 = ta_list_pop(&list);
  TEST_ASSERT_EQUAL_PTR(&item1, popped1);
  TEST_ASSERT_NULL(list.head);
  TEST_ASSERT_NULL(list.tail);
}

void test_list_remove_middle_item(void) {
  ta_list_t list;
  ta_list_init(&list);
  
  ta_item_t item1 = {0};
  ta_item_t item2 = {0};
  ta_item_t item3 = {0};
  
  ta_list_push(&list, &item1);
  ta_list_push(&list, &item2);
  ta_list_push(&list, &item3);
  
  ta_list_remove(&list, &item2);
  
  TEST_ASSERT_EQUAL_PTR(&item3, item1.next);
  TEST_ASSERT_EQUAL_PTR(&item1, item3.prev);
  TEST_ASSERT_NULL(item2.next);
  TEST_ASSERT_NULL(item2.prev);
}

int main(void) {
  UNITY_BEGIN();
  
  RUN_TEST(test_list_init);
  RUN_TEST(test_list_push_single_item);
  RUN_TEST(test_list_push_multiple_items);
  RUN_TEST(test_list_pop_single_item);
  RUN_TEST(test_list_pop_multiple_items);
  RUN_TEST(test_list_remove_middle_item);
  
  return UNITY_END();
}