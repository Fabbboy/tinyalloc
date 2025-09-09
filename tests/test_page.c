#include "unity.h"
#include "../lib/platform/unix.c"

void setUp(void) {
    // Setup for each test
}

void tearDown(void) {
    // Cleanup after each test
}

void test_page_init_success(void) {
    ta_page_t page;
    ta_mapper_t mapper = ta_mapper();
    
    bool result = ta_page_init(&page, 4096, mapper);
    
    TEST_ASSERT_TRUE(result);
    TEST_ASSERT_NOT_NULL(page.ptr);
    TEST_ASSERT_EQUAL_size_t(4096, page.size);
    TEST_ASSERT_NOT_NULL(page.mapper.map);
    TEST_ASSERT_NOT_NULL(page.mapper.unmap);
    
    ta_page_deinit(&page);
}

void test_page_init_null_page(void) {
    ta_mapper_t mapper = ta_mapper();
    
    bool result = ta_page_init(NULL, 4096, mapper);
    
    TEST_ASSERT_FALSE(result);
}

void test_page_init_zero_size(void) {
    ta_page_t page;
    ta_mapper_t mapper = ta_mapper();
    
    bool result = ta_page_init(&page, 0, mapper);
    
    TEST_ASSERT_FALSE(result);
}

void test_page_init_invalid_mapper(void) {
    ta_page_t page;
    ta_mapper_t mapper = {NULL, NULL};
    
    bool result = ta_page_init(&page, 4096, mapper);
    
    TEST_ASSERT_FALSE(result);
}

void test_page_deinit_success(void) {
    ta_page_t page;
    ta_mapper_t mapper = ta_mapper();
    
    ta_page_init(&page, 4096, mapper);
    
    ta_page_deinit(&page);
    
    TEST_ASSERT_NULL(page.ptr);
    TEST_ASSERT_EQUAL_size_t(0, page.size);
}

void test_page_deinit_null_page(void) {
    ta_page_deinit(NULL);
}

void test_page_deinit_null_ptr(void) {
    ta_page_t page = {NULL, 4096, ta_mapper()};
    
    ta_page_deinit(&page);
}

void test_page_deinit_zero_size(void) {
    ta_page_t page = {(uint8_t*)0x1000, 0, ta_mapper()};
    
    ta_page_deinit(&page);
}

void test_mapper_functionality(void) {
    ta_mapper_t mapper = ta_mapper();
    
    TEST_ASSERT_NOT_NULL(mapper.map);
    TEST_ASSERT_NOT_NULL(mapper.unmap);
    
    uint8_t *ptr = NULL;
    bool result = mapper.map(&ptr, 4096);
    
    TEST_ASSERT_TRUE(result);
    TEST_ASSERT_NOT_NULL(ptr);
    
    mapper.unmap(ptr, 4096);
}

int main(void) {
    UNITY_BEGIN();
    
    RUN_TEST(test_page_init_success);
    RUN_TEST(test_page_init_null_page);
    RUN_TEST(test_page_init_zero_size);
    RUN_TEST(test_page_init_invalid_mapper);
    RUN_TEST(test_page_deinit_success);
    RUN_TEST(test_page_deinit_null_page);
    RUN_TEST(test_page_deinit_null_ptr);
    RUN_TEST(test_page_deinit_zero_size);
    RUN_TEST(test_mapper_functionality);
    
    return UNITY_END();
}