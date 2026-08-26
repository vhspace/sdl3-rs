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

// Re-export property key constants for play options, audio loading, and metadata.
pub use sys::{
    MIX_DURATION_INFINITE, MIX_DURATION_UNKNOWN, MIX_PROP_AUDIO_DECODER_STRING,
    MIX_PROP_AUDIO_LOAD_CLOSEIO_BOOLEAN, MIX_PROP_AUDIO_LOAD_IOSTREAM_POINTER,
    MIX_PROP_AUDIO_LOAD_PREDECODE_BOOLEAN, MIX_PROP_AUDIO_LOAD_PREFERRED_MIXER_POINTER,
    MIX_PROP_AUDIO_LOAD_SKIP_METADATA_TAGS_BOOLEAN, MIX_PROP_METADATA_ALBUM_STRING,
    MIX_PROP_METADATA_ARTIST_STRING, MIX_PROP_METADATA_COPYRIGHT_STRING,
    MIX_PROP_METADATA_DURATION_FRAMES_NUMBER, MIX_PROP_METADATA_DURATION_INFINITE_BOOLEAN,
    MIX_PROP_METADATA_TITLE_STRING, MIX_PROP_METADATA_TOTAL_TRACKS_NUMBER,
    MIX_PROP_METADATA_TRACK_NUMBER, MIX_PROP_METADATA_YEAR_NUMBER, MIX_PROP_MIXER_DEVICE_NUMBER,
    MIX_PROP_PLAY_APPEND_SILENCE_FRAMES_NUMBER, MIX_PROP_PLAY_APPEND_SILENCE_MILLISECONDS_NUMBER,
    MIX_PROP_PLAY_FADE_IN_FRAMES_NUMBER, MIX_PROP_PLAY_FADE_IN_MILLISECONDS_NUMBER,
    MIX_PROP_PLAY_FADE_IN_START_GAIN_FLOAT, MIX_PROP_PLAY_HALT_WHEN_EXHAUSTED_BOOLEAN,
    MIX_PROP_PLAY_LOOPS_NUMBER, MIX_PROP_PLAY_LOOP_START_FRAME_NUMBER,
    MIX_PROP_PLAY_LOOP_START_MILLISECOND_NUMBER, MIX_PROP_PLAY_MAX_FRAME_NUMBER,
    MIX_PROP_PLAY_MAX_MILLISECONDS_NUMBER, MIX_PROP_PLAY_START_FRAME_NUMBER,
    MIX_PROP_PLAY_START_MILLISECOND_NUMBER,
};

use crate::{get_error, Error};
use sdl3_sys::stdinc::Sint64;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;

/// A context manager for `SDL3_mixer` to manage library init and quit.
///
/// SDL3_mixer is reference-counted internally, so multiple contexts can exist.
/// The library is only deinitialized when `MIX_Quit()` has been called a
/// matching number of times.
///
/// `MIX_Init` and `MIX_Quit` are not thread-safe, so this type is `!Send`
/// and `!Sync`.
#[must_use]
pub struct MixerContext {
    _marker: PhantomData<*mut ()>, // !Send + !Sync
}

impl Clone for MixerContext {
    fn clone(&self) -> Self {
        // MIX_Init is safe to call multiple times; the C library ref-counts internally.
        // After the first successful init, subsequent calls always succeed.
        let ok = unsafe { sys::MIX_Init() };
        assert!(
            ok,
            "MIX_Init failed during clone: this should not happen after initial init succeeded"
        );
        MixerContext {
            _marker: PhantomData,
        }
    }
}

impl MixerContext {
    fn new() -> Result<Self, Error> {
        let result = unsafe { sys::MIX_Init() };
        if !result {
            return Err(get_error());
        }
        Ok(MixerContext {
            _marker: PhantomData,
        })
    }
}

/// Convert an SDL bool return to a Rust Result.
fn bool_result(ok: bool) -> Result<(), Error> {
    if ok {
        Ok(())
    } else {
        Err(get_error())
    }
}

/// Create a CString from a &str, returning an Error on invalid input.
fn to_cstring(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|e| Error(e.to_string()))
}

impl Drop for MixerContext {
    fn drop(&mut self) {
        unsafe { sys::MIX_Quit() };
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

/// Convert milliseconds to sample frames at a given sample rate.
///
/// This is a standalone helper that doesn't require a track or audio object.
#[doc(alias = "MIX_MSToFrames")]
pub fn ms_to_frames(sample_rate: i32, ms: i64) -> i64 {
    unsafe { sys::MIX_MSToFrames(sample_rate, ms as Sint64) as i64 }
}

/// Convert sample frames to milliseconds at a given sample rate.
///
/// This is a standalone helper that doesn't require a track or audio object.
#[doc(alias = "MIX_FramesToMS")]
pub fn frames_to_ms(sample_rate: i32, frames: i64) -> i64 {
    unsafe { sys::MIX_FramesToMS(sample_rate, frames as Sint64) as i64 }
}
