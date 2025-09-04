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
set(STATIC_LIB "${CARGO_TARGET_DIR}/${CMAKE_STATIC_LIBRARY_PREFIX}tinyalloc${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(DYNAMIC_LIB "${CARGO_TARGET_DIR}/${CMAKE_SHARED_LIBRARY_PREFIX}tinyalloc${CMAKE_SHARED_LIBRARY_SUFFIX}")

add_custom_command(
    OUTPUT ${STATIC_LIB} ${DYNAMIC_LIB} ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
    COMMAND ${CARGO_CMD} build ${CARGO_BUILD_FLAGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library with cargo"
    VERBATIM
)

add_custom_target(cargo_build ALL
    DEPENDS ${STATIC_LIB} ${DYNAMIC_LIB} ${CMAKE_CURRENT_SOURCE_DIR}/tinyalloc.h
)

add_library(tinyalloc_static STATIC IMPORTED GLOBAL)
set_target_properties(tinyalloc_static PROPERTIES
    IMPORTED_LOCATION ${STATIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_static cargo_build)

add_library(tinyalloc_dynamic SHARED IMPORTED GLOBAL)
set_target_properties(tinyalloc_dynamic PROPERTIES
    IMPORTED_LOCATION ${DYNAMIC_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}
)
add_dependencies(tinyalloc_dynamic cargo_build)
