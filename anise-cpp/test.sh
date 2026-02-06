#!/bin/bash
set -e

# Build the Rust library
cargo build --package anise-cpp --release

# Find the generated header and library
HEADER_PATH=$(find target/release/build -name "lib.rs.h" | head -n 1)
INCLUDE_DIR=$(dirname $(dirname $(dirname $(dirname $HEADER_PATH))))
LIB_DIR="target/release"

# Find cxx.h as well
CXX_H_PATH=$(find target/release/build -name "cxx.h" | head -n 1)
CXX_INCLUDE_DIR=$(dirname $(dirname $CXX_H_PATH))

# Compile the C++ test
g++ anise-cpp/tests/main.cpp \
    target/release/build/*/out/cxxbridge/sources/anise-cpp/src/lib.rs.cc \
    -I $INCLUDE_DIR \
    -I $CXX_INCLUDE_DIR \
    -L $LIB_DIR -lanise_cpp -lpthread -ldl -o test_time

# Run the test
./test_time
