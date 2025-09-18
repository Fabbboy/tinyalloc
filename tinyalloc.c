#include "tinyalloc.h"
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <unistd.h>
#include <string.h>
#include <time.h>

#define NUM_THREADS 4
#define ALLOCS_PER_THREAD 25
#define TOTAL_ALLOCS 100
#define MAX_ALLOC_SIZE 1024

typedef struct {
    void **ptrs;
    size_t *sizes;
    int thread_id;
    int start_idx;
    int count;
} thread_data_t;

static void **global_ptrs;
static size_t *global_sizes;
static pthread_mutex_t array_mutex = PTHREAD_MUTEX_INITIALIZER;
static int allocation_count = 0;

void *allocator_thread(void *arg) {
    thread_data_t *data = (thread_data_t *)arg;

    printf("Thread %d: Starting allocations from index %d\n", data->thread_id, data->start_idx);

    for (int i = 0; i < data->count; i++) {
        size_t size = (rand() % MAX_ALLOC_SIZE) + 1;
        void *ptr = malloc(size);

        if (ptr) {
            memset(ptr, data->thread_id + 1, size);

            pthread_mutex_lock(&array_mutex);
            int idx = data->start_idx + i;
            global_ptrs[idx] = ptr;
            global_sizes[idx] = size;
            allocation_count++;
            printf("Thread %d: Allocated %zu bytes at %p (index %d, total: %d)\n",
                   data->thread_id, size, ptr, idx, allocation_count);
            pthread_mutex_unlock(&array_mutex);

            usleep(rand() % 1000);
        } else {
            printf("Thread %d: Allocation failed for %zu bytes\n", data->thread_id, size);
        }
    }

    printf("Thread %d: Finished allocations\n", data->thread_id);
    return NULL;
}

void *deallocator_thread(void *arg) {
    thread_data_t *data = (thread_data_t *)arg;

    printf("Deallocator thread %d: Starting deallocations\n", data->thread_id);

    while (1) {
        pthread_mutex_lock(&array_mutex);

        if (allocation_count == 0) {
            pthread_mutex_unlock(&array_mutex);
            break;
        }

        int idx = rand() % TOTAL_ALLOCS;
        if (global_ptrs[idx] != NULL) {
            void *ptr = global_ptrs[idx];
            size_t size = global_sizes[idx];
            global_ptrs[idx] = NULL;
            global_sizes[idx] = 0;
            allocation_count--;

            printf("Deallocator thread %d: Freeing %zu bytes at %p (index %d, remaining: %d)\n",
                   data->thread_id, size, ptr, idx, allocation_count);

            pthread_mutex_unlock(&array_mutex);
            free(ptr);
            usleep(rand() % 2000);
        } else {
            pthread_mutex_unlock(&array_mutex);
            usleep(100);
        }
    }

    printf("Deallocator thread %d: Finished deallocations\n", data->thread_id);
    return NULL;
}

int main() {
    printf("Starting chaotic multi-threaded allocation test with %d threads and %d total allocations\n",
           NUM_THREADS, TOTAL_ALLOCS);

    global_ptrs = calloc(TOTAL_ALLOCS, sizeof(void *));
    global_sizes = calloc(TOTAL_ALLOCS, sizeof(size_t));

    if (!global_ptrs || !global_sizes) {
        printf("Failed to allocate global arrays\n");
        return 1;
    }

    pthread_t allocator_threads[NUM_THREADS];
    pthread_t deallocator_threads[2];
    thread_data_t thread_data[NUM_THREADS + 2];

    srand(time(NULL));

    for (int i = 0; i < NUM_THREADS; i++) {
        thread_data[i].thread_id = i;
        thread_data[i].start_idx = i * ALLOCS_PER_THREAD;
        thread_data[i].count = ALLOCS_PER_THREAD;

        if (pthread_create(&allocator_threads[i], NULL, allocator_thread, &thread_data[i]) != 0) {
            printf("Failed to create allocator thread %d\n", i);
            return 1;
        }
    }

    usleep(500000);

    for (int i = 0; i < 2; i++) {
        thread_data[NUM_THREADS + i].thread_id = NUM_THREADS + i;

        if (pthread_create(&deallocator_threads[i], NULL, deallocator_thread, &thread_data[NUM_THREADS + i]) != 0) {
            printf("Failed to create deallocator thread %d\n", i);
            return 1;
        }
    }

    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(allocator_threads[i], NULL);
    }

    for (int i = 0; i < 2; i++) {
        pthread_join(deallocator_threads[i], NULL);
    }

    printf("\nFinal cleanup - freeing any remaining allocations:\n");
    int remaining = 0;
    for (int i = 0; i < TOTAL_ALLOCS; i++) {
        if (global_ptrs[i] != NULL) {
            printf("Cleaning up remaining allocation at index %d: %p (%zu bytes)\n",
                   i, global_ptrs[i], global_sizes[i]);
            free(global_ptrs[i]);
            remaining++;
        }
    }

    printf("Test completed. Cleaned up %d remaining allocations.\n", remaining);

    free(global_ptrs);
    free(global_sizes);

    return 0;
}