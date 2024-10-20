//! Audio Functions
//!
//! # Example
//! ```no_run
//! use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec};
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
//! impl AudioCallback<f32> for SquareWave {
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
//!     format: Some(AudioFormat::S16BE) // signed 16 bit samples
//! };
//!
//! let device = audio_subsystem.open_playback_stream(&desired_spec, |spec| {
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
use crate::sys;
use crate::AudioSubsystem;
use iostream::IOStream;
use libc::c_void;
use std::convert::{TryFrom, TryInto};
use std::ffi::{c_int, CStr};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::ptr::null_mut;
use sys::audio::{SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, SDL_AUDIO_DEVICE_DEFAULT_RECORDING};
use sys::stdinc::SDL_free;

impl AudioSubsystem {
    /// Open a default playback device with the specified audio spec.
    pub fn open_playback_device(&self, spec: &AudioSpec) -> Result<AudioDevice, String> {
        self.open_device(sys::audio::SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, spec)
    }

    /// Open a default recording device with the specified audio spec.
    pub fn open_recording_device(&self, spec: &AudioSpec) -> Result<AudioDevice, String> {
        self.open_device(sys::audio::SDL_AUDIO_DEVICE_DEFAULT_RECORDING, spec)
    }

    /// General method to open a device by ID.
    fn open_device(
        &self,
        device_id: sys::audio::SDL_AudioDeviceID,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, String> {
        let sdl_spec: sys::audio::SDL_AudioSpec = spec.clone().into();
        let device = unsafe { sys::audio::SDL_OpenAudioDevice(device_id, &sdl_spec) };
        if device == 0 {
            Err(get_error())
        } else {
            Ok(AudioDevice {
                device_id: device,
                subsystem: self,
            })
        }
    }

    pub fn open_playback_stream<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStream<CB>, String>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let device = AudioDevice::open_playback(self, None, spec)?;
        device.open_stream_with_callback(spec, callback)
    }

    pub fn open_recording_stream<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStream<CB>, String>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let device = AudioDevice::open_recording(self, None, spec)?;
        device.open_stream_with_callback(spec, callback)
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
            let buf = sys::audio::SDL_GetCurrentAudioDriver();
            assert!(!buf.is_null());

            CStr::from_ptr(buf as *const _).to_str().unwrap()
        }
    }

    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn audio_playback_device_name(&self, index: u32) -> Result<String, String> {
        unsafe {
            let dev_name = sys::audio::SDL_GetAudioDeviceName(index);
            if dev_name.is_null() {
                Err(get_error())
            } else {
                let cstr = CStr::from_ptr(dev_name as *const _);
                Ok(cstr.to_str().unwrap().to_owned())
            }
        }
    }

    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn audio_recording_device_name(&self, index: u32) -> Result<String, String> {
        unsafe {
            let dev_name = sys::audio::SDL_GetAudioDeviceName(index);
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
    U8 = sys::audio::SDL_AUDIO_U8.0,
    /// Signed 8-bit samples
    S8 = sys::audio::SDL_AUDIO_S8.0,
    /// Signed 16-bit samples, little-endian
    S16LE = sys::audio::SDL_AUDIO_S16LE.0,
    /// Signed 16-bit samples, big-endian
    S16BE = sys::audio::SDL_AUDIO_S16BE.0,
    /// Signed 32-bit samples, little-endian
    S32LE = sys::audio::SDL_AUDIO_S32LE.0,
    /// Signed 32-bit samples, big-endian
    S32BE = sys::audio::SDL_AUDIO_S32BE.0,
    /// 32-bit floating point samples, little-endian
    F32LE = sys::audio::SDL_AUDIO_F32LE.0,
    /// 32-bit floating point samples, big-endian
    F32BE = sys::audio::SDL_AUDIO_F32BE.0,
}

impl AudioFormat {
    fn from_ll(raw: sys::audio::SDL_AudioFormat) -> Option<AudioFormat> {
        use self::AudioFormat::*;
        match raw {
            sys::audio::SDL_AUDIO_S8 => Some(AudioFormat::S8),
            sys::audio::SDL_AUDIO_S16LE => Some(AudioFormat::S16LE),
            sys::audio::SDL_AUDIO_S16BE => Some(AudioFormat::S16BE),
            sys::audio::SDL_AUDIO_S32LE => Some(AudioFormat::S32LE),
            sys::audio::SDL_AUDIO_S32BE => Some(AudioFormat::S32BE),
            sys::audio::SDL_AUDIO_F32LE => Some(AudioFormat::F32LE),
            sys::audio::SDL_AUDIO_F32BE => Some(AudioFormat::F32BE),
            _ => None,
        }
    }

    #[doc(alias = "SDL_AudioFormat")]
    fn to_ll(self) -> sys::audio::SDL_AudioFormat {
        self.into()
    }
}

impl From<AudioFormat> for sys::audio::SDL_AudioFormat {
    fn from(format: AudioFormat) -> sys::audio::SDL_AudioFormat {
        match format {
            AudioFormat::U8 => sys::audio::SDL_AUDIO_U8,
            AudioFormat::S8 => sys::audio::SDL_AUDIO_S8,
            AudioFormat::S16LE => sys::audio::SDL_AUDIO_S16LE,
            AudioFormat::S16BE => sys::audio::SDL_AUDIO_S16BE,
            AudioFormat::S32LE => sys::audio::SDL_AUDIO_S32LE,
            AudioFormat::S32BE => sys::audio::SDL_AUDIO_S32BE,
            AudioFormat::F32LE => sys::audio::SDL_AUDIO_F32LE,
            AudioFormat::F32BE => sys::audio::SDL_AUDIO_F32BE,
        }
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
                let buf = sys::audio::SDL_GetAudioDriver(self.index);
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
        length: unsafe { sys::audio::SDL_GetNumAudioDrivers() },
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
            let ret = sys::audio::SDL_LoadWAV_IO(
                src.raw(),
                false,
                desired.as_mut_ptr(),
                &mut audio_buf,
                &mut audio_len,
            );
            if !ret {
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
            SDL_free(self.audio_buf as *mut _);
        }
    }
}

pub trait AudioCallback<Channel>: Send + 'static
where
    Channel: AudioFormatNum + 'static,
{
    fn callback(&mut self, out: &mut [Channel]);
}

/// A phantom type for retrieving the `SDL_AudioFormat` of a given generic type.
/// All format types are returned as native-endian.
pub trait AudioFormatNum: Copy + 'static {
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
    /// impl AudioCallback<u16> for Silence {
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

// extern "C" fn audio_callback_marshall<CB: AudioCallback>(
//     userdata: *mut c_void,
//     stream: *mut u8,
//     len: c_int,
// ) {
//     use std::mem::size_of;
//     use std::slice::from_raw_parts_mut;
//     unsafe {
//         let cb_userdata: &mut Option<CB> = &mut *(userdata as *mut _);
//         let buf: &mut [CB::Channel] = from_raw_parts_mut(
//             stream as *mut CB::Channel,
//             len as usize / size_of::<CB::Channel>(),
//         );
//
//         if let Some(cb) = cb_userdata {
//             cb.callback(buf);
//         }
//     }
// }

#[derive(Clone)]
pub struct AudioSpec {
    /// DSP frequency (samples per second). Set to None for the device's fallback frequency.
    pub freq: Option<i32>,
    /// Number of separate audio channels. Set to None for the device's fallback number of channels.
    pub channels: Option<i32>,
    /// Audio format. Set to None for the device's fallback audio format.
    pub format: Option<AudioFormat>,
}

impl Into<sys::audio::SDL_AudioSpec> for AudioSpec {
    fn into(self) -> sys::audio::SDL_AudioSpec {
        AudioSpec::convert_to_ll(self.freq, self.channels, self.format)
    }
}

impl AudioSpec {
    fn convert_to_ll<R, C, F>(rate: R, channels: C, format: F) -> sys::audio::SDL_AudioSpec
    where
        R: Into<Option<i32>>,
        C: Into<Option<i32>>,
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

impl AudioSpec {
    fn convert_from_ll(spec: sys::audio::SDL_AudioSpec) -> AudioSpec {
        AudioSpec {
            freq: Some(spec.freq.into()),
            format: AudioFormat::from_ll(spec.format),
            channels: Some(spec.channels),
        }
    }
}

enum AudioDeviceID {
    Device(sys::audio::SDL_AudioDeviceID),
}

impl AudioDeviceID {
    fn id(&self) -> sys::audio::SDL_AudioDeviceID {
        match *self {
            AudioDeviceID::Device(id) => id,
        }
    }
}

impl Drop for AudioDeviceID {
    #[doc(alias = "SDL_CloseAudioDevice")]
    fn drop(&mut self) {
        //! Shut down audio processing and close the audio device.
        println!("Closing audio device: {}", self.id());
        unsafe { sys::audio::SDL_CloseAudioDevice(self.id()) }
        println!("Closed audio device: {}", self.id());
    }
}

/// Represents an open audio device (playback or recording).
pub struct AudioDevice {
    device_id: sys::audio::SDL_AudioDeviceID,
    // subsystem: &'a AudioSubsystem,
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe {
            sys::audio::SDL_CloseAudioDevice(self.device_id);
        }
    }
}

impl AudioDevice {
    /// Create an `AudioStream` for this device with the specified spec.
    pub fn open_stream(&self, spec: &AudioSpec) -> Result<AudioStream, String> {
        let sdl_spec = spec.to_sdl_spec();
        let stream = unsafe {
            sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id,
                &sdl_spec,
                None,
                std::ptr::null_mut(),
            )
        };
        if stream.is_null() {
            Err(get_error())
        } else {
            Ok(AudioStream {
                stream,
                device_id: self.device_id,
            })
        }
    }

    /// Opens a new audio device for playback or recording (given the desired parameters).
    #[doc(alias = "SDL_OpenAudioDevice")]
    fn open<'a, D>(
        a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
        recording: bool,
    ) -> Result<AudioDevice, String>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        use std::mem::MaybeUninit;

        let desired = AudioSpec::convert_to_ll(spec.freq, spec.channels, spec.format);

        let mut obtained = MaybeUninit::uninit();
        unsafe {
            let sdl_device = match device.into() {
                Some(device) => device.id(),
                // use default device if no device is specified
                None => {
                    if recording {
                        SDL_AUDIO_DEVICE_DEFAULT_RECORDING
                    } else {
                        SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK
                    }
                }
            };
            let device_id = sys::audio::SDL_OpenAudioDevice(sdl_device, &desired);
            match device_id {
                0 => Err(get_error()),
                id => {
                    let obtained = obtained.assume_init();
                    let device_id = AudioDeviceID::Device(id);
                    let spec = AudioSpec::convert_from_ll(obtained);
                    let name = if recording {
                        a.audio_recording_device_name(id as u32)?
                    } else {
                        a.audio_playback_device_name(id as u32)?
                    };

                    Ok(AudioDevice {
                        subsystem: a,
                        device_id: device_id.into(),
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

    /// Opens a new audio device for recording (given the desired parameters).
    pub fn open_recording<'a, D>(
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
    pub fn pause(&self) -> bool {
        unsafe { sys::audio::SDL_PauseAudioDevice(self.device_id.id()) }
    }

    /// Starts playback of the audio device.
    #[doc(alias = "SDL_ResumeAudioDevice")]
    pub fn resume(&self) -> bool {
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
    /// This function works with both playback and recording devices.
    // The open_stream function
    pub fn open_stream_with_callback<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStream<CB>, String>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let mut sdl_audiospec: sys::audio::SDL_AudioSpec = spec.clone().into();

        if sdl_audiospec.format != Channel::audio_format().to_ll() {
            return Err("AudioSpec format does not match AudioCallback Channel type".to_string());
        }

        let callback_box = Box::new(callback);
        let c_userdata = Box::into_raw(callback_box) as *mut c_void;

        unsafe extern "C" fn audio_stream_callback<CB, Channel>(
            userdata: *mut c_void,
            sdl_stream: *mut sys::audio::SDL_AudioStream,
            len: c_int,
            _bytes: c_int,
        ) where
            CB: AudioCallback<Channel>,
            Channel: AudioFormatNum + 'static,
        {
            let callback = &mut *(userdata as *mut CB);

            let sample_count = len as usize / std::mem::size_of::<Channel>();

            let mut buffer = vec![Channel::SILENCE; sample_count];

            callback.callback(&mut buffer);

            let buffer_ptr = buffer.as_ptr() as *const c_void;

            let ret = sys::audio::SDL_PutAudioStreamData(sdl_stream, buffer_ptr, len);

            if !ret {
                eprintln!("Error pushing audio data into stream: {}", get_error());
            }
        }

        unsafe {
            let stream = sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id.id(),
                &sdl_audiospec,
                Some(audio_stream_callback::<CB, Channel>),
                c_userdata,
            );

            if stream.is_null() {
                Box::from_raw(c_userdata as *mut CB);
                Err(get_error())
            } else {
                Ok(AudioStream {
                    stream,
                    _callback: PhantomData,
                    c_userdata,
                })
            }
        }
    }
}

pub struct AudioStream<CB> {
    stream: *mut sys::audio::SDL_AudioStream,
    _callback: PhantomData<CB>,
    c_userdata: *mut c_void,
}

impl<CB> Drop for AudioStream<CB> {
    fn drop(&mut self) {
        unsafe {
            sys::audio::SDL_DestroyAudioStream(self.stream);
            Box::from_raw(self.c_userdata as *mut CB);
        }
    }
}

impl<CB> AudioStream<CB> {
    pub fn get_stream(&mut self) -> *mut sys::audio::SDL_AudioStream {
        self.stream
    }

    fn get_device_id(&mut self) -> &mut AudioDeviceID {
        let device = unsafe { sys::audio::SDL_GetAudioStreamDevice(self.stream) };
        unsafe { &mut *(device as *mut AudioDeviceID) }
    }

    pub fn pause(&mut self) -> bool {
        unsafe { sys::audio::SDL_PauseAudioStreamDevice(self.stream) }
    }
    pub fn resume(&mut self) -> bool {
        unsafe { sys::audio::SDL_ResumeAudioStreamDevice(self.stream) }
    }

    /// Locks the audio stream using `SDL_LockAudioStream`.
    ///
    /// If the app assigns a callback to a specific stream, it can use the stream's lock through
    /// SDL_LockAudioStream() if necessary.
    #[doc(alias = "SDL_LockAudioStream")]
    pub fn lock(&mut self) -> AudioStreamLockGuard<CB> {
        unsafe { sys::audio::SDL_LockAudioStream(self.get_stream()) };
        AudioStreamLockGuard {
            stream: self,
            _nosend: PhantomData,
        }
    }
}

/// Similar to `std::sync::MutexGuard`, but for use with `AudioStream::lock()`.
pub struct AudioStreamLockGuard<'a, CB, F>
where
    CB: AudioCallback<F>,
    CB: 'a,
    F: AudioFormatNum + 'static,
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
