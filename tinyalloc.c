#include "tinyalloc.h"
#include <stdint.h>
#include <stdio.h>

int main() {
  char *p = (char *)malloc(100);
  for (int i = 0; i < 100; i++) {
    p[i] = (char)i;
  }

  for (int i = 0; i < 100; i++) {
    printf("%d ", p[i]);
  }

  free(p);
}