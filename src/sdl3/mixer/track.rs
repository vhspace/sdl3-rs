use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;

use crate::{get_error, Error};
use sdl3_sys::properties::SDL_PropertiesID;
use sdl3_sys::stdinc::Sint64;

use super::audio::Audio;
use super::device::Mixer;
use super::{bool_result, sys, to_cstring};

/// 3D coordinates for positional audio.
///
/// Uses a right-handed coordinate system (like OpenGL/OpenAL):
/// - X: negative left, positive right
/// - Y: negative down, positive up
/// - Z: negative forward, positive back
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<sys::MIX_Point3D> for Point3D {
    fn from(p: sys::MIX_Point3D) -> Self {
        Point3D {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

impl From<Point3D> for sys::MIX_Point3D {
    fn from(p: Point3D) -> Self {
        sys::MIX_Point3D {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

/// Per-channel gain for stereo panning.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StereoGains {
    pub left: f32,
    pub right: f32,
}

/// A track on a mixer that plays audio.
///
/// Tracks are the primary way to play sounds. Each track manages its own audio
/// source, gain, loops, position, and effects. Multiple tracks can play
/// simultaneously on the same mixer.
///
/// A track must not outlive its parent mixer (enforced by the lifetime
/// parameter).
pub struct Track<'mixer> {
    raw: *mut sys::MIX_Track,
    _marker: PhantomData<&'mixer Mixer>,
}

impl<'mixer> Track<'mixer> {
    pub(crate) fn from_raw(raw: *mut sys::MIX_Track) -> Self {
        Track {
            raw,
            _marker: PhantomData,
        }
    }

    /// Get a pointer to the underlying `MIX_Track`.
    #[inline]
    pub fn raw(&self) -> *mut sys::MIX_Track {
        self.raw
    }

    // -- Input assignment --

    /// Set the audio data for this track.
    ///
    /// The track holds an internal reference to the audio, so it's safe to drop
    /// the `Audio` object after this call -- the track will keep it alive.
    #[doc(alias = "MIX_SetTrackAudio")]
    pub fn set_audio(&self, audio: &Audio) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetTrackAudio(self.raw, audio.raw()) })
    }

    /// Remove the audio input from this track.
    #[doc(alias = "MIX_SetTrackAudio")]
    pub fn clear_audio(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetTrackAudio(self.raw, ptr::null_mut()) })
    }

    // -- Playback control --

    /// Start (or restart) playing this track.
    ///
    /// This uses the default playback properties.
    #[doc(alias = "MIX_PlayTrack")]
    pub fn play(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_PlayTrack(self.raw, SDL_PropertiesID(0)) })
    }

    /// Stop this track, with optional fade-out in sample frames.
    ///
    /// Use `ms_to_frames` to convert milliseconds to frames.
    /// Pass 0 for immediate stop.
    #[doc(alias = "MIX_StopTrack")]
    pub fn stop(&self, fade_out_frames: i64) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_StopTrack(self.raw, fade_out_frames as Sint64) })
    }

    /// Pause this track.
    #[doc(alias = "MIX_PauseTrack")]
    pub fn pause(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_PauseTrack(self.raw) })
    }

    /// Resume this track if paused.
    #[doc(alias = "MIX_ResumeTrack")]
    pub fn resume(&self) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_ResumeTrack(self.raw) })
    }

    /// Check if this track is currently playing.
    #[doc(alias = "MIX_TrackPlaying")]
    pub fn is_playing(&self) -> bool {
        unsafe { sys::MIX_TrackPlaying(self.raw) }
    }

    /// Check if this track is currently paused.
    #[doc(alias = "MIX_TrackPaused")]
    pub fn is_paused(&self) -> bool {
        unsafe { sys::MIX_TrackPaused(self.raw) }
    }

    // -- Gain --

    /// Set the gain (volume) for this track.
    ///
    /// A gain of 0.0 is silence, 1.0 is unchanged, >1.0 amplifies.
    #[doc(alias = "MIX_SetTrackGain")]
    pub fn set_gain(&self, gain: f32) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetTrackGain(self.raw, gain) })
    }

    /// Get the current gain (volume) for this track.
    #[doc(alias = "MIX_GetTrackGain")]
    pub fn gain(&self) -> f32 {
        unsafe { sys::MIX_GetTrackGain(self.raw) }
    }

    // -- Looping --

    /// Set the number of additional loops for this track.
    ///
    /// - 0: play once (no loops)
    /// - 1: play twice (one loop)
    /// - -1: loop forever
    #[doc(alias = "MIX_SetTrackLoops")]
    pub fn set_loops(&self, loops: i32) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetTrackLoops(self.raw, loops) })
    }

    /// Get the current loop count for this track.
    #[doc(alias = "MIX_GetTrackLoops")]
    pub fn loops(&self) -> i32 {
        unsafe { sys::MIX_GetTrackLoops(self.raw) }
    }

    // -- Position --

    /// Set the playback position in sample frames.
    #[doc(alias = "MIX_SetTrackPlaybackPosition")]
    pub fn set_playback_position(&self, frames: i64) -> Result<(), Error> {
        bool_result(unsafe { sys::MIX_SetTrackPlaybackPosition(self.raw, frames as Sint64) })
    }

    /// Get the current playback position in sample frames.
    #[doc(alias = "MIX_GetTrackPlaybackPosition")]
    pub fn playback_position(&self) -> i64 {
        unsafe { sys::MIX_GetTrackPlaybackPosition(self.raw) as i64 }
    }

    /// Get the number of sample frames remaining for this track.
    #[doc(alias = "MIX_GetTrackRemaining")]
    pub fn remaining(&self) -> i64 {
        unsafe { sys::MIX_GetTrackRemaining(self.raw) as i64 }
    }

    /// Get the number of sample frames in the current fade effect.
    #[doc(alias = "MIX_GetTrackFadeFrames")]
    pub fn fade_frames(&self) -> i64 {
        unsafe { sys::MIX_GetTrackFadeFrames(self.raw) as i64 }
    }

    // -- Time conversion --

    /// Convert milliseconds to sample frames for this track.
    #[doc(alias = "MIX_TrackMSToFrames")]
    pub fn ms_to_frames(&self, ms: i64) -> i64 {
        unsafe { sys::MIX_TrackMSToFrames(self.raw, ms as Sint64) as i64 }
    }

    /// Convert sample frames to milliseconds for this track.
    #[doc(alias = "MIX_TrackFramesToMS")]
    pub fn frames_to_ms(&self, frames: i64) -> i64 {
        unsafe { sys::MIX_TrackFramesToMS(self.raw, frames as Sint64) as i64 }
    }

    // -- Spatial audio --

    /// Set this track's position in 3D space.
    ///
    /// This enables distance attenuation and spatialization. On stereo setups,
    /// sounds will appear to move left/right. On surround-sound setups, sounds
    /// can move around the listener.
    #[doc(alias = "MIX_SetTrack3DPosition")]
    pub fn set_3d_position(&self, pos: Point3D) -> Result<(), Error> {
        let c_pos: sys::MIX_Point3D = pos.into();
        bool_result(unsafe { sys::MIX_SetTrack3DPosition(self.raw, &c_pos) })
    }

    /// Get this track's current 3D position.
    #[doc(alias = "MIX_GetTrack3DPosition")]
    pub fn get_3d_position(&self) -> Result<Point3D, Error> {
        let mut c_pos = sys::MIX_Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let ok = unsafe { sys::MIX_GetTrack3DPosition(self.raw, &mut c_pos) };
        if ok {
            Ok(c_pos.into())
        } else {
            Err(get_error())
        }
    }

    /// Force stereo output with left/right panning.
    ///
    /// Pass `None` to disable spatialization entirely (both forced-stereo
    /// and 3D positioning).
    #[doc(alias = "MIX_SetTrackStereo")]
    pub fn set_stereo(&self, gains: Option<StereoGains>) -> Result<(), Error> {
        let ok = match gains {
            Some(g) => {
                let c_gains = sys::MIX_StereoGains {
                    left: g.left,
                    right: g.right,
                };
                unsafe { sys::MIX_SetTrackStereo(self.raw, &c_gains) }
            }
            None => unsafe { sys::MIX_SetTrackStereo(self.raw, ptr::null()) },
        };
        bool_result(ok)
    }

    // -- Tagging --

    /// Add a tag to this track for batch operations.
    #[doc(alias = "MIX_TagTrack")]
    pub fn tag(&self, tag: &str) -> Result<(), Error> {
        let c_tag = to_cstring(tag)?;
        bool_result(unsafe { sys::MIX_TagTrack(self.raw, c_tag.as_ptr()) })
    }

    /// Remove a tag from this track.
    ///
    /// Pass `None` to remove all tags.
    #[doc(alias = "MIX_UntagTrack")]
    pub fn untag(&self, tag: Option<&str>) -> Result<(), Error> {
        match tag {
            Some(t) => {
                let c_tag = to_cstring(t)?;
                unsafe { sys::MIX_UntagTrack(self.raw, c_tag.as_ptr()) };
            }
            None => {
                unsafe { sys::MIX_UntagTrack(self.raw, ptr::null()) };
            }
        }
        Ok(())
    }

    /// Get all tags currently associated with this track.
    #[doc(alias = "MIX_GetTrackTags")]
    pub fn tags(&self) -> Vec<String> {
        let mut count: std::ffi::c_int = 0;
        let tags_ptr = unsafe { sys::MIX_GetTrackTags(self.raw, &mut count) };
        if tags_ptr.is_null() {
            return Vec::new();
        }
        if count <= 0 {
            unsafe { sdl3_sys::stdinc::SDL_free(tags_ptr as *mut _) };
            return Vec::new();
        }
        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count as isize {
            unsafe {
                let tag = *tags_ptr.offset(i);
                if !tag.is_null() {
                    if let Ok(s) = CStr::from_ptr(tag).to_str() {
                        result.push(s.to_owned());
                    }
                }
            }
        }
        unsafe { sdl3_sys::stdinc::SDL_free(tags_ptr as *mut _) };
        result
    }
}

impl Drop for Track<'_> {
    fn drop(&mut self) {
        unsafe { sys::MIX_DestroyTrack(self.raw) };
    }
}
