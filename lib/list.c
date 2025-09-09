#include <tinyalloc/tinyalloc-internal.h>

void ta_list_init(ta_list_t *list) {
  TA_CHECK_RET(TA_IS_NULLPTR(list), );
  list->head = NULL;
  list->tail = NULL;
  list->count = 0;
}

bool ta_list_empty(ta_list_t *list) {
  TA_CHECK_RET(TA_IS_NULLPTR(list), true);
  return list->count == 0;
}

size_t ta_list_count(ta_list_t *list) {
  TA_CHECK_RET(TA_IS_NULLPTR(list), 0);
  return list->count;
}

void ta_list_push(ta_list_t *list, ta_item_t *item) {
  TA_CHECK_RET(TA_IS_NULLPTR(list) || TA_IS_NULLPTR(item), );
  
  item->next = NULL;
  item->prev = list->tail;
  
  if (list->tail) {
    list->tail->next = item;
  } else {
    list->head = item;
  }
  
  list->tail = item;
  list->count++;
}

ta_item_t *ta_list_pop(ta_list_t *list) {
  TA_CHECK_RET(TA_IS_NULLPTR(list) || ta_list_empty(list), NULL);
  
  ta_item_t *item = list->tail;
  list->tail = item->prev;
  
  if (list->tail) {
    list->tail->next = NULL;
  } else {
    list->head = NULL;
  }
  
  item->next = NULL;
  item->prev = NULL;
  list->count--;
  
  return item;
}

void ta_list_remove(ta_list_t *list, ta_item_t *item) {
  TA_CHECK_RET(TA_IS_NULLPTR(list) || TA_IS_NULLPTR(item), );
  
  if (item->prev) {
    item->prev->next = item->next;
  } else {
    list->head = item->next;
  }
  
  if (item->next) {
    item->next->prev = item->prev;
  } else {
    list->tail = item->prev;
  }
  
  item->next = NULL;
  item->prev = NULL;
  list->count--;
}