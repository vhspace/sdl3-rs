//!
//! Bindings for the (currently legacy) `SDL_gfx` primitives.
//!
//! SDL3 does not yet ship an `SDL3_gfx` release, so the `gfx` feature is blocked
//! until upstream provides compatible headers and libraries. These bindings stay
//! here as a reference for when that work lands.
//!
//!
//! Note that you need to build with the
//! feature `gfx` for this module to be enabled,
//! like so:
//!
//! ```bash
//! $ cargo build --features "gfx"
//! ```
//!
//! If you want to use this with from inside your own
//! crate, you will need to add this in your Cargo.toml
//!
//! ```toml
//! [dependencies.sdl3]
//! version = ...
//! default-features = false
//! features = ["gfx"]
//! ```

compile_error!(
    "The 'gfx' feature is non-functional. SDL_gfx has not been ported to SDL3 yet. \
     See: https://github.com/vhspace/sdl3-rs/issues/160"
);

pub mod framerate;
pub mod imagefilter;
pub mod primitives;
pub mod rotozoom;
