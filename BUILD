load("@rules_foreign_cc//foreign_cc:defs.bzl", "cmake")

exports_files(["Cargo.toml", "Cargo.lock"])

filegroup(
    name = "all_srcs",
    srcs = glob(["**/*"], exclude = [
        "bazel-*/**",
        "target/**",
    ]) + [
        "//anise-cpp:srcs",
    ],
    visibility = ["//visibility:public"],
)

cmake(
    name = "anise_cpp_tests",
    lib_source = ":all_srcs",
    cache_entries = {
        "BUILD_RUST": "ON",
        "CMAKE_CXX_STANDARD": "14",
    },
    working_directory = "anise-cpp",
    out_binaries = ["test_time"],
    visibility = ["//visibility:public"],
)

sh_test(
    name = "cpp_test",
    srcs = ["run_cpp_tests.sh"],
    data = [":anise_cpp_tests"],
)
