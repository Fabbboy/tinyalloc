enable_testing()

set(TINYALLOC_TESTS
    test_math.c
    test_page.c
)

foreach(test ${TINYALLOC_TESTS})
    get_filename_component(test_name ${test} NAME_WE)
    add_executable(${test_name} ${test})
    target_link_libraries(${test_name} 
        tinyalloc 
        unity::framework
    )
    target_include_directories(${test_name} PRIVATE ${TINYALLOC_INCLUDE})
    add_test(NAME ${test_name} COMMAND ${test_name})
endforeach()