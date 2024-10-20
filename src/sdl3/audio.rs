//! Audio Functions
//!
//! # Example
//! ```no_run
//! use sdl3::audio::{AudioCallback, AudioSpec};
//! use std::time::Duration;//!
//!
//! use sdl3::sys;
//!
//! struct SquareWave {
//!     phase_inc: f32,
//!     phase: f32,
//!     volume: f32
//! }
//!
//! impl AudioCallback for SquareWave {
//!     type Channel = f32;
//!
//!     fn callback(&mut self, out: &mut [f32]) {
//!         // Generate a square wave
//!         for x in out.iter_mut() {
//!             *x = if self.phase <= 0.5 {
//!                 self.volume
//!             } else {
//!                 -self.volume
//!             };
//!             self.phase = (self.phase + self.phase_inc) % 1.0;
//!         }
//!     }
//! }
//!
//! let sdl_context = sdl3::init().unwrap();
//! let audio_subsystem = sdl_context.audio().unwrap();
//!
//! let desired_spec = AudioSpec {
//!     freq: Some(44100),
//!     channels: Some(1),  // mono
//!     samples: None       // default sample size
//! };
//!
//! let device = audio_subsystem.open_playback(&desired_spec, |spec| {
//!     // initialize the audio callback
//!     SquareWave {
//!         phase_inc: 440.0 / spec.freq as f32,
//!         phase: 0.0,
//!         volume: 0.25
//!     }
//! }).unwrap();
//!
//! // Start playback
//! device.resume();
//!
//! // Play for 2 seconds
//! std::thread::sleep(Duration::from_millis(2000));
//! ```

use crate::get_error;
use crate::AudioSubsystem;
use iostream::IOStream;
use libc::{c_int, c_void};
use std::convert::TryFrom;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::ptr;

use crate::sys;

impl AudioSubsystem {
    /// Opens a new audio device given the desired parameters.
    #[inline]
    pub fn open_playback<'a, D>(&self, device: D, spec: &AudioSpec) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open_playback(self, device, spec)
    }

    /// Opens a new audio device for capture (given the desired parameters).
    pub fn open_capture<'a, D>(&self, device: D, spec: &AudioSpec) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open_capture(self, device, spec)
    }

    // /// Opens a new audio device which uses queueing rather than older callback method.
    // #[inline]
    // pub fn open_queue<'a, Channel, D>(
    //     &self,
    //     device: D,
    //     spec: &AudioSpec,
    // ) -> Result<AudioQueue<Channel>, String>
    // where
    //     Channel: AudioFormatNum,
    //     D: Into<Option<&'a str>>,
    // {
    //     AudioQueue::open_queue(self, device, spec)
    // }

    #[doc(alias = "SDL_GetCurrentAudioDriver")]
    pub fn current_audio_driver(&self) -> &'static str {
        unsafe {
            let buf = sys::SDL_GetCurrentAudioDriver();
            assert!(!buf.is_null());

            CStr::from_ptr(buf as *const _).to_str().unwrap()
        }
    }

    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn audio_playback_device_name(&self, index: u32) -> Result<String, String> {
        unsafe {
            let dev_name = sys::SDL_GetAudioDeviceName(index);
            if dev_name.is_null() {
                Err(get_error())
            } else {
                let cstr = CStr::from_ptr(dev_name as *const _);
                Ok(cstr.to_str().unwrap().to_owned())
            }
        }
    }

    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn audio_capture_device_name(&self, index: u32) -> Result<String, String> {
        unsafe {
            let dev_name = sys::SDL_GetAudioDeviceName(index);
            if dev_name.is_null() {
                Err(get_error())
            } else {
                let cstr = CStr::from_ptr(dev_name as *const _);
                Ok(cstr.to_str().unwrap().to_owned())
            }
        }
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AudioFormat {
    /// Unsigned 8-bit samples
    U8 = sys::SDL_AUDIO_U8 as i32,
    /// Signed 8-bit samples
    S8 = sys::SDL_AUDIO_S8 as i32,
    /// Signed 16-bit samples, little-endian
    S16LE = sys::SDL_AUDIO_S16LE as i32,
    /// Signed 16-bit samples, big-endian
    S16BE = sys::SDL_AUDIO_S16BE as i32,
    /// Signed 32-bit samples, little-endian
    S32LE = sys::SDL_AUDIO_S32LE as i32,
    /// Signed 32-bit samples, big-endian
    S32BE = sys::SDL_AUDIO_S32BE as i32,
    /// 32-bit floating point samples, little-endian
    F32LE = sys::SDL_AUDIO_F32LE as i32,
    /// 32-bit floating point samples, big-endian
    F32BE = sys::SDL_AUDIO_F32BE as i32,
}

impl AudioFormat {
    fn from_ll(raw: sys::SDL_AudioFormat) -> Option<AudioFormat> {
        use self::AudioFormat::*;
        match raw as u32 {
            sys::SDL_AUDIO_U8 => Some(U8),
            sys::SDL_AUDIO_S8 => Some(S8),
            sys::SDL_AUDIO_S16LE => Some(S16LE),
            sys::SDL_AUDIO_S16BE => Some(S16BE),
            sys::SDL_AUDIO_S32LE => Some(S32LE),
            sys::SDL_AUDIO_S32BE => Some(S32BE),
            sys::SDL_AUDIO_F32LE => Some(F32LE),
            sys::SDL_AUDIO_F32BE => Some(F32BE),
            _ => None,
        }
    }

    #[doc(alias = "SDL_AudioFormat")]
    fn to_ll(self) -> sys::SDL_AudioFormat {
        self as sys::SDL_AudioFormat
    }
}

#[cfg(target_endian = "little")]
impl AudioFormat {
    /// Signed 16-bit samples, native endian
    #[inline]
    pub const fn s16_sys() -> AudioFormat {
        AudioFormat::S16LE
    }
    /// Signed 32-bit samples, native endian
    #[inline]
    pub const fn s32_sys() -> AudioFormat {
        AudioFormat::S32LE
    }
    /// 32-bit floating point samples, native endian
    #[inline]
    pub const fn f32_sys() -> AudioFormat {
        AudioFormat::F32LE
    }
}

#[cfg(target_endian = "big")]
impl AudioFormat {
    /// Signed 16-bit samples, native endian
    #[inline]
    pub const fn s16_sys() -> AudioFormat {
        AudioFormat::S16MSB
    }
    /// Signed 32-bit samples, native endian
    #[inline]
    pub const fn s32_sys() -> AudioFormat {
        AudioFormat::S32MSB
    }
    /// 32-bit floating point samples, native endian
    #[inline]
    pub const fn f32_sys() -> AudioFormat {
        AudioFormat::F32MSB
    }
}

#[doc(alias = "SDL_GetAudioDriver")]
#[derive(Copy, Clone)]
pub struct DriverIterator {
    length: i32,
    index: i32,
}

impl Iterator for DriverIterator {
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        if self.index >= self.length {
            None
        } else {
            unsafe {
                let buf = sys::SDL_GetAudioDriver(self.index);
                assert!(!buf.is_null());
                self.index += 1;

                Some(CStr::from_ptr(buf as *const _).to_str().unwrap())
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.length as usize;
        (l, Some(l))
    }
}

impl ExactSizeIterator for DriverIterator {}

/// Gets an iterator of all audio drivers compiled into the SDL2 library.
#[doc(alias = "SDL_GetAudioDriver")]
#[inline]
pub fn drivers() -> DriverIterator {
    // This function is thread-safe and doesn't require the audio subsystem to be initialized.
    // The list of drivers are read-only and statically compiled into SDL2, varying by platform.

    // SDL_GetNumAudioDrivers can never return a negative value.
    DriverIterator {
        length: unsafe { sys::SDL_GetNumAudioDrivers() },
        index: 0,
    }
}

pub struct AudioSpecWAV {
    pub freq: i32,
    pub format: AudioFormat,
    pub channels: u8,
    audio_buf: *mut u8,
    audio_len: u32,
}

impl AudioSpecWAV {
    /// Loads a WAVE from the file path.
    pub fn load_wav<P: AsRef<Path>>(path: P) -> Result<AudioSpecWAV, String> {
        let mut file = IOStream::from_file(path, "rb")?;
        AudioSpecWAV::load_wav_rw(&mut file)
    }

    /// Loads a WAVE from the data source.
    #[doc(alias = "SDL_LoadWAV_RW")]
    pub fn load_wav_rw(src: &mut IOStream) -> Result<AudioSpecWAV, String> {
        use std::mem::MaybeUninit;
        use std::ptr::null_mut;

        let mut desired = MaybeUninit::uninit();
        let mut audio_buf: *mut u8 = null_mut();
        let mut audio_len: u32 = 0;
        unsafe {
            let ret = sys::SDL_LoadWAV_RW(
                src.raw(),
                sys::SDL_bool::SDL_FALSE,
                desired.as_mut_ptr(),
                &mut audio_buf,
                &mut audio_len,
            );
            if ret == -1 {
                Err(get_error())
            } else {
                let desired = desired.assume_init();
                Ok(AudioSpecWAV {
                    freq: desired.freq,
                    format: AudioFormat::from_ll(desired.format).unwrap(),
                    channels: desired.channels.try_into().unwrap(),
                    audio_buf,
                    audio_len,
                })
            }
        }
    }

    pub fn buffer(&self) -> &[u8] {
        use std::slice::from_raw_parts;
        unsafe {
            let ptr = self.audio_buf as *const u8;
            let len = self.audio_len as usize;
            from_raw_parts(ptr, len)
        }
    }
}

impl Drop for AudioSpecWAV {
    #[doc(alias = "SDL_free")]
    fn drop(&mut self) {
        unsafe {
            sys::SDL_free(self.audio_buf as *mut _);
        }
    }
}

pub trait AudioCallback: Send
where
    Self::Channel: AudioFormatNum + 'static,
{
    type Channel;

    fn callback(&mut self, _: &mut [Self::Channel]);
}

/// A phantom type for retrieving the `SDL_AudioFormat` of a given generic type.
/// All format types are returned as native-endian.
pub trait AudioFormatNum {
    fn audio_format() -> AudioFormat;

    /// The appropriately typed silence value for the audio format used.
    ///
    /// # Examples
    ///
    /// ```
    /// // The AudioFormatNum trait has to be imported for the Channel::SILENCE part to work.
    /// use sdl3::audio::{AudioCallback, AudioFormatNum};
    ///
    /// struct Silence;
    ///
    /// impl AudioCallback for Silence {
    ///     type Channel = u16;
    ///
    ///     fn callback(&mut self, out: &mut [u16]) {
    ///         for dst in out.iter_mut() {
    ///             *dst = Self::Channel::SILENCE;
    ///         }
    ///     }
    /// }
    /// ```
    const SILENCE: Self;
}

/// `AUDIO_S8`
impl AudioFormatNum for i8 {
    fn audio_format() -> AudioFormat {
        AudioFormat::S8
    }
    const SILENCE: i8 = 0;
}
/// `AUDIO_U8`
impl AudioFormatNum for u8 {
    fn audio_format() -> AudioFormat {
        AudioFormat::U8
    }
    const SILENCE: u8 = 0x80;
}
/// `AUDIO_S16`
impl AudioFormatNum for i16 {
    fn audio_format() -> AudioFormat {
        AudioFormat::s16_sys()
    }
    const SILENCE: i16 = 0;
}
/// `AUDIO_S32`
impl AudioFormatNum for i32 {
    fn audio_format() -> AudioFormat {
        AudioFormat::s32_sys()
    }
    const SILENCE: i32 = 0;
}
/// `AUDIO_F32`
impl AudioFormatNum for f32 {
    fn audio_format() -> AudioFormat {
        AudioFormat::f32_sys()
    }
    const SILENCE: f32 = 0.0;
}

extern "C" fn audio_callback_marshall<CB: AudioCallback>(
    userdata: *mut c_void,
    stream: *mut u8,
    len: c_int,
) {
    use std::mem::size_of;
    use std::slice::from_raw_parts_mut;
    unsafe {
        let cb_userdata: &mut Option<CB> = &mut *(userdata as *mut _);
        let buf: &mut [CB::Channel] = from_raw_parts_mut(
            stream as *mut CB::Channel,
            len as usize / size_of::<CB::Channel>(),
        );

        if let Some(cb) = cb_userdata {
            cb.callback(buf);
        }
    }
}

#[derive(Clone)]
pub struct AudioSpec {
    /// DSP frequency (samples per second). Set to None for the device's fallback frequency.
    pub freq: Option<i32>,
    /// Number of separate audio channels. Set to None for the device's fallback number of channels.
    pub channels: Option<i32>,
    /// Audio format. Set to None for the device's fallback audio format.
    pub format: Option<AudioFormat>,
}

impl AudioSpec {
    fn convert_to_ll<R, C, F>(rate: R, channels: C, format: F) -> sys::audio::SDL_AudioSpec
    where
        R: Into<Option<i32>>,
        C: Into<Option<u8>>,
        F: Into<Option<AudioFormat>>,
    {
        let channels = channels.into();
        let freq = rate.into();
        let format = format.into();

        if let Some(channels) = channels {
            assert!(channels > 0);
        }
        if let Some(freq) = freq {
            assert!(freq > 0);
        }

        // A value of 0 means "fallback" or "default".

        sys::audio::SDL_AudioSpec {
            format: format.unwrap_or(AudioFormat::U8).to_ll(),
            channels: channels.unwrap_or(0),
            freq: freq.unwrap_or(0),
        }
    }

    // fn convert_queue_to_ll<Channel, F, C, S>(freq: F, channels: C, samples: S) -> sys::SDL_AudioSpec
    // where
    //     Channel: AudioFormatNum,
    //     F: Into<Option<i32>>,
    //     C: Into<Option<u8>>,
    //     S: Into<Option<u16>>,
    // {
    //     let freq = freq.into();
    //     let channels = channels.into();
    //     let samples = samples.into();
    //
    //     if let Some(freq) = freq {
    //         assert!(freq > 0);
    //     }
    //     if let Some(channels) = channels {
    //         assert!(channels > 0);
    //     }
    //     if let Some(samples) = samples {
    //         assert!(samples > 0);
    //     }
    //
    //     // A value of 0 means "fallback" or "default".
    //
    //     sys::SDL_AudioSpec {
    //         freq: freq.unwrap_or(0),
    //         format: <Channel as AudioFormatNum>::audio_format().to_ll(),
    //         channels: channels.unwrap_or(0),
    //         silence: 0,
    //         samples: samples.unwrap_or(0),
    //         padding: 0,
    //         size: 0,
    //         callback: None,
    //         userdata: ptr::null_mut(),
    //     }
    // }
}

#[allow(missing_copy_implementations)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AudioSpec {
    pub freq: i32,
    pub format: AudioFormat,
    pub channels: u8,
}

impl AudioSpec {
    fn convert_from_ll(spec: sys::SDL_AudioSpec) -> AudioSpec {
        AudioSpec {
            freq: spec.freq,
            format: AudioFormat::from_ll(spec.format).unwrap(),
            channels: spec.channels.try_into().unwrap(),
        }
    }
}

enum AudioDeviceID {
    PlaybackDevice(sys::SDL_AudioDeviceID),
}

impl AudioDeviceID {
    fn id(&self) -> sys::SDL_AudioDeviceID {
        match *self {
            AudioDeviceID::PlaybackDevice(id) => id,
        }
    }
}

impl Drop for AudioDeviceID {
    #[doc(alias = "SDL_CloseAudioDevice")]
    fn drop(&mut self) {
        //! Shut down audio processing and close the audio device.
        println!("Closing audio device: {}", self.id());
        unsafe { sys::SDL_CloseAudioDevice(self.id()) }
        println!("Closed audio device: {}", self.id());
    }
}

// /// Wraps `SDL_AudioDeviceID` and owns the callback data used by the audio device.
// pub struct AudioQueue<Channel: AudioFormatNum> {
//     subsystem: AudioSubsystem,
//     device_id: AudioDeviceID,
//     phantom: PhantomData<Channel>,
//     spec: AudioSpec,
// }
//
// impl<'a, Channel: AudioFormatNum> AudioQueue<Channel> {
//     /// Opens a new audio device given the desired parameters and callback.
//     #[doc(alias = "SDL_OpenAudioDevice")]
//     pub fn open_queue<D: Into<Option<&'a str>>>(
//         a: &AudioSubsystem,
//         device: D,
//         spec: &AudioSpec,
//     ) -> Result<AudioQueue<Channel>, String> {
//         use std::mem::MaybeUninit;
//
//         let desired = AudioSpec::convert_queue_to_ll::<
//             Channel,
//             Option<i32>,
//             Option<u8>,
//             Option<u16>,
//         >(spec.freq, spec.channels, spec.samples);
//
//         let mut obtained = MaybeUninit::uninit();
//         unsafe {
//             let device = match device.into() {
//                 Some(device) => Some(CString::new(device).unwrap()),
//                 None => None,
//             };
//             // Warning: map_or consumes its argument; `device.map_or()` would therefore consume the
//             // CString and drop it, making device_ptr a dangling pointer! To avoid that we downgrade
//             // device to an Option<&_> first.
//             let device_ptr = device.as_ref().map_or(ptr::null(), |s| s.as_ptr());
//
//             let iscapture_flag = 0;
//             let device_id = sys::SDL_OpenAudioDevice(
//                 device_ptr as *const c_char,
//                 iscapture_flag,
//                 &desired,
//                 obtained.as_mut_ptr(),
//                 0,
//             );
//             match device_id {
//                 0 => Err(get_error()),
//                 id => {
//                     let obtained = obtained.assume_init();
//                     let device_id = AudioDeviceID::PlaybackDevice(id);
//                     let spec = AudioSpec::convert_from_ll(obtained);
//
//                     Ok(AudioQueue {
//                         subsystem: a.clone(),
//                         device_id,
//                         phantom: PhantomData::default(),
//                         spec,
//                     })
//                 }
//             }
//         }
//     }
//
//     #[inline]
//     #[doc(alias = "SDL_GetAudioDeviceStatus")]
//     pub fn subsystem(&self) -> &AudioSubsystem {
//         &self.subsystem
//     }
//
//     #[inline]
//     pub fn spec(&self) -> &AudioSpec {
//         &self.spec
//     }
//
//     /// Pauses playback of the audio device.
//     #[doc(alias = "SDL_PauseAudioDevice")]
//     pub fn pause(&self) -> i32 {
//         unsafe { sys::SDL_PauseAudioDevice(self.device_id.id()) }
//     }
//
//     /// (Re-)starts playback of the audio device.
//     #[doc(alias = "SDL_ResumeAudioDevice")]
//     pub fn resume(&self) -> i32 {
//         unsafe { sys::SDL_ResumeAudioDevice(self.device_id.id()) }
//     }
// }

/// Wraps `SDL_AudioDeviceID` and owns the streams attached to the audio device.
pub struct AudioDevice {
    // DEFAULT_CAPTURE: sys::SDL
    subsystem: AudioSubsystem,
    device_id: AudioDeviceID,
    spec: AudioSpec,

    /// Name of the device
    pub name: String,
    // pub streams: Vec<AudioStream<FnOnce(&mut [u8])>>,
}

impl AudioDevice {
    /// Opens a new audio device for playback or capture (given the desired parameters).
    #[doc(alias = "SDL_OpenAudioDevice")]
    fn open<'a, D>(
        a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
        capture: bool,
    ) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        use std::mem::MaybeUninit;

        let desired = AudioSpec::convert_to_ll(spec.freq, spec.channels, spec.format);

        let mut obtained = MaybeUninit::uninit();
        unsafe {
            // let device = match device.into() {
            //     Some(device) => Some(CString::new(device).unwrap()),
            //     None => None,
            // };
            // Warning: map_or consumes its argument; `device.map_or()` would therefore consume the
            // CString and drop it, making device_ptr a dangling pointer! To avoid that we downgrade
            // device to an Option<&_> first.
            // let device_ptr = device.as_ref().map_or(ptr::null(), |s| s.as_ptr());
            let device_ptr = device.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let device_id = sys::SDL_OpenAudioDevice(device_ptr, &desired);
            match device_id {
                0 => Err(get_error()),
                id => {
                    let obtained = obtained.assume_init();
                    let device_id = AudioDeviceID::PlaybackDevice(id);
                    let spec = AudioSpec::convert_from_ll(obtained);
                    let name = if capture {
                        a.audio_capture_device_name(id as u32)?
                    } else {
                        a.audio_playback_device_name(id as u32)?
                    };

                    Ok(AudioDevice {
                        subsystem: a.clone(),
                        device_id,
                        spec,
                        name,
                    })
                }
            }
        }
    }

    /// Opens a new audio device for playback (given the desired parameters).
    pub fn open_playback<'a, D>(
        a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open(a, device, spec, false)
    }

    /// Opens a new audio device for capture (given the desired parameters).
    pub fn open_capture<'a, D>(
        a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open(a, device, spec, true)
    }

    #[inline]
    #[doc(alias = "SDL_GetAudioDeviceStatus")]
    pub fn subsystem(&self) -> &AudioSubsystem {
        &self.subsystem
    }

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    /// Pauses playback of the audio device.
    #[doc(alias = "SDL_PauseAudioDevice")]
    pub fn pause(&self) -> i32 {
        unsafe { sys::SDL_PauseAudioDevice(self.device_id.id()) }
    }

    /// Starts playback of the audio device.
    #[doc(alias = "SDL_ResumeAudioDevice")]
    pub fn resume(&self) -> i32 {
        unsafe { sys::audio::SDL_ResumeAudioDevice(self.device_id.id()) }
    }

    /// Closes the audio device and saves the callback data from being dropped.
    ///
    /// Note that simply dropping `AudioDevice` will close the audio device,
    /// but the callback data will be dropped.
    // pub fn close_and_get_callback(self) -> CB {
    //     drop(self.device_id);
    //     self.userdata.expect("Missing callback")
    // }

    /// Creates a bound audio stream on this device.
    /// Convenience function for straightforward audio init for the common case.
    /// This function will open an audio device, create a stream and bind it. Unlike other methods of setup, the audio device will be closed when this stream is destroyed, so the app can treat the returned SDL_AudioStream as the only object needed to manage audio playback.
    /// Also unlike other functions, the audio device begins paused. This is to map more closely to SDL2-style behavior, and since there is no extra step here to bind a stream to begin audio flowing. The audio device should be resumed with
    /// This function works with both playback and capture devices.
    pub fn open_stream<CB: AudioCallback>(
        &self,
        spec: &AudioSpec,
        callback: CB,
        userdata: Option<CB>,
    ) -> Result<AudioStream<CB>, String> {
        //  Returns an audio stream on success, ready to use. NULL on error; call SDL_GetError() for more information. When done with this stream, call SDL_DestroyAudioStream to free resources and close the device.
        let stream = unsafe {
            sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id.id(),
                AudioSpec::convert_to_ll(spec.rate, spec.channels, spec.format),
                callback,
                userdata,
            )
        };
        if stream.is_null() {
            Err(get_error())
        } else {
            Ok(AudioStream {
                stream,
                callback: userdata,
            })
        }
    }
}

/// Represents a stream of audio attached to a device.
/// See [SDL_AudioStream](https://wiki.libsdl.org/SDL_AudioStream)
pub struct AudioStream<CB: AudioCallback> {
    stream: *mut sys::audio::SDL_AudioStream,

    callback: Option<CB>,
    userdata: Option<CB>,
}

impl<CB: AudioCallback> AudioStream<CB> {
    pub fn get_stream(&mut self) -> *mut sys::audio::SDL_AudioStream {
        self.stream
    }

    fn get_device_id(&mut self) -> &mut AudioDeviceID {
        let device = unsafe { sys::audio::SDL_GetAudioStreamDevice(self.stream) };
        unsafe { &mut *(device as *mut AudioDeviceID) }
    }

    /// Locks the audio stream using `SDL_LockAudioStream`.
    ///
    /// If the app assigns a callback to a specific stream, it can use the stream's lock through
    /// SDL_LockAudioStream() if necessary.
    #[doc(alias = "SDL_LockAudioStream")]
    pub fn lock(&mut self) -> AudioStreamLockGuard<CB> {
        unsafe { sys::SDL_LockAudioStream(self.get_stream()) };
        AudioStreamLockGuard {
            stream: self,
            _nosend: PhantomData,
        }
    }
}

/// Similar to `std::sync::MutexGuard`, but for use with `AudioStream::lock()`.
pub struct AudioStreamLockGuard<'a, CB>
where
    CB: AudioCallback,
    CB: 'a,
{
    stream: &'a mut AudioStream<CB>,
    _nosend: PhantomData<*mut ()>,
}

impl<'a, CB: AudioCallback> Deref for AudioStreamLockGuard<'a, CB> {
    type Target = CB;
    #[doc(alias = "SDL_UnlockAudioStream")]
    fn deref(&self) -> &CB {
        (*self.device.userdata).as_ref().expect("Missing callback")
    }
}

impl<'a, CB: AudioCallback> DerefMut for AudioStreamLockGuard<'a, CB> {
    fn deref_mut(&mut self) -> &mut CB {
        (*self.device.userdata).as_mut().expect("Missing callback")
    }
}

impl<'a, CB: AudioCallback> Drop for AudioStreamLockGuard<'a, CB> {
    fn drop(&mut self) {
        unsafe { sys::SDL_UnlockAudioStream(self._audio_stream) }
    }
}

#[cfg(test)]
mod test {}
