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

# Create a target that depends on the libraries being built
add_custom_target(tinyalloc_build
    DEPENDS ${STATIC_LIB} ${DYNAMIC_LIB} ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
)

# Create the static library target
add_library(tinyalloc_static STATIC IMPORTED)
set_target_properties(tinyalloc_static PROPERTIES
    IMPORTED_LOCATION ${STATIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_static tinyalloc_build)

# Create the shared library target  
add_library(tinyalloc_dynamic SHARED IMPORTED)
set_target_properties(tinyalloc_dynamic PROPERTIES
    IMPORTED_LOCATION ${DYNAMIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_dynamic tinyalloc_build)

# Create alias targets for easier usage
add_library(tinyalloc::static ALIAS tinyalloc_static)
add_library(tinyalloc::shared ALIAS tinyalloc_dynamic)