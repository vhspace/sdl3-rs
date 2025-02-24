use libc::c_char;
use std::cell::Cell;
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use sys::init::{
    SDL_INIT_AUDIO, SDL_INIT_CAMERA, SDL_INIT_EVENTS, SDL_INIT_GAMEPAD, SDL_INIT_HAPTIC,
    SDL_INIT_JOYSTICK, SDL_INIT_SENSOR, SDL_INIT_VIDEO,
};

use crate::sys;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Error(pub(crate) String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Error {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// True if the main thread has been declared. The main thread is declared when
/// SDL is first initialized.
static IS_MAIN_THREAD_DECLARED: AtomicBool = AtomicBool::new(false);

/// Number of active `SdlDrop` objects keeping SDL alive.
static SDL_COUNT: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// True if the current thread is the main thread.
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// The SDL context type. Initialize with `sdl3::init()`.
///
/// From a thread-safety perspective, `Sdl` represents the main thread.
/// As such, `Sdl` is a useful type for ensuring that SDL types that can only
/// be used on the main thread are initialized that way.
///
/// For instance, `SDL_PumpEvents()` is not thread safe, and may only be
/// called on the main thread.
/// All functionality that calls `SDL_PumpEvents()` is thus put into an
/// `EventPump` type, which can only be obtained through `Sdl`.
/// This guarantees that the only way to call event-pumping functions is on
/// the main thread.
#[derive(Clone)]
pub struct Sdl {
    sdldrop: SdlDrop,
}

impl Sdl {
    #[inline]
    #[doc(alias = "SDL_Init")]
    fn new() -> Result<Sdl, Error> {
        // Check if we can safely initialize SDL on this thread.
        let was_main_thread_declared = IS_MAIN_THREAD_DECLARED.swap(true, Ordering::SeqCst);

        IS_MAIN_THREAD.with(|is_main_thread| {
            if was_main_thread_declared {
                if !is_main_thread.get() {
                    // Since 'cargo test' runs its tests in a separate thread, we must disable
                    // this safety check during testing.
                    if !(cfg!(test) || cfg!(feature = "test-mode")) {
                        return Err(Error("Cannot initialize `Sdl` from a thread other than the main thread.  For testing, you can disable this check with the feature 'test-mode'.".to_owned()));
                    }
        }
            } else {
                is_main_thread.set(true);
            }
            Ok(())
        })?;

        // Initialize SDL.
        if SDL_COUNT.fetch_add(1, Ordering::Relaxed) == 0 {
            let result;

            unsafe {
                result = sys::init::SDL_Init(0);
            }

            if !result {
                SDL_COUNT.store(0, Ordering::Relaxed);
                return Err(get_error());
            }
        }

        Ok(Sdl {
            sdldrop: SdlDrop {
                marker: PhantomData,
            },
        })
    }

    /// Initializes the audio subsystem.
    #[inline]
    pub fn audio(&self) -> Result<AudioSubsystem, Error> {
        AudioSubsystem::new(self)
    }

    /// Initializes the event subsystem.
    #[inline]
    pub fn event(&self) -> Result<EventSubsystem, Error> {
        EventSubsystem::new(self)
    }

    /// Initializes the joystick subsystem.
    #[inline]
    pub fn joystick(&self) -> Result<JoystickSubsystem, Error> {
        JoystickSubsystem::new(self)
    }

    /// Initializes the haptic subsystem.
    #[inline]
    pub fn haptic(&self) -> Result<HapticSubsystem, Error> {
        HapticSubsystem::new(self)
    }

    /// Initializes the gamepad subsystem.
    #[inline]
    pub fn gamepad(&self) -> Result<GamepadSubsystem, Error> {
        GamepadSubsystem::new(self)
    }

    /// Initializes the game controller subsystem.
    #[inline]
    pub fn sensor(&self) -> Result<SensorSubsystem, Error> {
        SensorSubsystem::new(self)
    }

    /// Initializes the video subsystem.
    #[inline]
    pub fn video(&self) -> Result<VideoSubsystem, Error> {
        VideoSubsystem::new(self)
    }

    /// Obtains the SDL event pump.
    ///
    /// At most one `EventPump` is allowed to be alive during the program's execution.
    /// If this function is called while an `EventPump` instance is alive, the function will return
    /// an error.
    #[inline]
    pub fn event_pump(&self) -> Result<EventPump, Error> {
        EventPump::new(self)
    }

    #[inline]
    #[doc(hidden)]
    pub fn sdldrop(&self) -> SdlDrop {
        self.sdldrop.clone()
    }
}

/// When SDL is no longer in use, the library is quit.
#[doc(hidden)]
#[derive(Debug)]
pub struct SdlDrop {
    // Make it impossible to construct `SdlDrop` without access to this member,
    // and opt out of Send and Sync.
    marker: PhantomData<*mut ()>,
}

impl SdlDrop {
    /// Create an [`SdlDrop`] out of thin air.
    ///
    /// This is probably not what you are looking for. To initialize SDL use [`Sdl::new`].
    ///
    /// # Safety
    ///
    /// For each time this is called, previously an [`SdlDrop`] must have been passed to
    /// [`mem::forget`].
    unsafe fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl Clone for SdlDrop {
    fn clone(&self) -> SdlDrop {
        let prev_count = SDL_COUNT.fetch_add(1, Ordering::Relaxed);
        assert!(prev_count > 0);
        SdlDrop {
            marker: PhantomData,
        }
    }
}

impl Drop for SdlDrop {
    #[inline]
    #[doc(alias = "SDL_Quit")]
    fn drop(&mut self) {
        let prev_count = SDL_COUNT.fetch_sub(1, Ordering::Relaxed);
        assert!(prev_count > 0);
        if prev_count == 1 {
            unsafe {
                sys::init::SDL_Quit();
            }
            IS_MAIN_THREAD_DECLARED.store(false, Ordering::SeqCst);
        }
    }
}

// No subsystem can implement `Send` because the destructor, `SDL_QuitSubSystem`,
// utilizes non-atomic reference counting and should thus be called on a single thread.
// Some subsystems have functions designed to be thread-safe, such as adding a timer or accessing
// the event queue. These subsystems implement `Sync`.

macro_rules! subsystem {
    ($name:ident, $flag:expr, $counter:ident, nosync) => {
        static $counter: AtomicU32 = AtomicU32::new(0);

        #[derive(Debug)]
        pub struct $name {
            // Per subsystem all instances together keep one [`SdlDrop`].
            // Subsystems cannot be moved or (usually) used on non-main threads.
            /// This field makes sure [`Send`] and [`Sync`] are not implemented by default.
            marker: PhantomData<*mut ()>,
        }

        impl $name {
            #[inline]
            #[doc(alias = "SDL_InitSubSystem")]
            fn new(sdl: &Sdl) -> Result<Self, Error> {
                if $counter.fetch_add(1, Ordering::Relaxed) == 0 {
                    let result;

                    unsafe {
                        result = sys::init::SDL_InitSubSystem($flag);
                    }

                    if !result {
                        $counter.store(0, Ordering::Relaxed);
                        return Err(get_error());
                    }

                    // The first created subsystem instance "stores" an SdlDrop.
                    mem::forget(sdl.sdldrop.clone());
                }

                Ok(Self {
                    marker: PhantomData,
                })
            }

            #[doc = concat!("Create a [`", stringify!($name), "`] out of thin air.")]
            #[doc = ""]
            #[doc = concat!("This is probably not what you are looking for. To initialize the subsystem use [`", stringify!($name), "::new`].")]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = concat!("For each time this is called, previously a [`", stringify!($name), "`] must have been passed to [`mem::forget`].")]
            #[allow(dead_code, reason = "not all subsystems need this right now")]
            pub(crate) unsafe fn new_unchecked() -> Self {
                Self {
                    marker: PhantomData,
                }
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                let prev_count = $counter.fetch_add(1, Ordering::Relaxed);
                assert!(prev_count > 0);
                Self {
                    marker: PhantomData,
                }
            }
        }

        impl Drop for $name {
            #[inline]
            #[doc(alias = "SDL_QuitSubSystem")]
            fn drop(&mut self) {
                let prev_count = $counter.fetch_sub(1, Ordering::Relaxed);
                assert!(prev_count > 0);
                if prev_count == 1 {
                    unsafe {
                        sys::init::SDL_QuitSubSystem($flag);
                        // The last dropped subsystem instance "retrieves" an SdlDrop and drops it.
                        let _ = SdlDrop::new();
                    }
                }
            }
        }
    };
    ($name:ident, $flag:expr, $counter:ident, sync) => {
        subsystem!($name, $flag, $counter, nosync);
        unsafe impl Sync for $name {}
    };
}

subsystem!(AudioSubsystem, SDL_INIT_AUDIO, AUDIO_COUNT, nosync);
subsystem!(VideoSubsystem, SDL_INIT_VIDEO, VIDEO_COUNT, nosync);
subsystem!(JoystickSubsystem, SDL_INIT_JOYSTICK, JOYSTICK_COUNT, nosync);
subsystem!(HapticSubsystem, SDL_INIT_HAPTIC, HAPTIC_COUNT, nosync);
subsystem!(GamepadSubsystem, SDL_INIT_GAMEPAD, GAMEPAD_COUNT, nosync);
// The event queue can be read from other threads.
subsystem!(EventSubsystem, SDL_INIT_EVENTS, EVENT_COUNT, sync);
subsystem!(SensorSubsystem, SDL_INIT_SENSOR, SENSOR_COUNT, nosync);
subsystem!(CameraSubsystem, SDL_INIT_CAMERA, CAMERA_COUNT, nosync);

static IS_EVENT_PUMP_ALIVE: AtomicBool = AtomicBool::new(false);

/// A thread-safe type that encapsulates SDL event-pumping functions.
pub struct EventPump {
    _event_subsystem: EventSubsystem,
}

impl EventPump {
    /// Obtains the SDL event pump.
    #[inline]
    #[doc(alias = "SDL_InitSubSystem")]
    fn new(sdl: &Sdl) -> Result<EventPump, Error> {
        // Called on the main SDL thread.
        if IS_EVENT_PUMP_ALIVE.load(Ordering::Relaxed) {
            Err(Error("an `EventPump` instance is already alive - there can only be one `EventPump` in use at a time.".to_owned()))
        } else {
            let _event_subsystem = sdl.event()?;
            IS_EVENT_PUMP_ALIVE.store(true, Ordering::Relaxed);
            Ok(EventPump { _event_subsystem })
        }
    }
}

impl Drop for EventPump {
    #[inline]
    #[doc(alias = "SDL_QuitSubSystem")]
    fn drop(&mut self) {
        // Called on the main SDL thread.
        assert!(IS_EVENT_PUMP_ALIVE.load(Ordering::Relaxed));
        IS_EVENT_PUMP_ALIVE.store(false, Ordering::Relaxed);
    }
}

/// Get platform name
#[inline]
#[doc(alias = "SDL_GetPlatform")]
pub fn get_platform() -> &'static str {
    unsafe {
        CStr::from_ptr(sys::platform::SDL_GetPlatform())
            .to_str()
            .unwrap()
    }
}

/// Initializes the SDL library.
/// This must be called before using any other SDL function.
///
/// # Example
/// ```no_run
/// let sdl_context = sdl3::init().unwrap();
/// let mut event_pump = sdl_context.event_pump().unwrap();
///
/// for event in event_pump.poll_iter() {
///     // ...
/// }
///
/// // SDL_Quit() is called here as `sdl_context` is dropped.
/// ```
#[inline]
#[doc(alias = "SDL_GetError")]
pub fn init() -> Result<Sdl, Error> {
    Sdl::new()
}

pub fn get_error() -> Error {
    unsafe {
        let err = sys::error::SDL_GetError();
        Error(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
    }
}

#[doc(alias = "SDL_SetError")]
pub fn set_error(err: &str) -> Result<(), NulError> {
    let c_string = CString::new(err)?;
    unsafe {
        sys::error::SDL_SetError(
            c"%s".as_ptr() as *const c_char,
            c_string.as_ptr() as *const c_char,
        );
    }
    Ok(())
}

// #[doc(alias = "SDL_Error")]
// pub fn set_error_from_code(err: Error) {
//     unsafe {
//         sys::error::SDL_Error(transmute(err));
//     }
// }

#[doc(alias = "SDL_ClearError")]
pub fn clear_error() {
    unsafe {
        sys::error::SDL_ClearError();
    }
}
