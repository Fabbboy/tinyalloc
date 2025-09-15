cmake_minimum_required(VERSION 3.20)
project(tinyalloc)

find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_BUILD_FLAG "")
    set(CARGO_TARGET_DIR "debug")
else()
    set(CARGO_BUILD_FLAG "--release")
    set(CARGO_TARGET_DIR "release")
endif()

add_custom_target(tinyalloc_build ALL
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_BUILD_FLAG}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

set(TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/${CARGO_TARGET_DIR}")
set(STATIC_PREFIX ${CMAKE_STATIC_LIBRARY_PREFIX})
set(STATIC_SUFFIX ${CMAKE_STATIC_LIBRARY_SUFFIX})
set(SHARED_PREFIX ${CMAKE_SHARED_LIBRARY_PREFIX})
set(SHARED_SUFFIX ${CMAKE_SHARED_LIBRARY_SUFFIX})

set(LIB_STATIC "${TARGET_DIR}/${STATIC_PREFIX}tinyalloc${STATIC_SUFFIX}")
set(LIB_SHARED "${TARGET_DIR}/${SHARED_PREFIX}tinyalloc${SHARED_SUFFIX}")

add_library(tinyalloc_static STATIC IMPORTED GLOBAL)
set_target_properties(tinyalloc_static PROPERTIES
    IMPORTED_LOCATION ${LIB_STATIC}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_static tinyalloc_build)

add_library(tinyalloc_shared SHARED IMPORTED GLOBAL)
set_target_properties(tinyalloc_shared PROPERTIES
    IMPORTED_LOCATION ${LIB_SHARED}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_shared tinyalloc_build)
add_library(tinyalloc::static ALIAS tinyalloc_static)
add_library(tinyalloc::shared ALIAS tinyalloc_shared)