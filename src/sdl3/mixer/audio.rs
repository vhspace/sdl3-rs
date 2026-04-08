use std::marker::PhantomData;

use crate::{get_error, Error};
use sdl3_sys::audio::SDL_AudioSpec;
use sdl3_sys::stdinc::Sint64;

use super::sys;

/// Loaded audio data that can be assigned to tracks for playback.
///
/// Audio objects are independent of any particular mixer — they can be shared
/// across multiple tracks and even multiple mixers. The C library internally
/// reference-counts audio data assigned to tracks, so it's safe to drop an
/// `Audio` while tracks still reference it.
pub struct Audio {
    raw: *mut sys::MIX_Audio,
    _marker: PhantomData<*mut ()>, // !Send + !Sync
}

impl Audio {
    pub(crate) fn from_raw(raw: *mut sys::MIX_Audio) -> Self {
        Audio {
            raw,
            _marker: PhantomData,
        }
    }

    /// Get a pointer to the underlying `MIX_Audio`.
    #[inline]
    pub fn raw(&self) -> *mut sys::MIX_Audio {
        self.raw
    }

    /// Get the total duration of this audio in sample frames.
    ///
    /// Returns a negative value if the duration is unknown or infinite.
    #[doc(alias = "MIX_GetAudioDuration")]
    pub fn duration(&self) -> i64 {
        unsafe { sys::MIX_GetAudioDuration(self.raw) as i64 }
    }

    /// Get the audio format of this audio data.
    #[doc(alias = "MIX_GetAudioFormat")]
    pub fn format(&self) -> Result<SDL_AudioSpec, Error> {
        let mut spec = SDL_AudioSpec {
            format: sdl3_sys::audio::SDL_AudioFormat::default(),
            channels: 0,
            freq: 0,
        };
        let ok = unsafe { sys::MIX_GetAudioFormat(self.raw, &mut spec) };
        if ok {
            Ok(spec)
        } else {
            Err(get_error())
        }
    }

    /// Convert milliseconds to sample frames for this audio.
    #[doc(alias = "MIX_AudioMSToFrames")]
    pub fn ms_to_frames(&self, ms: i64) -> i64 {
        unsafe { sys::MIX_AudioMSToFrames(self.raw, ms as Sint64) as i64 }
    }

    /// Convert sample frames to milliseconds for this audio.
    #[doc(alias = "MIX_AudioFramesToMS")]
    pub fn frames_to_ms(&self, frames: i64) -> i64 {
        unsafe { sys::MIX_AudioFramesToMS(self.raw, frames as Sint64) as i64 }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        unsafe { sys::MIX_DestroyAudio(self.raw) };
    }
}
