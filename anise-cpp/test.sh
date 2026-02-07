#!/bin/bash
set -e

# Use absolute paths where possible
REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
BUILD_DIR="$REPO_ROOT/anise-cpp/build_local"

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

cmake .. -DBUILD_RUST=ON
make
./test_time
