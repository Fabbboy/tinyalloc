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
set(STATIC_LIB "${CARGO_TARGET_DIR}/${CMAKE_STATIC_LIBRARY_PREFIX}tinyalloc${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(DYNAMIC_LIB "${CARGO_TARGET_DIR}/${CMAKE_SHARED_LIBRARY_PREFIX}tinyalloc${CMAKE_SHARED_LIBRARY_SUFFIX}")

# Create a unique target name to avoid conflicts when used as subdirectory
add_custom_target(tinyalloc_cargo_build ALL
    COMMAND ${CARGO_CMD} build ${CARGO_BUILD_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library with cargo"
    VERBATIM
)

add_custom_target(cargo_build ALL
    DEPENDS ${STATIC_LIB} ${DYNAMIC_LIB} ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
)

# Create imported libraries - remove GLOBAL to work properly with add_subdirectory
add_library(tinyalloc_static STATIC IMPORTED)
set_target_properties(tinyalloc_static PROPERTIES
    IMPORTED_LOCATION ${STATIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_static tinyalloc_cargo_build)

add_library(tinyalloc_dynamic SHARED IMPORTED)
set_target_properties(tinyalloc_dynamic PROPERTIES
    IMPORTED_LOCATION ${DYNAMIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_dynamic tinyalloc_cargo_build)
add_library(tinyalloc::static ALIAS tinyalloc_static)
add_library(tinyalloc::shared ALIAS tinyalloc_dynamic)
