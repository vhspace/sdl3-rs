# SDL3 [![Crates.io Version](https://img.shields.io/crates/v/sdl3)](https://crates.io/crates/sdl3) [![](https://dcbadge.limes.pink/api/server/https://discord.gg/RE93FmF6Um?style=flat)](https://discord.gg/RE93FmF6Um)



Bindings for SDL3 in Rust.

SDL is the [Simple Directmedia Library](https://www.libsdl.org/), a cross-platform library to
abstract the platform-specific details for building applications. It takes care of everything
from handling events, creating windows, playing audio, accessing device cameras and sensors,
locking, GPU access, and much more. See more here: [https://wiki.libsdl.org/SDL3/APIByCategory](https://wiki.libsdl.org/SDL3/APIByCategory).

SDL officially supports Windows, macOS, Linux, iOS, and Android, and several other platforms.

## Migration Progress

Now that the SDL3 API is mostly stabilized, we are working on a new version of the Rust bindings for SDL3.
The migration is in progress, and we are looking for contributors to help us complete it.

Expect some bugs and [missing features](https://wiki.libsdl.org/SDL3/NewFeatures).
Feel free to create issues or work on them yourself.

- [x] Update all modules to SDL3, use new [sdl3-sys](https://github.com/maia-s/sdl3-sys-rs) bindings,
      follow [migration guide](https://github.com/libsdl-org/SDL/blob/main/docs/README-migration.md).
- [x] Fix tests.
- [x] Update examples to SDL3.
- [ ] Add [new features](https://wiki.libsdl.org/SDL3/NewFeatures) from SDL3.
- [ ] Update documentation.

Please refer to the [sdl3-rs](https://github.com/vhspace/sdl3-rs) repository for the latest updates.

The low-level bindings are being worked on in the [sdl3-sys](https://github.com/maia-s/sdl3-sys-rs) repository.

# Overview

This is an interface to use SDL3 from Rust.

Low-level C components are wrapped in Rust code to make them more idiomatic and
abstract away inappropriate manual memory management.

## Quick Start

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sdl3 = { version = "0", features = [] }
```

### [Read the documentation](https://docs.rs/sdl3/latest/sdl3/)

Or if you would like to open it offline, run `cargo doc --package sdl3 --open`

# Documentation

- [SDL3 higher-level documentation](https://docs.rs/sdl3/).
- [SDL3-sys lower-level bindings documentation](https://docs.rs/sdl3-sys/latest/sdl3_sys/)

# Extension Libraries
Not all [SDL3 extension libraries](https://wiki.libsdl.org/SDL3/Libraries) have full support in sdl3-rs yet.
| Library  |  |
| ------------- | ------------- |
| Dear ImGUI  | ❌ Not currently supported  |
| RmlUI  | ❌ Not currently supported  |
| SDL_gfx  | 🟨 Waiting on improvements to the C library  |
| SDL_image  | ✅ Supported  |
| SDL_mixer  | 🟨 Awaiting a stable C release  |
| SDL_sound  | 🟨 Awaiting a stable C release   |
| SDL_ttf  | ✅ Supported  |
| SDL_net  | 🟨 Awaiting a stable C release  |
| SDL_shadercross  | ❌ Not currently supported  |
| sdl3-main | ✅ Supported  |

# Feature Flags

## `ffi-safe`

Enables FFI-safe subsystem handles for hot-reloading scenarios.

```toml
[dependencies]
sdl3 = { version = "0", features = ["ffi-safe"] }
```

By default, subsystem handles (like `VideoSubsystem`) use static reference counters.
This is efficient but breaks when handles are passed across DLL/shared library boundaries
during hot-reloading, because each compilation unit gets its own copy of the static.

With `ffi-safe` enabled, subsystem handles use heap-allocated reference counters and
`#[repr(C)]` layout, making them safe to pass across FFI boundaries.

**Limitations:**
- The main `Sdl` context still uses static counters and must remain in the host binary
- Only subsystem handles should be passed to hot-reloaded code
- The `Sdl` context must outlive all subsystems

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

# Screenshots

<img width="873" alt="Screenshot 2024-10-22 at 1 13 20 PM" src="https://github.com/user-attachments/assets/3f2b7399-b8fd-4dc7-9d09-7fa04eff0e8a">
