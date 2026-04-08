use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use crate::iostream::IOStream;
use crate::properties::Properties;
use crate::{get_error, Error};
use sdl3_sys::audio::{SDL_AudioDeviceID, SDL_AudioSpec, SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK};
use sdl3_sys::properties::SDL_PropertiesID;
use sdl3_sys::stdinc::Sint64;

use super::audio::Audio;
use super::group::Group;
use super::track::Track;
use super::{bool_result, sys, to_cstring, MixerContext};

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
        bool_result(unsafe { sys::MIX_SetMixerGain(self.raw, gain) })
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
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| Error("path contains invalid UTF-8".into()))?;
        let c_path = to_cstring(path_str)?;
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
        bool_result(unsafe { sys::MIX_PlayAudio(self.raw, audio.raw()) })
    }

    /// Stop all tracks on this mixer, with optional fade-out in milliseconds.
    #[doc(alias = "MIX_StopAllTracks")]
    pub fn stop_all(&self, fade_out_ms: i64) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_StopAllTracks(self.raw, fade_out_ms as Sint64) })
    }

    /// Pause all tracks on this mixer.
    #[doc(alias = "MIX_PauseAllTracks")]
    pub fn pause_all(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_PauseAllTracks(self.raw) })
    }

    /// Resume all paused tracks on this mixer.
    #[doc(alias = "MIX_ResumeAllTracks")]
    pub fn resume_all(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_ResumeAllTracks(self.raw) })
    }

    /// Stop all tracks with a specific tag, with optional fade-out in milliseconds.
    #[doc(alias = "MIX_StopTag")]
    pub fn stop_tag(&self, tag: &str, fade_out_ms: i64) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        bool_result(unsafe { sys::MIX_StopTag(self.raw, c_tag.as_ptr(), fade_out_ms as Sint64) })
    }

    /// Pause all tracks with a specific tag.
    #[doc(alias = "MIX_PauseTag")]
    pub fn pause_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        bool_result(unsafe { sys::MIX_PauseTag(self.raw, c_tag.as_ptr()) })
    }

    /// Resume all tracks with a specific tag.
    #[doc(alias = "MIX_ResumeTag")]
    pub fn resume_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        bool_result(unsafe { sys::MIX_ResumeTag(self.raw, c_tag.as_ptr()) })
    }

    // -- Frequency ratio --

    /// Set the master frequency ratio for the entire mix.
    ///
    /// 1.0 is normal speed, 2.0 is double speed, 0.5 is half speed.
    #[doc(alias = "MIX_SetMixerFrequencyRatio")]
    pub fn set_frequency_ratio(&self, ratio: f32) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetMixerFrequencyRatio(self.raw, ratio) })
    }

    /// Get the master frequency ratio for the entire mix.
    #[doc(alias = "MIX_GetMixerFrequencyRatio")]
    pub fn frequency_ratio(&self) -> f32 {
        unsafe { sys::MIX_GetMixerFrequencyRatio(self.raw) }
    }

    // -- Tag-based gain --

    /// Set the gain for all tracks with a specific tag.
    #[doc(alias = "MIX_SetTagGain")]
    pub fn set_tag_gain(&self, tag: &str, gain: f32) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        bool_result(unsafe { sys::MIX_SetTagGain(self.raw, c_tag.as_ptr(), gain) })
    }

    // -- Tag-based playback --

    /// Start playing all tracks with a specific tag.
    ///
    /// Pass `None` for options to use default playback properties, or create
    /// a `Properties` object and set `MIX_PROP_PLAY_*` keys for advanced control.
    #[doc(alias = "MIX_PlayTag")]
    pub fn play_tag(&self, tag: &str, options: Option<&Properties>) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        let props = options.map_or(SDL_PropertiesID(0), |p| p.raw());
        bool_result(unsafe { sys::MIX_PlayTag(self.raw, c_tag.as_ptr(), props) })
    }

    // -- Tag queries --

    /// Get the raw track pointers for all tracks with a specific tag.
    ///
    /// Returns raw pointers that can be compared with `track.raw()` to identify
    /// specific tracks. The returned pointers are borrowed from the mixer and
    /// must not be destroyed.
    #[doc(alias = "MIX_GetTaggedTracks")]
    pub fn tagged_tracks(&self, tag: &str) -> Vec<*mut sys::MIX_Track> {
        let c_tag = to_cstring(tag).unwrap_or_default();
        let mut count: std::ffi::c_int = 0;
        let ptr = unsafe { sys::MIX_GetTaggedTracks(self.raw, c_tag.as_ptr(), &mut count) };
        if ptr.is_null() || count <= 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count as isize {
            unsafe {
                let track = *ptr.offset(i);
                if !track.is_null() {
                    result.push(track);
                }
            }
        }
        unsafe { sdl3_sys::stdinc::SDL_free(ptr as *mut _) };
        result
    }

    // -- Properties --

    /// Get the properties associated with this mixer.
    ///
    /// The returned properties object is read-only.
    #[doc(alias = "MIX_GetMixerProperties")]
    pub fn properties(&self) -> Properties {
        let id = unsafe { sys::MIX_GetMixerProperties(self.raw) };
        Properties::const_from_ll(id)
    }

    // -- IOStream loading --

    /// Load audio from an IOStream.
    ///
    /// If `predecode` is true, the audio will be fully decompressed into memory.
    /// Otherwise it will be decoded on the fly during playback.
    #[doc(alias = "MIX_LoadAudio_IO")]
    pub fn load_audio_io(&self, io: &IOStream, predecode: bool) -> Result<Audio, Error> {
        let raw = unsafe { sys::MIX_LoadAudio_IO(self.raw, io.raw(), predecode, false) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio::from_raw(raw))
        }
    }

    /// Load raw PCM audio data from a byte slice.
    ///
    /// The data is copied internally, so the slice does not need to outlive
    /// the returned `Audio`.
    #[doc(alias = "MIX_LoadRawAudio")]
    pub fn load_raw_audio(&self, data: &[u8], spec: &SDL_AudioSpec) -> Result<Audio, Error> {
        let raw =
            unsafe { sys::MIX_LoadRawAudio(self.raw, data.as_ptr() as *const _, data.len(), spec) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio::from_raw(raw))
        }
    }

    /// Load raw PCM audio from an IOStream.
    #[doc(alias = "MIX_LoadRawAudio_IO")]
    pub fn load_raw_audio_io(&self, io: &IOStream, spec: &SDL_AudioSpec) -> Result<Audio, Error> {
        let raw = unsafe { sys::MIX_LoadRawAudio_IO(self.raw, io.raw(), spec, false) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio::from_raw(raw))
        }
    }

    // -- Memory-only mixer --

    /// Create a mixer that renders to memory instead of an audio device.
    ///
    /// Use `generate` to pull mixed audio into a buffer. Pass `None` for `spec`
    /// to let SDL choose the best format.
    #[doc(alias = "MIX_CreateMixer")]
    pub fn create_memory(spec: Option<&SDL_AudioSpec>) -> Result<Mixer, Error> {
        let context = MixerContext::new()?;
        let spec_ptr = spec
            .map(|s| s as *const SDL_AudioSpec)
            .unwrap_or(ptr::null());
        let raw = unsafe { sys::MIX_CreateMixer(spec_ptr) };
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

    /// Generate mixed audio into a buffer.
    ///
    /// Only valid for memory-only mixers created with `create_memory`.
    /// Returns the number of bytes written, or a negative value on error.
    #[doc(alias = "MIX_Generate")]
    pub fn generate(&self, buffer: &mut [u8]) -> i32 {
        unsafe { sys::MIX_Generate(self.raw, buffer.as_mut_ptr() as *mut _, buffer.len() as i32) }
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
