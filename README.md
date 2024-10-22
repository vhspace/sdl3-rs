# SDL3 ![Crates.io Version](https://img.shields.io/crates/v/sdl3)

Bindings for SDL3 in Rust.

Now that the SDL3 API is mostly stabilized, we are working on a new version of the Rust bindings for SDL3.
The migration is in progress, and we are looking for contributors to help us complete it.

Expect some bugs and [missing features](https://wiki.libsdl.org/SDL3/NewFeatures).
Feel free to create issues or work on them yourself.

## Migration Progress

- [x] Update all modules to SDL3, use new [sdl3-sys](https://github.com/maia-s/sdl3-sys-rs) bindings,
  follow [migration guide](https://github.com/libsdl-org/SDL/blob/main/docs/README-migration.md).
- [ ] Fix tests.
- [ ] Update examples to SDL3.
- [ ] Add [new features](https://wiki.libsdl.org/SDL3/NewFeatures) from SDL3.
- [ ] Update documentation.

Please refer to the [sdl3-rs](https://github.com/revmischa/sdl3-rs) repository for the latest updates.

The low-level bindings are being worked on in the [sdl3-sys](https://github.com/maia-s/sdl3-sys-rs) repository.

# Overview

This is an interface to use SDL3 from Rust.

Low-level C components are wrapped in Rust code to make them more idiomatic and
abstract away inappropriate manual memory management.

## Quick Start

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sdl3 = { version = "0", features = ["use-pkg-config"] }
```

SDL3 requires the libsdl3 library to link and run. You may select how to link the library via features:

- `use-pkg-config`: Use `pkg-config` to find the library.
- `use-vcpkg`: Use `vcpkg` to find the library.
- `static-link`: Link the library statically.
- `link-framework`: Link the library as a framework on macOS.
- `build-from-source`: Fetch and build the library from source.
- `build-from-source-static`: Fetch and build the library from source and link it statically.

You can read more about these options [here](https://github.com/maia-s/sdl3-sys-rs/tree/main/sdl3-sys#usage).

# Documentation

- [SDL3 higher-level documentation](https://docs.rs/sdl3/).
- [SDL3-sys lower-level bindings documentation](https://docs.rs/sdl3-sys/latest/sdl3_sys/)

# Contributing

We're looking for people to help get SDL3 support in Rust built, tested, and completed. You can help out!

Many examples and documentation requires updating. Interfaces have changed from SDL2 to SDL3, and the Rust bindings need
to be updated to reflect these changes.

If you see anything wrong, missing, or suboptimal, please feel free to open a PR with your improvements.

If you would like to discuss ideas or ask questions, join the #rust channel on [Discord](https://discord.gg/qMyEpKVnCD).

# History

This project was forked from [Rust-sdl2](https://github.com/Rust-sdl2/rust-sdl2) and the SDL2 code migrated to SDL3
according to the [SDL2->SDL3 migration guide](https://github.com/libsdl-org/SDL/blob/main/docs/README-migration.md).

If you want a library compatible with earlier versions of SDL, please
see [Rust-sdl2](https://github.com/Rust-sdl2/rust-sdl2).
