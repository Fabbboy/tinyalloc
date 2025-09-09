#include <tinyalloc/tinyalloc.h>

void ta_heap_init(ta_heap_t *heap, ta_mapper_t mapper) {
  heap->mapper = mapper;
}