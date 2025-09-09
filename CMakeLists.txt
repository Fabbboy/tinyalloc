cmake_minimum_required(VERSION 3.20)
project(tinyalloc)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if(WIN32)
    add_compile_definitions(TA_PLATFORM_WINDOWS=1)
else()
    add_compile_definitions(TA_PLATFORM_POSIX=1)
endif()

if(CMAKE_C_COMPILER_ID MATCHES "MSVC")
    add_compile_definitions(TA_COMPILER_MSVC_LIKE=1)
else()
    add_compile_definitions(TA_COMPILER_POSIX_LIKE=1)
endif()

set(TINYALLOC_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/include")
set(TINYALLOC_SOURCES
    lib/math.c
)

add_library(tinyalloc STATIC ${TINYALLOC_SOURCES})
target_include_directories(tinyalloc
    PUBLIC
        $<BUILD_INTERFACE:${TINYALLOC_INCLUDE}>
        $<INSTALL_INTERFACE:include>
)
