cmake_minimum_required(VERSION 3.20)
project(tinyalloc)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if (WIN32)
    add_compile_definitions(TA_PLATFORM_WINDOWS=1)
else()
    if (APPLE OR CMAKE_SYSTEM_NAME MATCHES "FreeBSD|OpenBSD|NetBSD|DragonFly")
        add_compile_definitions(TA_PLATFORM_OSX=1)
    elseif(UNIX)
        add_compile_definitions(TA_PLATFORM_UNIX=1)
    endif()
endif()

if(CMAKE_C_COMPILER_ID MATCHES "MSVC")
    add_compile_definitions(TA_COMPILER_MSVC_LIKE=1)
else()
    add_compile_definitions(TA_COMPILER_POSIX_LIKE=1)
endif()

set(TINYALLOC_LIB "${CMAKE_CURRENT_SOURCE_DIR}/lib")
set(TINYALLOC_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/include")
set(TINYALLOC_SOURCES
    lib/math.c
    lib/heap.c
    lib/bitmap.c
    lib/platform/unix.c
)

add_library(tinyalloc STATIC ${TINYALLOC_SOURCES})
target_include_directories(tinyalloc
    PUBLIC
        $<BUILD_INTERFACE:${TINYALLOC_INCLUDE}>
        $<INSTALL_INTERFACE:include> 
)

add_subdirectory(vendor/Unity)
add_subdirectory(tests)