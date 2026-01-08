//! # Getting started
//!
//! ```rust,no_run
//! extern crate sdl3;
//!
//! use sdl3::pixels::Color;
//! use sdl3::event::Event;
//! use sdl3::keyboard::Keycode;
//! use std::time::Duration;
//!
//! pub fn main() {
//!     let sdl_context = sdl3::init().unwrap();
//!     let video_subsystem = sdl_context.video().unwrap();
//!
//!     let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
//!         .position_centered()
//!         .build()
//!         .unwrap();
//!
//!     let mut canvas = window.into_canvas();
//!
//!     canvas.set_draw_color(Color::RGB(0, 255, 255));
//!     canvas.clear();
//!     canvas.present();
//!     let mut event_pump = sdl_context.event_pump().unwrap();
//!     let mut i = 0;
//!     'running: loop {
//!         i = (i + 1) % 255;
//!         canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
//!         canvas.clear();
//!         for event in event_pump.poll_iter() {
//!             match event {
//!                 Event::Quit {..} |
//!                 Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//!                     break 'running
//!                 },
//!                 _ => {}
//!             }
//!         }
//!         // The rest of the game loop goes here...
//!
//!         canvas.present();
//!         ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
//!     }
//! }
//! ```
//!
//! # Feature Flags
//!
//! ## Linking to `libsdl3`
//!
//! This crate requires the SDL3 library to link and run.
//!
//! By default without any of these features enabled, it will try to link a system SDL3 library as a dynamic/shared library
//! using the default library search paths.
//!
//! If you don't have `libsdl3` installed on your system and just want it to work, we recommend using `build-from-source`.
//! Also see the different build configurations that `sdl3-sys` provides:  <https://github.com/maia-s/sdl3-sys-rs/tree/main/sdl3-sys#usage>
//!
//! | Name                             | Description                                                    |
//! |----------------------------------|----------------------------------------------------------------|
//! | `build-from-source`              | Fetch and build the library from source                        |
//! | `build-from-source-static`       | Fetch and build the library from source and link it statically |
//! | `build-from-source-unix-console` | Build SDL3 from source but skip the X11/Wayland dependencies for console targets |
//! | `use-pkg-config`                 | Use `pkg-config` to find the library                           |
//! | `use-vcpkg`                      | Use `vcpkg` to find the library                                |
//! | `static-link`                    | Link the library statically                                    |
//! | `link-framework`                 | Link the library as a framework on macOS                       |
//!
//! ## Optional Features
//!
//! Note that since `sdl3` is still in the progress of migrating to and integrating the new
//! features of `libsdl3`, some features might be not yet, or only partially implemented.
//!
//! | Name                | Description                                                            | Implementation Status |
//! |---------------------|------------------------------------------------------------------------|-----------------------|
//! | `ash`               | Use Vulkan types from the ash crate                                    | Implemented           |
//! | `unsafe_textures`   | Skip lifetime tracking for textures; you must manage destruction safety yourself | Implemented (unsafe opt-in) |
//! | `gfx`               | Legacy SDL_gfx drawing helpers; blocked on an SDL3_gfx C library       | Blocked               |
//! | `mixer`             | SDL_mixer bindings (needs upstream SDL3_mixer and `sdl3-mixer-sys`)    | Blocked               |
//! | `image`             | Enable SDL_image helpers for loading/saving surfaces and textures      | Implemented           |
//! | `ttf`               | Enable SDL_ttf font/text rendering APIs                                | Implemented           |
//! | `hidapi`            | Use SDL's hidapi backend for sensors and controllers                   | Implemented           |
//! | `test-mode`         | Allows SDL to be initialised from a thread that is not the main thread | Implemented           |
//! | `raw-window-handle` | Enables integrations with the [`wgpu`] crate                           | Implemented           |
//! | `main`              | Enables integrations with the [`sdl3-main`] crate (main callbacks api) | Implemented           |
//!
//! [`wgpu`]: https://docs.rs/wgpu/latest/wgpu/
//! [`sdl3-main`]: https://docs.rs/sdl3-main/latest/sdl3_main/

#![crate_name = "sdl3"]
#![crate_type = "lib"]
#![allow(
    clippy::cast_lossless,
    clippy::transmute_ptr_to_ref,
    clippy::missing_transmute_annotations,
    clippy::missing_safety_doc
)]

#[macro_use]
extern crate bitflags;
#[cfg(feature = "gfx")]
extern crate c_vec;
#[macro_use]
extern crate lazy_static;
pub extern crate libc;
pub extern crate sdl3_sys as sys;
// use sdl3_sys as sys;

pub use crate::sdl::*;

pub mod clipboard;
pub mod cpuinfo;
#[macro_use]
mod macros;
pub mod audio;
pub mod dialog;
pub mod event;
pub mod filesystem;
pub mod gamepad;
pub mod gpu;
pub mod haptic;
pub mod hint;
pub mod iostream;
pub mod joystick;
pub mod keyboard;
pub mod log;
pub mod messagebox;
pub mod mouse;
pub mod pen;
pub mod pixels;
pub mod properties;
pub mod rect;
pub mod render;
mod sdl;
#[cfg(feature = "hidapi")]
pub mod sensor;
pub mod surface;
pub mod timer;
pub mod touch;
pub mod url;
pub mod version;
pub mod video;

// modules
#[cfg(feature = "gfx")]
pub mod gfx;
#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "mixer")]
pub mod mixer;
#[cfg(feature = "ttf")]
pub mod ttf;

#[cfg(feature = "main")]
mod sdlmain; // this just implements traits, so it doesn't have to be public

mod common;
// Export return types and such from the common module.
pub use crate::common::IntegerOrSdlError;

mod guid;
#[cfg(feature = "raw-window-handle")]
pub mod raw_window_handle;
mod util;
