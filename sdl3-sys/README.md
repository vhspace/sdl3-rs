# sdl3-sys

Rust bindings to version 3 of the [SDL library](https://github.com/libsdl-org/SDL).

## Building

This requires the SDL3 C library and header files.

To build SDL3 from source:

```shell
cargo build -F bindgen -F bundled
```

To use the SDL3 library installed in your system:

```shell
cargo build -F bindgen -F pkg-config
```
