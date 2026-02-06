cmake_minimum_required(VERSION 3.10)
project(anise-cpp)

set(CMAKE_CXX_STANDARD 14)

# Option to build Rust library via Cargo
option(BUILD_RUST "Build Rust library via Cargo" ON)

if(BUILD_RUST)
    find_program(CARGO_EXECUTABLE cargo)
    if(CARGO_EXECUTABLE)
        execute_process(
            COMMAND ${CARGO_EXECUTABLE} build --package anise-cpp --release
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/..
        )
        # Try to find the library
        file(GLOB_RECURSE RUST_LIB_INTERNAL "${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libanise_cpp.a")
        if(RUST_LIB_INTERNAL)
            list(GET RUST_LIB_INTERNAL 0 RUST_LIB)
            message(STATUS "Found Rust library: ${RUST_LIB}")
        endif()

        # Try to find include dir
        file(GLOB_RECURSE CXX_HEADER "${CMAKE_CURRENT_SOURCE_DIR}/../target/release/build/*/out/cxxbridge/include/anise-cpp/src/lib.rs.h")
        if(CXX_HEADER)
            list(GET CXX_HEADER 0 CXX_HEADER_PATH)
            get_filename_component(RUST_INCLUDE_DIR "${CXX_HEADER_PATH}" DIRECTORY)
            get_filename_component(RUST_INCLUDE_DIR "${RUST_INCLUDE_DIR}" DIRECTORY)
            get_filename_component(RUST_INCLUDE_DIR "${RUST_INCLUDE_DIR}" DIRECTORY)
            message(STATUS "Found Rust include dir: ${RUST_INCLUDE_DIR}")
        endif()
    endif()
endif()

add_executable(test_time tests/main.cpp)

if(RUST_LIB)
    target_link_libraries(test_time PRIVATE ${RUST_LIB})
else()
    target_link_libraries(test_time PRIVATE anise_cpp) # Assume it's provided by Bazel
endif()

if(RUST_INCLUDE_DIR)
    target_include_directories(test_time PRIVATE ${RUST_INCLUDE_DIR})
endif()

# We also need the cxx.h header which is usually in the same include dir or provided
# For now, assume it's in the same place as lib.rs.h (which it should be in the 'rust' subdir)

target_link_libraries(test_time PRIVATE pthread dl)
