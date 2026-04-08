//!
//! Bindings for the `SDL3_mixer` extension library.
//!
//! SDL3_mixer provides audio mixing, playback, and effects for SDL3-based
//! applications. It supports loading many audio formats (WAV, MP3, OGG, FLAC,
//! etc.), mixing multiple sounds simultaneously, basic 3D positional audio,
//! and various audio effects.
//!
//! Note that you need to build with the `mixer` feature for this module to be
//! enabled:
//!
//! ```bash
//! $ cargo build --features "mixer"
//! ```
//!
//! If you want to use this from inside your own crate, add this to your
//! Cargo.toml:
//!
//! ```toml
//! [dependencies.sdl3]
//! version = "..."
//! default-features = false
//! features = ["mixer"]
//! ```

mod audio;
mod device;
mod group;
mod track;

pub use sdl3_mixer_sys::mixer as sys;

pub use self::audio::Audio;
pub use self::device::{Mixer, MixerLock};
pub use self::group::Group;
pub use self::track::{Point3D, StereoGains, Track};

use crate::{get_error, Error};
use std::ffi::CStr;
use std::sync::atomic::{AtomicU32, Ordering};

/// A context manager for `SDL3_mixer` to manage library init and quit.
///
/// SDL3_mixer is reference-counted internally, so multiple contexts can exist.
/// The library is only deinitialized when the last context is dropped.
#[must_use]
#[derive(Clone)]
pub struct MixerContext;

static MIXER_COUNT: AtomicU32 = AtomicU32::new(0);

impl MixerContext {
    fn new() -> Result<Self, Error> {
        if MIXER_COUNT.fetch_add(1, Ordering::Relaxed) == 0 {
            let result = unsafe { sys::MIX_Init() };
            if !result {
                MIXER_COUNT.store(0, Ordering::Relaxed);
                return Err(get_error());
            }
        }
        Ok(Self)
    }
}

impl Drop for MixerContext {
    fn drop(&mut self) {
        let prev_count = MIXER_COUNT.fetch_sub(1, Ordering::Relaxed);
        assert!(prev_count > 0);
        if prev_count == 1 {
            unsafe {
                sys::MIX_Quit();
            }
        }
    }
}

/// Initialize the SDL3_mixer library and return a context manager.
///
/// The context will clean up the library when dropped. Multiple calls are
/// safe — the library uses internal reference counting.
#[doc(alias = "MIX_Init")]
pub fn init() -> Result<MixerContext, Error> {
    MixerContext::new()
}

/// Get the version of the dynamically linked SDL_mixer library.
#[doc(alias = "MIX_Version")]
pub fn get_linked_version() -> i32 {
    sys::MIX_Version()
}

/// Get the number of audio decoders available.
#[doc(alias = "MIX_GetNumAudioDecoders")]
pub fn get_num_audio_decoders() -> i32 {
    unsafe { sys::MIX_GetNumAudioDecoders() }
}

/// Get the name of a specific audio decoder by index.
///
/// Returns `None` if the index is out of range.
#[doc(alias = "MIX_GetAudioDecoder")]
pub fn get_audio_decoder(index: i32) -> Option<String> {
    unsafe {
        let name = sys::MIX_GetAudioDecoder(index);
        if name.is_null() {
            None
        } else {
            Some(
                CStr::from_ptr(name)
                    .to_str()
                    .unwrap_or("(invalid UTF-8)")
                    .to_owned(),
            )
        }
    }
}
