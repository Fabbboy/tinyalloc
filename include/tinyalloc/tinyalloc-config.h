#ifndef TINYALLOC_CONFIG_H
#define TINYALLOC_CONFIG_H

#ifndef TA_CONSTANTS
#define TA_CONSTANTS

#define TA_KiB (1 << 10)
#define TA_ONE_KB (TA_KiB)
#define TA_ONE_MB (TA_KiB * TA_ONE_KB)

#endif

#ifndef TA_ARENA_SIZE
#define TA_ARENA_MULTIPLIER 64
#define TA_ARENA_SIZE (TA_ARENA_MULTIPLIER * TA_ONE_MB) // 64 MiB
#endif

#endif