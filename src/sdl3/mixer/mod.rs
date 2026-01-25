//!
//! Bindings for the `SDL3_mixer` extension library.
//!
//! SDL3_mixer has a completely new track-based API compared to SDL2_mixer.
//! Instead of numbered channels, you create tracks that can be assigned audio
//! data and controlled independently.
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use sdl3::mixer;
//!
//! // Initialize the library
//! let _ctx = mixer::init().expect("Failed to init SDL3_mixer");
//!
//! // Create a mixer that plays to the default audio device
//! let mix = mixer::Mixer::open_default().expect("Failed to open mixer");
//!
//! // Load an audio file
//! let audio = mixer::Audio::from_file(&mix, "sound.wav").expect("Failed to load audio");
//!
//! // Create a track and play the audio
//! let track = mix.create_track().expect("Failed to create track");
//! track.set_audio(&audio).expect("Failed to set audio");
//! track.play().expect("Failed to play");
//! ```
//!
//! Note that you need to build with the feature `mixer` for this module to be enabled:
//!
//! ```bash
//! $ cargo build --features "mixer"
//! ```
//!
//! If you want to use this from inside your own crate, add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies.sdl3]
//! version = ...
//! default-features = false
//! features = ["mixer"]
//! ```

use crate::sys;
use crate::{get_error, Error};
use libc::{c_float, c_int};
use sdl3_mixer_sys::mixer;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use sys::audio::{SDL_AudioSpec, SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK};
use sys::properties::SDL_PropertiesID;

/// The version of SDL_mixer linked to this build.
pub fn get_version() -> i32 {
    mixer::MIX_Version()
}

/// Context manager for `SDL3_mixer` to manage init and quit.
///
/// When this is dropped, `MIX_Quit()` is called.
pub struct Sdl3MixerContext;

impl Drop for Sdl3MixerContext {
    fn drop(&mut self) {
        unsafe {
            mixer::MIX_Quit();
        }
    }
}

/// Initialize the SDL3_mixer library.
///
/// This must be called before using any other SDL_mixer functions (except `get_version()`).
/// Returns a context that will clean up the library when dropped.
pub fn init() -> Result<Sdl3MixerContext, Error> {
    let success = unsafe { mixer::MIX_Init() };
    if success {
        Ok(Sdl3MixerContext)
    } else {
        Err(get_error())
    }
}

/// Get the number of audio decoders available.
pub fn get_num_audio_decoders() -> i32 {
    unsafe { mixer::MIX_GetNumAudioDecoders() as i32 }
}

/// Get the name of an audio decoder by index.
pub fn get_audio_decoder(index: i32) -> Option<String> {
    unsafe {
        let name = mixer::MIX_GetAudioDecoder(index as c_int);
        if name.is_null() {
            None
        } else {
            Some(CStr::from_ptr(name).to_str().unwrap_or_default().to_owned())
        }
    }
}

/// A mixer that manages audio playback.
///
/// Create a mixer with `Mixer::open_default()` or `Mixer::open_device()` to play
/// to an audio device, or with `Mixer::new()` to generate audio to memory.
pub struct Mixer {
    raw: *mut mixer::MIX_Mixer,
}

impl Drop for Mixer {
    fn drop(&mut self) {
        unsafe {
            mixer::MIX_DestroyMixer(self.raw);
        }
    }
}

impl Mixer {
    /// Create a mixer that plays to the default audio device.
    pub fn open_default() -> Result<Mixer, Error> {
        let raw =
            unsafe { mixer::MIX_CreateMixerDevice(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, ptr::null()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Mixer { raw })
        }
    }

    /// Create a mixer that plays to a specific audio device.
    pub fn open_device(device_id: sys::audio::SDL_AudioDeviceID) -> Result<Mixer, Error> {
        let raw = unsafe { mixer::MIX_CreateMixerDevice(device_id, ptr::null()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Mixer { raw })
        }
    }

    /// Create a mixer that generates audio to memory (for use with `generate()`).
    ///
    /// Requires an audio spec to define the output format.
    pub fn new(spec: &SDL_AudioSpec) -> Result<Mixer, Error> {
        let raw = unsafe { mixer::MIX_CreateMixer(spec as *const _) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Mixer { raw })
        }
    }

    /// Get the raw pointer to the mixer.
    pub fn raw(&self) -> *mut mixer::MIX_Mixer {
        self.raw
    }

    /// Create a new track for this mixer.
    pub fn create_track(&self) -> Result<Track<'_>, Error> {
        let raw = unsafe { mixer::MIX_CreateTrack(self.raw) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Track {
                raw,
                _marker: PhantomData,
            })
        }
    }

    /// Create a new group for this mixer.
    pub fn create_group(&self) -> Result<Group<'_>, Error> {
        let raw = unsafe { mixer::MIX_CreateGroup(self.raw) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Group {
                raw,
                _marker: PhantomData,
            })
        }
    }

    /// Set the master gain (volume) for this mixer.
    ///
    /// Gain is a multiplier: 0.0 = silent, 1.0 = normal, 2.0 = double volume.
    pub fn set_gain(&self, gain: f32) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetMixerGain(self.raw, gain as c_float) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the master gain (volume) for this mixer.
    pub fn get_gain(&self) -> f32 {
        unsafe { mixer::MIX_GetMixerGain(self.raw) as f32 }
    }

    /// Set the frequency ratio for this mixer.
    ///
    /// This speeds up or slows down all audio, which also changes pitch.
    /// 1.0 = normal, 2.0 = double speed (one octave higher), 0.5 = half speed (one octave lower).
    pub fn set_frequency_ratio(&self, ratio: f32) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetMixerFrequencyRatio(self.raw, ratio as c_float) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the frequency ratio for this mixer.
    pub fn get_frequency_ratio(&self) -> f32 {
        unsafe { mixer::MIX_GetMixerFrequencyRatio(self.raw) as f32 }
    }

    /// Stop all tracks on this mixer with optional fade-out.
    ///
    /// `fade_out_ms` is the fade duration in milliseconds (0 for immediate stop).
    pub fn stop_all(&self, fade_out_ms: i64) {
        unsafe {
            mixer::MIX_StopAllTracks(self.raw, fade_out_ms);
        }
    }

    /// Pause all tracks on this mixer.
    pub fn pause_all(&self) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_PauseAllTracks(self.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resume all paused tracks on this mixer.
    pub fn resume_all(&self) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_ResumeAllTracks(self.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Play audio directly on this mixer (convenience method that creates an internal track).
    pub fn play_audio(&self, audio: &Audio) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_PlayAudio(self.raw, audio.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Stop all tracks with a specific tag.
    pub fn stop_tag(&self, tag: &str, fade_out_ms: i64) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        let success = unsafe { mixer::MIX_StopTag(self.raw, c_tag.as_ptr(), fade_out_ms) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Pause all tracks with a specific tag.
    pub fn pause_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        let success = unsafe { mixer::MIX_PauseTag(self.raw, c_tag.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resume all tracks with a specific tag.
    pub fn resume_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        let success = unsafe { mixer::MIX_ResumeTag(self.raw, c_tag.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Play all tracks with a specific tag using default options.
    pub fn play_tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        // Use 0 for SDL_PropertiesID to use default options
        let success = unsafe { mixer::MIX_PlayTag(self.raw, c_tag.as_ptr(), SDL_PropertiesID(0)) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Set gain for all tracks with a specific tag.
    pub fn set_tag_gain(&self, tag: &str, gain: f32) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        let success = unsafe { mixer::MIX_SetTagGain(self.raw, c_tag.as_ptr(), gain as c_float) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

// Safety: MIX_Mixer is designed to be accessed from multiple threads
unsafe impl Send for Mixer {}

/// Audio data that can be played through tracks.
///
/// Load audio with `Audio::from_file()` or `Audio::from_io()`.
/// Audio objects can be shared between mixers.
pub struct Audio {
    raw: *mut mixer::MIX_Audio,
}

impl Drop for Audio {
    fn drop(&mut self) {
        unsafe {
            mixer::MIX_DestroyAudio(self.raw);
        }
    }
}

impl Audio {
    /// Load audio from a file.
    ///
    /// The mixer is used to hint at the optimal format, but the audio can be
    /// used with any mixer.
    pub fn from_file<P: AsRef<Path>>(mixer: &Mixer, path: P) -> Result<Audio, Error> {
        let c_path = CString::new(
            path.as_ref()
                .to_str()
                .ok_or(Error("Invalid path".to_owned()))?,
        )
        .map_err(|_| Error("Invalid path".to_owned()))?;
        let raw = unsafe { mixer::MIX_LoadAudio(mixer.raw, c_path.as_ptr(), false) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio { raw })
        }
    }

    /// Load audio from a file and pre-decode it to PCM.
    ///
    /// This uses more memory but reduces CPU usage during playback.
    pub fn from_file_predecoded<P: AsRef<Path>>(mixer: &Mixer, path: P) -> Result<Audio, Error> {
        let c_path = CString::new(
            path.as_ref()
                .to_str()
                .ok_or(Error("Invalid path".to_owned()))?,
        )
        .map_err(|_| Error("Invalid path".to_owned()))?;
        let raw = unsafe { mixer::MIX_LoadAudio(mixer.raw, c_path.as_ptr(), true) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Audio { raw })
        }
    }

    /// Get the raw pointer to the audio.
    pub fn raw(&self) -> *mut mixer::MIX_Audio {
        self.raw
    }

    /// Get the duration of this audio in sample frames.
    pub fn duration(&self) -> i64 {
        unsafe { mixer::MIX_GetAudioDuration(self.raw) }
    }

    /// Get the duration of this audio in milliseconds.
    pub fn duration_ms(&self, sample_rate: i32) -> i64 {
        let frames = self.duration();
        unsafe { mixer::MIX_FramesToMS(sample_rate as c_int, frames) }
    }

    /// Convert milliseconds to sample frames for this audio.
    pub fn ms_to_frames(&self, ms: i64) -> i64 {
        unsafe { mixer::MIX_AudioMSToFrames(self.raw, ms) }
    }

    /// Convert sample frames to milliseconds for this audio.
    pub fn frames_to_ms(&self, frames: i64) -> i64 {
        unsafe { mixer::MIX_AudioFramesToMS(self.raw, frames) }
    }
}

// Safety: MIX_Audio is designed to be shared between threads
unsafe impl Send for Audio {}
unsafe impl Sync for Audio {}

/// A track that plays audio on a mixer.
///
/// Tracks are like channels on a mixer board - each can play one piece of
/// audio at a time, with independent volume, position, and effects.
pub struct Track<'mixer> {
    raw: *mut mixer::MIX_Track,
    _marker: PhantomData<&'mixer Mixer>,
}

impl<'mixer> Drop for Track<'mixer> {
    fn drop(&mut self) {
        unsafe {
            mixer::MIX_DestroyTrack(self.raw);
        }
    }
}

impl<'mixer> Track<'mixer> {
    /// Get the raw pointer to the track.
    pub fn raw(&self) -> *mut mixer::MIX_Track {
        self.raw
    }

    /// Set the audio to play on this track.
    pub fn set_audio(&self, audio: &Audio) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackAudio(self.raw, audio.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Start playing the track with default options.
    ///
    /// For more control over playback options (loops, fade-in, max duration),
    /// use `set_loops()` before calling this method, or use `play_with_options()`.
    pub fn play(&self) -> Result<(), Error> {
        // Use 0 for SDL_PropertiesID to use default options
        let success = unsafe { mixer::MIX_PlayTrack(self.raw, SDL_PropertiesID(0)) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Stop the track with optional fade-out.
    ///
    /// `fade_out_frames` is the number of sample frames to fade out (0 for immediate stop).
    pub fn stop(&self, fade_out_frames: i64) {
        unsafe {
            mixer::MIX_StopTrack(self.raw, fade_out_frames);
        }
    }

    /// Pause the track.
    pub fn pause(&self) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_PauseTrack(self.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resume a paused track.
    pub fn resume(&self) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_ResumeTrack(self.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Check if the track is currently playing.
    pub fn is_playing(&self) -> bool {
        unsafe { mixer::MIX_TrackPlaying(self.raw) }
    }

    /// Check if the track is paused.
    pub fn is_paused(&self) -> bool {
        unsafe { mixer::MIX_TrackPaused(self.raw) }
    }

    /// Set the gain (volume) for this track.
    ///
    /// Gain is a multiplier: 0.0 = silent, 1.0 = normal, 2.0 = double volume.
    pub fn set_gain(&self, gain: f32) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackGain(self.raw, gain as c_float) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the gain (volume) for this track.
    pub fn get_gain(&self) -> f32 {
        unsafe { mixer::MIX_GetTrackGain(self.raw) as f32 }
    }

    /// Set the frequency ratio for this track.
    ///
    /// This speeds up or slows down the audio, which also changes pitch.
    /// 1.0 = normal, 2.0 = double speed, 0.5 = half speed.
    pub fn set_frequency_ratio(&self, ratio: f32) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackFrequencyRatio(self.raw, ratio as c_float) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the frequency ratio for this track.
    pub fn get_frequency_ratio(&self) -> f32 {
        unsafe { mixer::MIX_GetTrackFrequencyRatio(self.raw) as f32 }
    }

    /// Set the loop count for this track.
    ///
    /// - loops > 0: play that many more times
    /// - loops = 0: stop after current playback
    /// - loops = -1: loop forever
    pub fn set_loops(&self, loops: i32) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackLoops(self.raw, loops as c_int) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the remaining loop count for this track.
    pub fn get_loops(&self) -> i32 {
        unsafe { mixer::MIX_GetTrackLoops(self.raw) as i32 }
    }

    /// Get the remaining sample frames to play.
    pub fn get_remaining(&self) -> i64 {
        unsafe { mixer::MIX_GetTrackRemaining(self.raw) }
    }

    /// Get the current playback position in sample frames.
    pub fn get_position(&self) -> i64 {
        unsafe { mixer::MIX_GetTrackPlaybackPosition(self.raw) }
    }

    /// Set the playback position in sample frames.
    pub fn set_position(&self, position: i64) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackPlaybackPosition(self.raw, position) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Add a tag to this track.
    ///
    /// Tags can be used to control multiple tracks at once with `Mixer::play_tag()`,
    /// `Mixer::stop_tag()`, etc.
    pub fn tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        let success = unsafe { mixer::MIX_TagTrack(self.raw, c_tag.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Remove a tag from this track.
    pub fn untag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = CString::new(tag).map_err(|_| Error("Invalid tag string".to_owned()))?;
        unsafe { mixer::MIX_UntagTrack(self.raw, c_tag.as_ptr()) };
        Ok(())
    }

    /// Set the 3D position of this track for spatial audio.
    ///
    /// This affects how the sound is panned and attenuated based on
    /// distance from the listener (assumed to be at origin, facing -Z).
    pub fn set_3d_position(&self, x: f32, y: f32, z: f32) -> Result<(), Error> {
        let pos = mixer::MIX_Point3D {
            x: x as c_float,
            y: y as c_float,
            z: z as c_float,
        };
        let success = unsafe { mixer::MIX_SetTrack3DPosition(self.raw, &pos) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Get the 3D position of this track.
    pub fn get_3d_position(&self) -> Option<(f32, f32, f32)> {
        let mut pos = mixer::MIX_Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let success = unsafe { mixer::MIX_GetTrack3DPosition(self.raw, &mut pos) };
        if success {
            Some((pos.x as f32, pos.y as f32, pos.z as f32))
        } else {
            None
        }
    }

    /// Set the stereo panning for this track.
    ///
    /// - `left`: gain for the left channel (0.0 to 1.0+)
    /// - `right`: gain for the right channel (0.0 to 1.0+)
    pub fn set_stereo(&self, left: f32, right: f32) -> Result<(), Error> {
        let gains = mixer::MIX_StereoGains {
            left: left as c_float,
            right: right as c_float,
        };
        let success = unsafe { mixer::MIX_SetTrackStereo(self.raw, &gains) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Convert milliseconds to sample frames for this track.
    pub fn ms_to_frames(&self, ms: i64) -> i64 {
        unsafe { mixer::MIX_TrackMSToFrames(self.raw, ms) }
    }

    /// Convert sample frames to milliseconds for this track.
    pub fn frames_to_ms(&self, frames: i64) -> i64 {
        unsafe { mixer::MIX_TrackFramesToMS(self.raw, frames) }
    }

    /// Assign this track to a group.
    pub fn set_group(&self, group: &Group) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackGroup(self.raw, group.raw) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Remove this track from any group.
    pub fn unset_group(&self) -> Result<(), Error> {
        let success = unsafe { mixer::MIX_SetTrackGroup(self.raw, ptr::null_mut()) };
        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

/// A group of tracks that can be processed together.
///
/// Groups allow you to apply effects or callbacks to a subset of tracks
/// before they are mixed with other tracks.
pub struct Group<'mixer> {
    raw: *mut mixer::MIX_Group,
    _marker: PhantomData<&'mixer Mixer>,
}

impl<'mixer> Drop for Group<'mixer> {
    fn drop(&mut self) {
        unsafe {
            mixer::MIX_DestroyGroup(self.raw);
        }
    }
}

impl<'mixer> Group<'mixer> {
    /// Get the raw pointer to the group.
    pub fn raw(&self) -> *mut mixer::MIX_Group {
        self.raw
    }
}
