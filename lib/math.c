#include "limits.h"
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

size_t ta_next_power_of_2(size_t n) {
  if (n == 0)
    return 1;

  const size_t HIGHEST_P2 = (SIZE_MAX >> 1) + 1;
  if (n > HIGHEST_P2)
    return 0;

  n--;
  for (size_t shift = 1; shift < sizeof(size_t) * CHAR_BIT; shift <<= 1) {
    n |= n >> shift;
  }
  return n + 1;
}

size_t ta_prev_power_of_2(size_t n) {
  if (n == 0)
    return 0;

  for (size_t shift = 1; shift < sizeof(size_t) * CHAR_BIT; shift <<= 1) {
    n |= n >> shift;
  }

  return n - (n >> 1);
}

bool ta_is_power_of_2(size_t n) { return n != 0 && (n & (n - 1)) == 0; }

size_t ta_align_up(size_t n, size_t alignment) {
  if (alignment == 0)
    return n;
  return (n + alignment - 1) & ~(alignment - 1);
}

size_t ta_align_down(size_t n, size_t alignment) {
  if (alignment == 0)
    return n;
  return n & ~(alignment - 1);
}
