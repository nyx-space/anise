cmake_minimum_required(VERSION 3.10)
project(anise-cpp)

set(CMAKE_CXX_STANDARD 14)

# Option to build Rust library via Cargo
option(BUILD_RUST "Build Rust library via Cargo" ON)

if(BUILD_RUST)
    find_program(CARGO_EXECUTABLE cargo REQUIRED)

    # We want to build the static library
    set(CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo_target")

    execute_process(
        COMMAND ${CARGO_EXECUTABLE} build --package anise-cpp --release --target-dir ${CARGO_TARGET_DIR}
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/..
    )

    set(RUST_LIB "${CARGO_TARGET_DIR}/release/libanise_cpp.a")

    # Find generated headers
    file(GLOB_RECURSE CXX_HEADER "${CARGO_TARGET_DIR}/release/build/*/out/cxxbridge/include/anise-cpp/src/lib.rs.h")
    if(CXX_HEADER)
        list(GET CXX_HEADER 0 CXX_HEADER_PATH)
        get_filename_component(RUST_INCLUDE_DIR "${CXX_HEADER_PATH}" DIRECTORY)
        get_filename_component(RUST_INCLUDE_DIR "${RUST_INCLUDE_DIR}" DIRECTORY)
        get_filename_component(RUST_INCLUDE_DIR "${RUST_INCLUDE_DIR}" DIRECTORY)
    endif()
endif()

add_executable(test_time tests/main.cpp)

if(RUST_LIB)
    target_link_libraries(test_time PRIVATE ${RUST_LIB})
endif()

if(RUST_INCLUDE_DIR)
    target_include_directories(test_time PRIVATE ${RUST_INCLUDE_DIR})
endif()

target_link_libraries(test_time PRIVATE pthread dl)

install(TARGETS test_time DESTINATION bin)
