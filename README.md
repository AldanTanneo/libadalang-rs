# libadalang-rs

Work-in-progress bindings for the `libadalang` library.

High level bindings are in the root `libadalang` crate.

Low level bindings are generated via [bindgen](https://github.com/rust-lang/rust-bindgen) in the
`libadalang-sys` crate. They require an Alire installation to build (the `alr` binary must be
visible to the build script). Currently, the build script runs
`alr get --build libadalang=<library_version>`; the build is cached in the dependency's working
directory.

## Future improvements: 

- Allow the library to be linked from an external source (eg. a system installation?)
