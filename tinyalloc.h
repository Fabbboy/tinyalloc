#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void *malloc(uintptr_t size);

void *calloc(uintptr_t num, uintptr_t size);

void free(void *ptr);

void *realloc(void *ptr, uintptr_t size);
