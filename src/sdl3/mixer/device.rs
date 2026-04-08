use std::ffi::CString;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use crate::{get_error, Error};
use sdl3_sys::audio::{SDL_AudioDeviceID, SDL_AudioSpec, SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK};
use sdl3_sys::stdinc::Sint64;

use super::audio::Audio;
use super::group::Group;
use super::track::Track;
use super::{sys, MixerContext};

/// A mixer device that plays sound to an audio device.
///
/// This is the main entry point for SDL3_mixer. Create a `Mixer` to open an
/// audio device, then load audio and create tracks to play sounds.
pub struct Mixer {
    raw: *mut sys::MIX_Mixer,
    _context: MixerContext,
    _marker: PhantomData<*mut ()>, // !Send + !Sync
}

impl Mixer {
    /// Create a mixer that plays to the default audio device.
    ///
    /// Pass `None` for `spec` to let SDL choose the best format.
    #[doc(alias = "MIX_CreateMixerDevice")]
    pub fn open_device(spec: Option<&SDL_AudioSpec>) -> Result<Mixer, Error> {
        Self::open_device_id(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, spec)
    }

    /// Create a mixer that plays to a specific audio device.
    #[doc(alias = "MIX_CreateMixerDevice")]
    pub fn open_device_id(
        device_id: SDL_AudioDeviceID,
        spec: Option<&SDL_AudioSpec>,
    ) -> Result<Mixer, Error> {
        let context = MixerContext::new()?;
        let spec_ptr = spec
            .map(|s| s as *const SDL_AudioSpec)
            .unwrap_or(ptr::null());
        let raw = unsafe { sys::MIX_CreateMixerDevice(device_id, spec_ptr) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Mixer {
                raw,
                _context: context,
                _marker: PhantomData,
            })
        }
    }

    /// Get a pointer to the underlying `MIX_Mixer`.
    #[inline]
    pub fn raw(&self) -> *mut sys::MIX_Mixer {
        self.raw
    }

    /// Get the actual audio format in use by this mixer.
    #[doc(alias = "MIX_GetMixerFormat")]
    pub fn format(&self) -> Result<SDL_AudioSpec, Error> {
        let mut spec = SDL_AudioSpec {
            format: sdl3_sys::audio::SDL_AudioFormat::default(),
            channels: 0,
            freq: 0,
        };
        let ok = unsafe { sys::MIX_GetMixerFormat(self.raw, &mut spec) };
        if ok {
            Ok(spec)
        } else {
            Err(get_error())
        }
    }

    /// Set the master gain (volume) for the entire mix.
    ///
    /// A gain of 0.0 is silence, 1.0 is unchanged, >1.0 amplifies.
    #[doc(alias = "MIX_SetMixerGain")]
    pub fn set_gain(&self, gain: f32) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_SetMixerGain(self.raw, gain) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the master gain (volume) for the entire mix.
    #[doc(alias = "MIX_GetMixerGain")]
    pub fn gain(&self) -> f32 {
        unsafe { sys::MIX_GetMixerGain(self.raw) }
    }

    /// Lock the mixer for thread-safe access.
    ///
    /// Returns an RAII guard that unlocks on drop.
    #[doc(alias = "MIX_LockMixer")]
    pub fn lock(&self) -> MixerLock<'_> {
        unsafe { sys::MIX_LockMixer(self.raw) };
        MixerLock { mixer: self }
    }

    /// Create a new track on this mixer.
    #[doc(alias = "MIX_CreateTrack")]
    pub fn create_track(&self) -> Result<Track<'_>, Error> {
        let raw = unsafe { sys::MIX_CreateTrack(self.raw) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Track::from_raw(raw))
        }
    }

    /// Create a new group on this mixer.
    #[doc(alias = "MIX_CreateGroup")]
    pub fn create_group(&self) -> Result<Group<'_>, Error> {
        let raw = unsafe { sys::MIX_CreateGroup(self.raw) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Group::from_raw(raw))
        }
    }

    /// Load audio from a file path.
    ///
    /// If `predecode` is true, the audio will be fully decompressed into memory.
    /// Otherwise it will be decoded on the fly during playback.
    #[doc(alias = "MIX_LoadAudio")]
    pub fn load_audio<P: AsRef<Path>>(&self, path: P, predecode: bool) -> Result<Audio, Error> {
        let c_path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
        let raw = unsafe { sys::MIX_LoadAudio(self.raw, c_path.as_ptr(), predecode) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio::from_raw(raw))
        }
    }

    /// Create a sine wave audio source for testing or debugging.
    ///
    /// - `hz`: frequency of the sine wave in Hz
    /// - `amplitude`: volume from 0.0 (silent) to 1.0 (loud)
    /// - `ms`: duration in milliseconds, or negative for infinite
    #[doc(alias = "MIX_CreateSineWaveAudio")]
    pub fn create_sine_wave(&self, hz: i32, amplitude: f32, ms: i64) -> Result<Audio, Error> {
        let raw = unsafe { sys::MIX_CreateSineWaveAudio(self.raw, hz, amplitude, ms as Sint64) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio::from_raw(raw))
        }
    }

    /// Play audio once from start to finish without any management ("fire and forget").
    ///
    /// Internally, SDL_mixer creates a temporary track, plays it, and cleans up
    /// when done.
    #[doc(alias = "MIX_PlayAudio")]
    pub fn play_audio(&self, audio: &Audio) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_PlayAudio(self.raw, audio.raw()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Stop all tracks on this mixer, with optional fade-out in milliseconds.
    #[doc(alias = "MIX_StopAllTracks")]
    pub fn stop_all(&self, fade_out_ms: i64) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_StopAllTracks(self.raw, fade_out_ms as Sint64) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Pause all tracks on this mixer.
    #[doc(alias = "MIX_PauseAllTracks")]
    pub fn pause_all(&self) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_PauseAllTracks(self.raw) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resume all paused tracks on this mixer.
    #[doc(alias = "MIX_ResumeAllTracks")]
    pub fn resume_all(&self) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_ResumeAllTracks(self.raw) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Stop all tracks with a specific tag, with optional fade-out in milliseconds.
    #[doc(alias = "MIX_StopTag")]
    pub fn stop_tag(&self, tag: &str, fade_out_ms: i64) -> Result<(), Error> {
        let c_tag = CString::new(tag).unwrap();
        let ok = unsafe { sys::MIX_StopTag(self.raw, c_tag.as_ptr(), fade_out_ms as Sint64) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Pause all tracks with a specific tag.
    #[doc(alias = "MIX_PauseTag")]
    pub fn pause_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).unwrap();
        let ok = unsafe { sys::MIX_PauseTag(self.raw, c_tag.as_ptr()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resume all tracks with a specific tag.
    #[doc(alias = "MIX_ResumeTag")]
    pub fn resume_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).unwrap();
        let ok = unsafe { sys::MIX_ResumeTag(self.raw, c_tag.as_ptr()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        unsafe { sys::MIX_DestroyMixer(self.raw) };
    }
}

/// RAII guard for a locked mixer. Unlocks on drop.
pub struct MixerLock<'a> {
    mixer: &'a Mixer,
}

impl Drop for MixerLock<'_> {
    fn drop(&mut self) {
        unsafe { sys::MIX_UnlockMixer(self.mixer.raw) };
    }
}
