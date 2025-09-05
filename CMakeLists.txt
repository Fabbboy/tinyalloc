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

# Build the Rust library - use ALL to ensure it's always built
add_custom_target(tinyalloc_cargo_build ALL
    COMMAND ${CARGO_CMD} build ${CARGO_BUILD_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    BYPRODUCTS 
        ${STATIC_LIB}
        ${DYNAMIC_LIB}
        ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
    COMMENT "Building Rust libraries and header"
    VERBATIM
)

# Create interface library instead of imported library to fix linking
add_library(tinyalloc_static INTERFACE)
target_link_libraries(tinyalloc_static INTERFACE ${STATIC_LIB})
target_include_directories(tinyalloc_static INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(tinyalloc_static tinyalloc_cargo_build)

add_library(tinyalloc_dynamic INTERFACE) 
target_link_libraries(tinyalloc_dynamic INTERFACE ${DYNAMIC_LIB})
target_include_directories(tinyalloc_dynamic INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(tinyalloc_dynamic tinyalloc_cargo_build)

# Create alias targets for easier usage
add_library(tinyalloc::static ALIAS tinyalloc_static)
add_library(tinyalloc::shared ALIAS tinyalloc_dynamic)