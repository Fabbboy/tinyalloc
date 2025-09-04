cmake_minimum_required(VERSION 3.20)
project(tinyalloc)

find_program(CARGO_CMD cargo REQUIRED)

set(CARGO_BUILD_FLAGS "")
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(CARGO_BUILD_FLAGS "--release")
    set(CARGO_PROFILE "release")
else()
    set(CARGO_PROFILE "debug")
endif()

set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/${CARGO_PROFILE}")

add_custom_target(cargo_build ALL
    COMMAND ${CARGO_CMD} build ${CARGO_BUILD_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    BYPRODUCTS 
        ${CARGO_TARGET_DIR}/libtinyalloc.a
        ${CARGO_TARGET_DIR}/libtinyalloc.so
        ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
    COMMENT "Building Rust library with cargo"
)

add_library(tinyalloc_static STATIC IMPORTED GLOBAL)
set_target_properties(tinyalloc_static PROPERTIES
    IMPORTED_LOCATION ${CARGO_TARGET_DIR}/libtinyalloc.a
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_static cargo_build)

add_library(tinyalloc_dynamic SHARED IMPORTED GLOBAL)
set_target_properties(tinyalloc_dynamic PROPERTIES
    IMPORTED_LOCATION ${CARGO_TARGET_DIR}/libtinyalloc.so
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_dynamic cargo_build)