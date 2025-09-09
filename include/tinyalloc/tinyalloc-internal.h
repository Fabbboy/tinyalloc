#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define TA_IS_NULLPTR(p) ((p) == NULL)
#define TA_IS_ZERO(s) ((s) == 0)
#define TA_CHECK_RET(expr, ret)                                                \
  if (expr) {                                                                  \
    return ret;                                                                \
  }

typedef struct {
  bool (*map)(uint8_t **ptr, size_t size);
  void (*unmap)(uint8_t *ptr, size_t size);
  // add protect decommit and commit -> MVP 2
} ta_mapper_t;

ta_mapper_t ta_mapper(void);

#define TA_VALIDATE_MAPPER(mapper, ret)                                        \
  TA_CHECK_RET(TA_IS_NULLPTR(mapper.map) || TA_IS_NULLPTR(mapper.unmap), ret)

typedef struct {
  uint8_t *ptr;
  size_t size;
  ta_mapper_t mapper;
} ta_page_t;

bool ta_page_init(ta_page_t *page, size_t size, ta_mapper_t mapper);
void ta_page_deinit(ta_page_t *page);