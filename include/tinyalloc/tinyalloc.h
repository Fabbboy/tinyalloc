#pragma once

#include "tinyalloc/tinyalloc-internal.h"

typedef struct {
  ta_mapper_t mapper;
} ta_heap_t;

void ta_heap_init(ta_heap_t *heap, ta_mapper_t mapper);