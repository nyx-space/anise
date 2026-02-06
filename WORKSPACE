workspace(name = "anise")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# rules_rust
http_archive(
    name = "rules_rust",
    sha256 = "6501960c3e4da32495d1e1007ded0769a534cb195c30dea36aa54f9d8a3f0361",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.38.0/rules_rust-v0.38.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()
rust_register_toolchains()

# rules_foreign_cc for CMake
http_archive(
    name = "rules_foreign_cc",
    sha256 = "5816f4198184a1e0e682d7e6b817331219929401e2f18358fac7f7b172737976",
    strip_prefix = "rules_foreign_cc-0.10.0",
    url = "https://github.com/bazelbuild/rules_foreign_cc/archive/0.10.0.tar.gz",
)

load("@rules_foreign_cc//foreign_cc:repositories.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies()
