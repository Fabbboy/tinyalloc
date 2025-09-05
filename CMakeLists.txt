cmake_minimum_required(VERSION 3.20)

# Only set project if we're the top-level directory
if(CMAKE_CURRENT_SOURCE_DIR STREQUAL CMAKE_SOURCE_DIR)
    project(tinyalloc)
endif()

find_program(CARGO_CMD cargo REQUIRED)

set(CARGO_BUILD_FLAGS "")
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(CARGO_BUILD_FLAGS "--release")
    set(CARGO_PROFILE "release")
else()
    set(CARGO_PROFILE "debug")
endif()

set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/${CARGO_PROFILE}")
set(STATIC_LIB "${CARGO_TARGET_DIR}/libtinyalloc.a")
set(DYNAMIC_LIB "${CARGO_TARGET_DIR}/libtinyalloc.so")

# Build the Rust library
add_custom_command(
    OUTPUT ${STATIC_LIB} ${DYNAMIC_LIB} ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
    COMMAND ${CARGO_CMD} build ${CARGO_BUILD_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust libraries and header"
    VERBATIM
)

# Create wrapper static library that depends on the Rust library
add_library(tinyalloc_static_wrapper ${CMAKE_CURRENT_SOURCE_DIR}/wrapper.c)
target_include_directories(tinyalloc_static_wrapper PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(tinyalloc_static_wrapper PRIVATE ${STATIC_LIB})
add_dependencies(tinyalloc_static_wrapper ${STATIC_LIB})

# Create wrapper shared library that depends on the Rust library
add_library(tinyalloc_dynamic_wrapper SHARED ${CMAKE_CURRENT_SOURCE_DIR}/wrapper.c)
target_include_directories(tinyalloc_dynamic_wrapper PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(tinyalloc_dynamic_wrapper PRIVATE ${DYNAMIC_LIB})
add_dependencies(tinyalloc_dynamic_wrapper ${DYNAMIC_LIB})

# Create the preferred targets
add_library(tinyalloc_static ALIAS tinyalloc_static_wrapper)
add_library(tinyalloc_dynamic ALIAS tinyalloc_dynamic_wrapper)

# Create alias targets for easier usage
add_library(tinyalloc::static ALIAS tinyalloc_static_wrapper)
add_library(tinyalloc::shared ALIAS tinyalloc_dynamic_wrapper)