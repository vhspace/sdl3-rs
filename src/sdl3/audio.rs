//! Audio Functions
//!
//! # Example
//! ```no_run
//! use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream};
//! use std::time::Duration;
//!
//! struct SquareWave {
//!     phase_inc: f32,
//!     phase: f32,
//!     volume: f32
//! }
//!
//! impl AudioCallback<f32> for SquareWave {
//!     fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
//!         let mut out = Vec::<f32>::with_capacity(requested as usize);
//!         // Generate a square wave
//!         for _ in 0..requested {
//!             out.push(if self.phase <= 0.5 {
//!                 self.volume
//!             } else {
//!                 -self.volume
//!             });
//!             self.phase = (self.phase + self.phase_inc) % 1.0;
//!         }
//!         stream.put_data_f32(&out);
//!     }
//! }
//!
//! let sdl_context = sdl3::init().unwrap();
//! let audio_subsystem = sdl_context.audio().unwrap();
//!
//! let source_freq = 44100;
//! let source_spec = AudioSpec {
//!     freq: Some(source_freq),
//!     channels: Some(1),                      // mono
//!     format: Some(AudioFormat::f32_sys())    // floating 32 bit samples
//! };
//!
//! // Initialize the audio callback
//! let device = audio_subsystem.open_playback_stream(&source_spec, SquareWave {
//!     phase_inc: 440.0 / source_freq as f32,
//!     phase: 0.0,
//!     volume: 0.25
//! }).unwrap();
//!
//! // Start playback
//! device.resume().expect("Failed to start playback");
//!
//! // Play for 2 seconds
//! std::thread::sleep(Duration::from_millis(2000));
//! ```

use crate::get_error;
use crate::iostream::IOStream;
use crate::sys;
use crate::AudioSubsystem;
use crate::Error;
use libc::c_void;
use std::convert::TryInto;
use std::ffi::{c_int, CStr};
use std::fmt;
use std::fmt::{Debug, Display};
use std::io::{self, Read};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use sys::audio::{SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, SDL_AUDIO_DEVICE_DEFAULT_RECORDING};
use sys::stdinc::SDL_free;

impl AudioSubsystem {
    /// Enumerate audio playback devices.
    #[doc(alias = "SDL_GetAudioPlaybackDevices")]
    pub fn audio_playback_device_ids(&self) -> Result<Vec<AudioDeviceID>, Error> {
        unsafe {
            self.audio_device_ids(|num_devices| {
                sys::audio::SDL_GetAudioPlaybackDevices(num_devices)
            })
        }
    }

    /// Enumerate audio recording devices.
    #[doc(alias = "SDL_GetAudioRecordingDevices")]
    pub fn audio_recording_device_ids(&self) -> Result<Vec<AudioDeviceID>, Error> {
        self.audio_device_ids(|num_devices| unsafe {
            sys::audio::SDL_GetAudioRecordingDevices(num_devices)
        })
    }

    fn audio_device_ids<F>(&self, get_devices: F) -> Result<Vec<AudioDeviceID>, Error>
    where
        F: FnOnce(&mut i32) -> *mut sys::audio::SDL_AudioDeviceID,
    {
        let mut num_devices: i32 = 0;
        let devices = get_devices(&mut num_devices);
        if devices.is_null() {
            return Err(get_error());
        }

        let mut ret = Vec::new();
        for i in 0..num_devices {
            let instance_id = unsafe { *devices.offset(i as isize) };
            ret.push(AudioDeviceID::Device(instance_id));
        }

        unsafe { SDL_free(devices as *mut c_void) };
        Ok(ret)
    }
    /// Open a default playback device with the specified audio spec.
    pub fn open_playback_device(&self, spec: &AudioSpec) -> Result<AudioDevice, Error> {
        self.open_device(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, spec)
    }

    /// Open a default recording device with the specified audio spec.
    pub fn open_recording_device(&self, spec: &AudioSpec) -> Result<AudioDevice, Error> {
        self.open_device(SDL_AUDIO_DEVICE_DEFAULT_RECORDING, spec)
    }

    pub fn default_playback_device(&self) -> AudioDevice {
        AudioDevice::new(
            AudioDeviceID::Device(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK),
            self.clone(),
        )
    }

    pub fn default_recording_device(&self) -> AudioDevice {
        AudioDevice::new(
            AudioDeviceID::Device(SDL_AUDIO_DEVICE_DEFAULT_RECORDING),
            self.clone(),
        )
    }

    /// General method to open a device by ID.
    fn open_device(
        &self,
        device_id: sys::audio::SDL_AudioDeviceID,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, Error> {
        let sdl_spec: sys::audio::SDL_AudioSpec = spec.clone().into();
        let device = unsafe { sys::audio::SDL_OpenAudioDevice(device_id, &sdl_spec) };
        if device == 0 {
            Err(get_error())
        } else {
            Ok(AudioDevice::new(
                AudioDeviceID::Device(device),
                self.clone(),
            ))
        }
    }

    pub fn open_playback_stream_with_callback<CB, Channel>(
        &self,
        device: &AudioDevice,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStreamWithCallback<CB>, Error>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        device.open_playback_stream_with_callback(spec, callback)
    }

    pub fn open_playback_stream<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStreamWithCallback<CB>, Error>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let device = AudioDevice::open_playback(self, None, spec)?;
        device.open_playback_stream_with_callback(spec, callback)
    }

    pub fn open_recording_stream<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStreamWithCallback<CB>, Error>
    where
        CB: AudioRecordingCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let device = AudioDevice::open_recording(self, None, spec)?;
        device.open_recording_stream_with_callback(spec, callback)
    }

    #[doc(alias = "SDL_GetCurrentAudioDriver")]
    pub fn current_audio_driver(&self) -> &'static str {
        unsafe {
            let buf = sys::audio::SDL_GetCurrentAudioDriver();
            assert!(!buf.is_null());

            CStr::from_ptr(buf as *const _).to_str().unwrap()
        }
    }

    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn audio_playback_device_name(&self, index: u32) -> Result<String, Error> {
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
    pub fn audio_recording_device_name(&self, index: u32) -> Result<String, Error> {
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

    /// Creates a new audio stream that converts audio data from the source format (`src_spec`)
    /// to the destination format (`dst_spec`).
    ///
    /// # Arguments
    ///
    /// * `src_spec` - The format details of the input audio.
    /// * `dst_spec` - The format details of the output audio.
    ///
    /// # Returns
    ///
    /// Returns `Ok(AudioStream)` on success or an error message on failure.
    ///
    /// # Safety
    ///
    /// This function is safe to call from any thread.
    pub fn new_stream(
        &self,
        src_spec: Option<&AudioSpec>,
        dst_spec: Option<&AudioSpec>,
    ) -> Result<AudioStreamOwner, Error> {
        let sdl_src_spec = src_spec.map(sys::audio::SDL_AudioSpec::from);
        let sdl_dst_spec = dst_spec.map(sys::audio::SDL_AudioSpec::from);

        let sdl_src_spec_ptr = sdl_src_spec
            .as_ref()
            .map_or(std::ptr::null(), |spec| spec as *const _);
        let sdl_dst_spec_ptr = sdl_dst_spec
            .as_ref()
            .map_or(std::ptr::null(), |spec| spec as *const _);

        let stream =
            unsafe { sys::audio::SDL_CreateAudioStream(sdl_src_spec_ptr, sdl_dst_spec_ptr) };
        if stream.is_null() {
            Err(get_error())
        } else {
            Ok(AudioStreamOwner {
                inner: AudioStream { stream },
                audio_subsystem: self.clone().into(),
            })
        }
    }

    /// Creates a new audio stream for playback.
    ///
    /// # Arguments
    ///
    /// * `app_spec` - The format of audio data the application will provide.
    /// * `device_spec` - The format of audio data the audio device expects.
    ///                   If `None`, SDL will choose an appropriate format.
    pub fn new_playback_stream(
        &self,
        app_spec: &AudioSpec,
        device_spec: Option<&AudioSpec>,
    ) -> Result<AudioStreamOwner, Error> {
        self.new_stream(Some(app_spec), device_spec)
    }

    /// Creates a new audio stream for recording.
    ///
    /// # Arguments
    ///
    /// * `device_spec` - The format of audio data the audio device provides.
    ///                   If `None`, SDL will choose an appropriate format.
    /// * `app_spec` - The format of audio data the application wants to receive.
    pub fn new_recording_stream(
        &self,
        device_spec: Option<&AudioSpec>,
        app_spec: &AudioSpec,
    ) -> Result<AudioStreamOwner, Error> {
        self.new_stream(device_spec, Some(app_spec))
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AudioFormat {
    UNKNOWN = sys::audio::SDL_AUDIO_UNKNOWN.0,

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
        match raw {
            sys::audio::SDL_AUDIO_UNKNOWN => Some(AudioFormat::UNKNOWN),
            sys::audio::SDL_AUDIO_U8 => Some(AudioFormat::U8),
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
            AudioFormat::UNKNOWN => sys::audio::SDL_AUDIO_UNKNOWN,
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
    pub fn load_wav<P: AsRef<Path>>(path: P) -> Result<AudioSpecWAV, Error> {
        let mut file = IOStream::from_file(path, "rb")?;
        AudioSpecWAV::load_wav_rw(&mut file)
    }

    /// Loads a WAVE from the data source.
    #[doc(alias = "SDL_LoadWAV_RW")]
    pub fn load_wav_rw(src: &mut IOStream) -> Result<AudioSpecWAV, Error> {
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
    fn callback(&mut self, stream: &mut AudioStream, requested: i32);
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
    /// use sdl3::audio::{AudioCallback, AudioFormatNum, AudioStream};
    ///
    /// struct Silence;
    ///
    /// impl AudioCallback<f32> for Silence {
    ///     fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
    ///         let len = requested.max(0) as usize;
    ///         let samples = vec![<f32 as AudioFormatNum>::SILENCE; len];
    ///         let _ = stream.put_data_f32(&samples);
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

#[derive(Clone, Debug)]
pub struct AudioSpec {
    /// DSP frequency (samples per second). Set to None for the device's fallback frequency.
    pub freq: Option<i32>,
    /// Number of separate audio channels. Set to None for the device's fallback number of channels.
    pub channels: Option<i32>,
    /// Audio format. Set to None for the device's fallback audio format.
    pub format: Option<AudioFormat>,
}

impl From<AudioSpec> for sys::audio::SDL_AudioSpec {
    fn from(val: AudioSpec) -> Self {
        AudioSpec::convert_to_ll(val.freq, val.channels, val.format)
    }
}

impl From<&AudioSpec> for sys::audio::SDL_AudioSpec {
    fn from(spec: &AudioSpec) -> Self {
        sys::audio::SDL_AudioSpec {
            freq: spec.freq.unwrap_or(0), // SDL uses 0 to indicate default frequency
            format: spec.format.unwrap_or(AudioFormat::UNKNOWN).to_ll(), // Use AudioFormat::Unknown for default
            channels: spec.channels.unwrap_or(0), // SDL uses 0 to indicate default channels
        }
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

        sys::audio::SDL_AudioSpec {
            freq: freq.unwrap_or(0),
            format: format.unwrap_or(AudioFormat::UNKNOWN).to_ll(),
            channels: channels.unwrap_or(0),
        }
    }
}

impl From<&sys::audio::SDL_AudioSpec> for AudioSpec {
    fn from(sdl_spec: &sys::audio::SDL_AudioSpec) -> Self {
        Self {
            freq: if sdl_spec.freq != 0 {
                Some(sdl_spec.freq)
            } else {
                None // SDL used default frequency
            },
            format: if sdl_spec.format != sys::audio::SDL_AUDIO_UNKNOWN {
                Some(AudioFormat::from_ll(sdl_spec.format).expect("Unknown audio format"))
            } else {
                None // SDL used default format
            },
            channels: if sdl_spec.channels != 0 {
                Some(sdl_spec.channels)
            } else {
                None // SDL used default channels
            },
        }
    }
}
impl AudioSpec {
    /// Creates a new `AudioSpec` with specified values.
    /// Use `None` for any parameter to indicate the device's default value.
    pub fn new(freq: Option<i32>, channels: Option<i32>, format: Option<AudioFormat>) -> Self {
        Self {
            freq,
            channels,
            format,
        }
    }

    // fn convert_from_ll(spec: sys::audio::SDL_AudioSpec) -> AudioSpec {
    //     AudioSpec {
    //         freq: Some(spec.freq.into()),
    //         format: AudioFormat::from_ll(spec.format),
    //         channels: Some(spec.channels),
    //     }
    // }
}

impl Default for AudioSpec {
    /// Creates an `AudioSpec` with all fields set to `None` (use device defaults).
    fn default() -> Self {
        Self {
            freq: None,
            channels: None,
            format: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AudioDeviceID {
    Device(sys::audio::SDL_AudioDeviceID),
}

impl Copy for AudioDeviceID {}

impl AudioDeviceID {
    pub fn id(&self) -> sys::audio::SDL_AudioDeviceID {
        match *self {
            AudioDeviceID::Device(id) => id,
        }
    }

    pub fn name(&self) -> Result<String, Error> {
        unsafe {
            let name_ptr = sys::audio::SDL_GetAudioDeviceName(self.id());
            if name_ptr.is_null() {
                return Err(get_error());
            }
            Ok(CStr::from_ptr(name_ptr).to_str().unwrap().to_owned())
        }
    }
}

impl PartialEq for AudioDeviceID {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for AudioDeviceID {}

/// Represents an open audio device (playback or recording).
#[derive(Clone)]
pub struct AudioDevice {
    device_id: AudioDeviceID,
    // keep the audio subsystem alive
    audio_subsystem: AudioSubsystem,
}

impl PartialEq for AudioDevice {
    fn eq(&self, other: &Self) -> bool {
        self.device_id == other.device_id
    }
}

impl Eq for AudioDevice {}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe {
            sys::audio::SDL_CloseAudioDevice(self.device_id.id());
        }
    }
}

impl AudioDevice {
    pub fn id(&self) -> AudioDeviceID {
        self.device_id
    }

    pub fn new(device_id: AudioDeviceID, audio_subsystem: AudioSubsystem) -> Self {
        AudioDevice {
            device_id,
            audio_subsystem,
        }
    }

    /// Get the name of the audio device.
    #[doc(alias = "SDL_GetAudioDeviceName")]
    pub fn name(&self) -> Result<String, Error> {
        unsafe {
            let name_ptr = sys::audio::SDL_GetAudioDeviceName(self.device_id.id());
            if name_ptr.is_null() {
                return Err(get_error());
            }
            Ok(CStr::from_ptr(name_ptr).to_str().unwrap().to_owned())
        }
    }

    /// Create an `AudioStream` for this device with the specified spec.
    /// This device will be closed when the stream is dropped.
    /// The device begins paused, so you must call `stream.resume()` to start playback.
    #[doc(alias = "SDL_OpenAudioDeviceStream")]
    pub fn open_device_stream(self, spec: Option<&AudioSpec>) -> Result<AudioStreamOwner, Error> {
        let sdl_spec = spec.map(|spec| spec.into());
        let sdl_spec_ptr = crate::util::option_to_ptr(sdl_spec.as_ref());

        let stream = unsafe {
            sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id.id(),
                sdl_spec_ptr,
                // not using callbacks here
                None,
                std::ptr::null_mut(),
            )
        };
        if stream.is_null() {
            Err(get_error())
        } else {
            // SDL will close the device when the stream is closed
            core::mem::forget(self);
            let audio_subsystem = unsafe { AudioSubsystem::new_unchecked() };

            Ok(AudioStreamOwner {
                inner: AudioStream { stream },
                audio_subsystem: audio_subsystem.into(),
            })
        }
    }

    /// Binds an audio stream to this device.
    #[doc(alias = "SDL_BindAudioStream")]
    pub fn bind_stream(&self, stream: &AudioStream) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_BindAudioStream(self.device_id.id(), stream.stream) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Binds multiple audio streams to this device.
    #[doc(alias = "SDL_BindAudioStreams")]
    pub fn bind_streams(&self, streams: &[&AudioStream]) -> Result<(), Error> {
        let streams_ptrs: Vec<*mut sys::audio::SDL_AudioStream> =
            streams.iter().map(|s| s.stream).collect();
        let result = unsafe {
            sys::audio::SDL_BindAudioStreams(
                self.device_id.id(),
                streams_ptrs.as_ptr() as *mut _,
                streams.len() as i32,
            )
        };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Opens a new audio device for playback or recording (given the desired parameters).
    #[doc(alias = "SDL_OpenAudioDevice")]
    fn open<'a, D>(
        device: D,
        spec: &AudioSpec,
        recording: bool,
        audio_subsystem: &AudioSubsystem,
    ) -> Result<AudioDevice, Error>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        let desired = AudioSpec::convert_to_ll(spec.freq, spec.channels, spec.format);

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
                    let device_id = AudioDeviceID::Device(id);

                    Ok(AudioDevice::new(device_id, audio_subsystem.clone()))
                }
            }
        }
    }

    /// Opens a new audio device for playback (given the desired parameters).
    pub fn open_playback<'a, D>(
        _a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, Error>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open(device, spec, false, _a)
    }

    /// Opens a new audio device for recording (given the desired parameters).
    pub fn open_recording<'a, D>(
        _a: &AudioSubsystem,
        device: D,
        spec: &AudioSpec,
    ) -> Result<AudioDevice, Error>
    where
        D: Into<Option<&'a AudioDeviceID>>,
    {
        AudioDevice::open(device, spec, true, _a)
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

    /// Opens a new audio stream for this device with the specified spec.
    /// The device begins paused, so you must call `stream.resume()` to start playback.
    #[doc(alias = "SDL_OpenAudioDeviceStream")]
    pub fn open_playback_stream_with_callback<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStreamWithCallback<CB>, Error>
    where
        CB: AudioCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        let sdl_audiospec: sys::audio::SDL_AudioSpec = spec.clone().into();

        if sdl_audiospec.format != Channel::audio_format().to_ll() {
            return Err(Error(
                "AudioSpec format does not match AudioCallback Channel type".to_string(),
            ));
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

            let mut stream = AudioStream { stream: sdl_stream };

            callback.callback(&mut stream, len / size_of::<Channel>() as i32);
        }

        unsafe {
            let stream = sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id.id(),
                &sdl_audiospec,
                Some(audio_stream_callback::<CB, Channel>),
                c_userdata,
            );

            if stream.is_null() {
                // Drop the callback box
                let _ = Box::from_raw(c_userdata as *mut CB);
                Err(get_error())
            } else {
                Ok(AudioStreamWithCallback {
                    base_stream: AudioStreamOwner {
                        inner: AudioStream { stream },
                        audio_subsystem: self.audio_subsystem.clone().into(),
                    },
                    _marker: PhantomData,
                    c_userdata,
                })
            }
        }
    }

    /// Opens a new audio stream for recording with the specified spec.
    /// The device begins paused, so you must call `stream.resume()` to start recording.
    #[doc(alias = "SDL_OpenAudioDeviceStream")]
    pub fn open_recording_stream_with_callback<CB, Channel>(
        &self,
        spec: &AudioSpec,
        callback: CB,
    ) -> Result<AudioStreamWithCallback<CB>, Error>
    where
        CB: AudioRecordingCallback<Channel>,
        Channel: AudioFormatNum + 'static,
    {
        // Convert Rust AudioSpec to SDL_AudioSpec
        let sdl_audiospec: sys::audio::SDL_AudioSpec = spec.clone().into();

        if sdl_audiospec.format != Channel::audio_format().to_ll() {
            return Err(Error(
                "AudioSpec format does not match AudioCallback Channel type".to_string(),
            ));
        }

        let callback_box = Box::new(callback);
        let c_userdata = Box::into_raw(callback_box) as *mut c_void;

        unsafe {
            let stream = sys::audio::SDL_OpenAudioDeviceStream(
                self.device_id.id(),
                &sdl_audiospec,
                Some(audio_recording_stream_callback::<CB, Channel>),
                c_userdata,
            );

            if stream.is_null() {
                // Drop the callback box
                let _ = Box::from_raw(c_userdata as *mut CB);
                Err(get_error())
            } else {
                Ok(AudioStreamWithCallback {
                    base_stream: AudioStreamOwner {
                        inner: AudioStream { stream },
                        audio_subsystem: self.audio_subsystem.clone().into(),
                    },
                    _marker: PhantomData,
                    c_userdata,
                })
            }
        }
    }
}

pub struct AudioStreamOwner {
    inner: AudioStream,
    #[expect(dead_code)]
    audio_subsystem: Option<AudioSubsystem>,
}

pub struct AudioStream {
    stream: *mut sys::audio::SDL_AudioStream,
}

impl Deref for AudioStreamOwner {
    type Target = AudioStream;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AudioStreamOwner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for AudioStreamOwner {
    /// Destroys the audio stream, unbinding it automatically from the device.
    /// If this stream was created with SDL_OpenAudioDeviceStream, the audio device that was opened alongside this stream’s creation will be closed, too.
    fn drop(&mut self) {
        if !self.inner.stream.is_null() {
            unsafe {
                sys::audio::SDL_DestroyAudioStream(self.inner.stream);
            }
            self.inner.stream = std::ptr::null_mut();
        }
    }
}

impl Debug for AudioStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Get the device "name [ID]"
        let device_name = self
            .device_id()
            .and_then(|id| id.name().ok())
            .unwrap_or("Unknown".to_string());
        let device_name = format!(
            "{} [{}]",
            device_name,
            self.device_id().map(|id| id.id()).unwrap_or(0)
        );

        // Get the audio specs
        let (src_spec, dst_spec) = match self.get_format() {
            Ok((src, dst)) => (Some(src), Some(dst)),
            Err(_) => (None, None),
        };

        // Get the gain
        let gain = self.get_gain().ok();

        // Begin building the debug struct
        let mut ds = f.debug_struct("AudioStream");

        ds.field("device", &device_name);

        if let Some(src_spec) = src_spec {
            ds.field("src_spec", &src_spec);
        } else {
            ds.field("src_spec", &"Unknown");
        }

        if let Some(dst_spec) = dst_spec {
            ds.field("dst_spec", &dst_spec);
        } else {
            ds.field("dst_spec", &"Unknown");
        }

        if let Some(gain) = gain {
            ds.field("gain", &gain);
        } else {
            ds.field("gain", &"Unknown");
        }

        ds.finish()
    }
}
impl Display for AudioStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.device_id().and_then(|id| id.name().ok()) {
            write!(f, "AudioStream({name})")
        } else {
            write!(f, "AudioStream")
        }
    }
}

impl AudioStream {
    /// Get the SDL_AudioStream pointer.
    #[doc(alias = "SDL_AudioStream")]
    pub fn stream(&mut self) -> *mut sys::audio::SDL_AudioStream {
        self.stream
    }

    /// Get the device ID bound to the stream.
    /// If the stream is not bound to a device, this will return `None`.
    #[doc(alias = "SDL_GetAudioStreamDevice")]
    pub fn device_id(&self) -> Option<AudioDeviceID> {
        let device_id = unsafe { sys::audio::SDL_GetAudioStreamDevice(self.stream) };
        // If not bound, or invalid, this returns zero, which is not a valid device ID.
        if device_id != 0 {
            Some(AudioDeviceID::Device(device_id))
        } else {
            None
        }
    }

    pub fn device_name(&self) -> Option<String> {
        self.device_id().and_then(|id| id.name().ok())
    }

    /// Retrieves the source and destination formats of the audio stream.
    ///
    /// Returns a tuple `(src_spec, dst_spec)` where each is an `Option<AudioSpec>`.
    #[doc(alias = "SDL_GetAudioStreamFormat")]
    pub fn get_format(&self) -> Result<(Option<AudioSpec>, Option<AudioSpec>), Error> {
        let mut sdl_src_spec = AudioSpec::default().into();
        let mut sdl_dst_spec = AudioSpec::default().into();
        let result = unsafe {
            sys::audio::SDL_GetAudioStreamFormat(self.stream, &mut sdl_src_spec, &mut sdl_dst_spec)
        };
        if result {
            let src_spec = if sdl_src_spec.format != sys::audio::SDL_AUDIO_UNKNOWN {
                Some(AudioSpec::from(&sdl_src_spec))
            } else {
                None
            };
            let dst_spec = if sdl_dst_spec.format != sys::audio::SDL_AUDIO_UNKNOWN {
                Some(AudioSpec::from(&sdl_dst_spec))
            } else {
                None
            };
            Ok((src_spec, dst_spec))
        } else {
            Err(get_error())
        }
    }

    /// Retrieves the gain of the audio stream.
    ///
    /// Returns the gain as a `f32` on success, or an error message on failure.
    #[doc(alias = "SDL_GetAudioStreamGain")]
    pub fn get_gain(&self) -> Result<f32, Error> {
        let gain = unsafe { sys::audio::SDL_GetAudioStreamGain(self.stream) };
        if gain >= 0.0 {
            Ok(gain)
        } else {
            Err(get_error())
        }
    }

    /// Change the gain of an audio stream.
    #[doc(alias = "SDL_SetAudioStreamGain")]
    pub fn set_gain(&self, gain: f32) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_SetAudioStreamGain(self.stream, gain) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Pauses playback of the audio stream.
    #[doc(alias = "SDL_PauseAudioStream")]
    pub fn pause(&self) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_PauseAudioStreamDevice(self.stream) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Resumes playback of the audio stream.
    #[doc(alias = "SDL_ResumeAudioStream")]
    pub fn resume(&self) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_ResumeAudioStreamDevice(self.stream) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Gets the number of converted/resampled bytes available.
    #[doc(alias = "SDL_GetAudioStreamAvailable")]
    pub fn available_bytes(&self) -> Result<i32, Error> {
        let available = unsafe { sys::audio::SDL_GetAudioStreamAvailable(self.stream) };
        if available == -1 {
            Err(get_error())
        } else {
            Ok(available)
        }
    }

    /// Gets the number of bytes queued.
    #[doc(alias = "SDL_GetAudioStreamQueued")]
    pub fn queued_bytes(&self) -> Result<i32, Error> {
        let queue = unsafe { sys::audio::SDL_GetAudioStreamQueued(self.stream) };
        if queue == -1 {
            Err(get_error())
        } else {
            Ok(queue)
        }
    }

    /// Clears any pending data.
    ///
    /// This drops any queued data, so there will be nothing to read from
    /// the stream until more is added.
    #[doc(alias = "SDL_ClearAudioStream")]
    pub fn clear(&self) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_ClearAudioStream(self.stream) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Tell the stream that you're done sending data.
    ///
    /// It is legal to add more data to a stream after flushing,
    /// but there may be audio gaps in the output.
    /// Generally this is intended to signal the end of input,
    /// so the complete output becomes available.
    #[doc(alias = "SDL_FlushAudioStream")]
    pub fn flush(&self) -> Result<(), Error> {
        let result = unsafe { sys::audio::SDL_FlushAudioStream(self.stream) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Converts a slice of bytes to a f32 sample based on AudioFormat.
    /// Returns a Result containing the converted f32 or an error message.
    fn read_bytes_to_f32(&self, chunk: &[u8]) -> Result<f32, Error> {
        // TODO: store specs so we don't have to call get_format every time
        let (_, output_spec) = self.get_format()?;
        match output_spec.unwrap().format {
            Some(AudioFormat::F32LE) => {
                Ok(f32::from_le_bytes(chunk.try_into().map_err(|_| {
                    Error("Invalid byte slice length for f32 LE".to_owned())
                })?))
            }
            Some(AudioFormat::F32BE) => {
                Ok(f32::from_be_bytes(chunk.try_into().map_err(|_| {
                    Error("Invalid byte slice length for f32 BE".to_owned())
                })?))
            }
            _ => Err(Error(
                "Unsupported AudioFormat for f32 conversion".to_string(),
            )),
        }
    }

    /// Converts a slice of bytes to an i16 sample based on AudioFormat.
    /// Returns a Result containing the converted i16 or an error message.
    fn read_bytes_to_i16(&self, chunk: &[u8]) -> Result<i16, Error> {
        // TODO: store specs so we don't have to call get_format every time
        let (_, output_spec) = self.get_format()?;
        match output_spec.unwrap().format {
            Some(AudioFormat::S16LE) => {
                Ok(i16::from_le_bytes(chunk.try_into().map_err(|_| {
                    Error("Invalid byte slice length for i16 LE".to_owned())
                })?))
            }
            Some(AudioFormat::S16BE) => {
                Ok(i16::from_be_bytes(chunk.try_into().map_err(|_| {
                    Error("Invalid byte slice length for i16 BE".to_owned())
                })?))
            }
            _ => Err(Error(
                "Unsupported AudioFormat for i16 conversion".to_string(),
            )),
        }
    }

    /// Reads samples as f32 into the provided buffer.
    /// Returns the number of samples read.
    pub fn read_f32_samples(&mut self, buf: &mut [f32]) -> io::Result<usize> {
        let byte_len = std::mem::size_of_val(buf);
        let mut byte_buf = vec![0u8; byte_len];

        // Read bytes from the stream and capture the number of bytes read
        let bytes_read = self.read(&mut byte_buf)?;

        // Calculate the number of complete samples read
        let samples_read = bytes_read / size_of::<f32>();

        // Iterate over each complete sample
        for (i, v) in buf.iter_mut().enumerate().take(samples_read) {
            let start = i * size_of::<f32>();
            let end = start + size_of::<f32>();
            let chunk = &byte_buf[start..end];

            // Convert bytes to f32 and handle potential errors
            *v = self
                .read_bytes_to_f32(chunk)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }

        Ok(samples_read)
    }

    /// Reads samples as i16 into the provided buffer.
    /// Returns the number of samples read.
    pub fn read_i16_samples(&mut self, buf: &mut [i16]) -> io::Result<usize> {
        let byte_len = std::mem::size_of_val(buf);
        let mut byte_buf = vec![0u8; byte_len];

        // Read bytes from the stream and capture the number of bytes read
        let bytes_read = self.read(&mut byte_buf)?;

        // Calculate the number of complete samples read
        let samples_read = bytes_read / size_of::<i16>();

        // Iterate over each complete sample
        for (i, v) in buf.iter_mut().enumerate().take(samples_read) {
            let start = i * size_of::<i16>();
            let end = start + size_of::<i16>();
            let chunk = &byte_buf[start..end];

            // Convert bytes to i16 and handle potential errors
            *v = self
                .read_bytes_to_i16(chunk)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }

        Ok(samples_read)
    }

    /// Adds data to the stream.
    pub fn put_data(&self, buf: &[u8]) -> Result<(), Error> {
        let result = unsafe {
            sys::audio::SDL_PutAudioStreamData(self.stream, buf.as_ptr().cast(), buf.len() as i32)
        };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Adds data to the stream (16-bit signed).
    pub fn put_data_i16(&self, buf: &[i16]) -> Result<(), Error> {
        let result = unsafe {
            sys::audio::SDL_PutAudioStreamData(
                self.stream,
                buf.as_ptr().cast(),
                buf.len() as i32 * size_of::<i16>() as i32,
            )
        };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Adds data to the stream (32-bit float).
    pub fn put_data_f32(&self, buf: &[f32]) -> Result<(), Error> {
        let result = unsafe {
            sys::audio::SDL_PutAudioStreamData(
                self.stream,
                buf.as_ptr().cast(),
                buf.len() as i32 * size_of::<f32>() as i32,
            )
        };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

impl Read for AudioStream {
    /// Reads audio data from the stream.
    /// Note that this reads bytes from the stream, not samples.
    /// You must convert the bytes to samples based on the format of the stream.
    /// `read_f32_samples` and `read_i16_samples` are provided for convenience.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            sys::audio::SDL_GetAudioStreamData(
                self.stream,
                buf.as_mut_ptr().cast(),
                buf.len() as c_int,
            )
        };
        if ret == -1 {
            Err(io::Error::other(get_error()))
        } else {
            Ok(ret as usize)
        }
    }
}

// Streams with callbacks
pub struct AudioStreamWithCallback<CB> {
    base_stream: AudioStreamOwner,
    c_userdata: *mut c_void,
    _marker: PhantomData<CB>,
}

impl<CB> Drop for AudioStreamWithCallback<CB> {
    fn drop(&mut self) {
        // `base_stream` will be dropped automatically.
        if !self.c_userdata.is_null() {
            unsafe {
                // Drop the callback box
                let _ = Box::from_raw(self.c_userdata as *mut CB);
            }
            self.c_userdata = std::ptr::null_mut();
        }
    }
}

impl<CB> AudioStreamWithCallback<CB> {
    /// Pauses the audio stream.
    pub fn pause(&self) -> Result<(), Error> {
        self.base_stream.pause()
    }

    /// Resumes the audio stream.
    pub fn resume(&self) -> Result<(), Error> {
        self.base_stream.resume()
    }

    /// Clear any pending data in the stream.
    pub fn clear(&self) -> Result<(), Error> {
        self.base_stream.clear()
    }

    /// Tell the stream that you're done sending data.
    pub fn flush(&self) -> Result<(), Error> {
        self.base_stream.flush()
    }

    pub fn queued_bytes(&self) -> Result<i32, Error> {
        self.base_stream.queued_bytes()
    }

    pub fn lock(&mut self) -> Option<AudioStreamLockGuard<CB>> {
        let raw_stream = self.base_stream.stream;
        let result = unsafe { sys::audio::SDL_LockAudioStream(raw_stream) };

        if result {
            Some(AudioStreamLockGuard {
                stream: self,
                _nosend: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_gain(&mut self) -> Result<f32, Error> {
        self.base_stream.get_gain()
    }

    pub fn set_gain(&mut self, gain: f32) -> Result<(), Error> {
        self.base_stream.set_gain(gain)
    }
}

pub trait AudioRecordingCallback<Channel>: Send + 'static
where
    Channel: AudioFormatNum + 'static,
{
    fn callback(&mut self, stream: &mut AudioStream, available: i32);
}

unsafe extern "C" fn audio_recording_stream_callback<CB, Channel>(
    userdata: *mut c_void,
    sdl_stream: *mut sys::audio::SDL_AudioStream,
    len: c_int,
    _bytes: c_int,
) where
    CB: AudioRecordingCallback<Channel>,
    Channel: AudioFormatNum + 'static,
{
    let callback = &mut *(userdata as *mut CB);

    let mut stream = AudioStream { stream: sdl_stream };

    // Call the user's callback with the captured audio data
    callback.callback(&mut stream, len / size_of::<Channel>() as c_int);
}

/// Similar to `std::sync::MutexGuard`, but for use with `AudioStream::lock()`.
pub struct AudioStreamLockGuard<'a, CB>
where
    CB: 'a,
{
    stream: &'a mut AudioStreamWithCallback<CB>,
    _nosend: PhantomData<*mut ()>,
}

impl<'a, CB> Deref for AudioStreamLockGuard<'a, CB>
where
    CB: 'a,
{
    type Target = CB;
    #[doc(alias = "SDL_UnlockAudioStream")]
    fn deref(&self) -> &CB {
        unsafe {
            (self.stream.c_userdata as *const CB)
                .as_ref()
                .expect("Missing callback")
        }
    }
}

impl<'a, CB> DerefMut for AudioStreamLockGuard<'a, CB> {
    fn deref_mut(&mut self) -> &mut CB {
        unsafe {
            (self.stream.c_userdata as *mut CB)
                .as_mut()
                .expect("Missing callback")
        }
    }
}

impl<'a, CB> Drop for AudioStreamLockGuard<'a, CB> {
    fn drop(&mut self) {
        unsafe {
            sys::audio::SDL_UnlockAudioStream(self.stream.base_stream.stream);
        }
    }
}

#[cfg(test)]
mod test {}
