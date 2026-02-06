load("@rules_foreign_cc//foreign_cc:defs.bzl", "cmake")

filegroup(
    name = "srcs",
    srcs = glob(["**"]),
    visibility = ["//visibility:public"],
)

cmake(
    name = "anise_cpp_tests",
    lib_source = "//:all_srcs",
    cache_entries = {
        "BUILD_RUST": "ON",
        "CMAKE_CXX_STANDARD": "14",
    },
    working_directory = "anise-cpp",
    out_binaries = ["test_time"],
    visibility = ["//visibility:public"],
)
