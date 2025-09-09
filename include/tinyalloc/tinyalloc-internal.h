#pragma once

#include <stdbool.h>
#include <stddef.h>

typedef struct {
  bool (*map)(void **ptr, size_t size);
  void (*unmap)(void *ptr, size_t size);
  //add protect decommit and commit -> MVP 2
} ta_mapper_t;