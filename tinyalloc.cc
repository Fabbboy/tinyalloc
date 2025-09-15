#include "tinyalloc.h"
#include <stdint.h>
#include <stdio.h>
#include <vector>

void* operator new(size_t size) {
  return malloc(size);
}

void* operator new[](size_t size) {
  return malloc(size);
}

void operator delete(void* p) noexcept {
  free(p);
}

void operator delete[](void* p) noexcept {
  free(p);
}

int main() {
  std::vector<char> chars;
  for (int i = 0; i < 100; i++) {
    chars.push_back((char)i);
  }

  for (int i = 0; i < 100; i++) {
    printf("%d ", chars[i]);
  }
}