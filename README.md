> These days I mainly just talk to plants and dogs.

# Meurglys3

This project is a library, cli tool and a C library written in Rust.
The library is designed to be used to package directories into singular .m3pkg files which can later be unpacked in memory and their contents used or written to a directory.

# Meurglys3 lib

This is the core of the project, the rust library that handles packaging. Its source is in the `src` directory at the top of the project tree.
You can build it manually or with `cargo make` (do note that the top level Makefile.toml builds the whole project).

# Meurglys3 cli

This part of the project is located under the `/meurglys3` subdirectory.

This is the cli tool that uses the rust library you can run the built binary with the `help` command to get a list of all available functions.
```
meurglys3 help
```

# Meurglys3c

This part of the project is located under the `/meurglys3c` subdirectory.

This is the C-binding of Meurglys3 lib that you can use outside of rust. See further advice on building.

# Building

The top level Makefile.toml is responsible for building the whole project. Additionally the meu3 cli tool and c-bindings library can be build with their own separate Makefile.toml files. You can build everything manually as well.

Meurglys3c will also use the `cbindgen` tool to generate a C header file and output it to `/target/include`.

# Cmake

There also is a Cmake script to build and run tests on the C bindings library. I'm not 100% certain it will work everywhere as it relies on a regex that I'm not really proud of. Cmake will manually build and link the static meurglys3c library. You can then enter the binary directory and run `ctest` to test the library.
