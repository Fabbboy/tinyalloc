#include <tinyalloc/tinyalloc-internal.h>

ta_bin_t ta_bin_create(size_t size) {
  ta_bin_t bin = {0};
  bin.size = size;

  ta_list_init(&bin.free_list);
  ta_list_init(&bin.used_list);

  return bin;
}