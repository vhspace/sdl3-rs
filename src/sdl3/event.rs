/*!
Event Handling
 */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use std::mem::transmute;
use std::ptr;
use std::sync::Mutex;

use crate::gamepad;
use crate::gamepad::{Axis, Button};
use crate::get_error;
use crate::joystick;
use crate::joystick::HatState;
use crate::keyboard;
use crate::keyboard::Keycode;
use crate::keyboard::Mod;
use crate::keyboard::Scancode;
use crate::mouse;
use crate::mouse::{MouseButton, MouseState, MouseWheelDirection};
use crate::sys;
use crate::sys::events::SDL_EventFilter;
use crate::video::Orientation;
use libc::c_int;
use libc::c_void;
use sys::events::{
    SDL_GamepadAxisEvent, SDL_GamepadButtonEvent, SDL_GamepadDeviceEvent, SDL_JoyAxisEvent,
    SDL_JoyButtonEvent, SDL_JoyDeviceEvent, SDL_JoyHatEvent, SDL_KeyboardEvent,
    SDL_MouseButtonEvent, SDL_MouseMotionEvent, SDL_MouseWheelEvent,
};
use sys::everything::SDL_DisplayOrientation;
use sys::stdinc::Uint16;
use sys::video::SDL_DisplayID;

struct CustomEventTypeMaps {
    sdl_id_to_type_id: HashMap<u32, ::std::any::TypeId>,
    type_id_to_sdl_id: HashMap<::std::any::TypeId, u32>,
}

impl CustomEventTypeMaps {
    fn new() -> Self {
        CustomEventTypeMaps {
            sdl_id_to_type_id: HashMap::new(),
            type_id_to_sdl_id: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref CUSTOM_EVENT_TYPES: Mutex<CustomEventTypeMaps> =
        Mutex::new(CustomEventTypeMaps::new());
}

impl crate::EventSubsystem {
    /// Removes all events in the event queue that match the specified event type.
    #[doc(alias = "SDL_FlushEvent")]
    pub fn flush_event(&self, event_type: EventType) {
        unsafe { sys::events::SDL_FlushEvent(event_type.into()) };
    }

    /// Removes all events in the event queue that match the specified type range.
    #[doc(alias = "SDL_FlushEvents")]
    pub fn flush_events(&self, min_type: u32, max_type: u32) {
        unsafe { sys::events::SDL_FlushEvents(min_type, max_type) };
    }

    /// Reads the events at the front of the event queue, until the maximum amount
    /// of events is read.
    ///
    /// The events will _not_ be removed from the queue.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::event::Event;
    ///
    /// let sdl_context = sdl3::init().unwrap();
    /// let event_subsystem = sdl_context.event().unwrap();
    ///
    /// // Read up to 1024 events
    /// let events: Vec<Event> = event_subsystem.peek_events(1024);
    ///
    /// // Print each one
    /// for event in events {
    ///     println!("{:?}", event);
    /// }
    /// ```
    #[doc(alias = "SDL_PeepEvents")]
    pub fn peek_events<B>(&self, max_amount: u32) -> B
    where
        B: FromIterator<Event>,
    {
        unsafe {
            let mut events = Vec::with_capacity(max_amount as usize);

            let result = {
                let events_ptr = events.as_mut_ptr();

                sys::events::SDL_PeepEvents(
                    events_ptr,
                    max_amount as c_int,
                    sys::events::SDL_PEEKEVENT,
                    sys::events::SDL_EVENT_FIRST.into(),
                    sys::events::SDL_EVENT_LAST.into(),
                )
            };

            if result < 0 {
                // The only error possible is "Couldn't lock event queue"
                panic!("{}", get_error());
            } else {
                events.set_len(result as usize);

                events
                    .into_iter()
                    .map(|event_raw| Event::from_ll(event_raw))
                    .collect()
            }
        }
    }

    /// Pushes an event to the event queue.
    pub fn push_event(&self, event: Event) -> Result<(), String> {
        self.event_sender().push_event(event)
    }

    /// Register a custom SDL event.
    ///
    /// When pushing a user event, you must make sure that the ``type_`` field is set to a
    /// registered SDL event number.
    ///
    /// The ``code``, ``data1``,  and ``data2`` fields can be used to store user defined data.
    ///
    /// See the [SDL documentation](https://wiki.libsdl.org/SDL_UserEvent) for more information.
    ///
    /// # Example
    /// ```
    /// let sdl = sdl3::init().unwrap();
    /// let ev = sdl.event().unwrap();
    ///
    /// let custom_event_type_id = unsafe { ev.register_event().unwrap() };
    /// let event = sdl3::event::Event::User {
    ///    timestamp: 0,
    ///    window_id: 0,
    ///    type_: custom_event_type_id,
    ///    code: 456,
    ///    data1: 0x1234 as *mut libc::c_void,
    ///    data2: 0x5678 as *mut libc::c_void,
    /// };
    ///
    /// ev.push_event(event);
    ///
    /// ```
    #[inline(always)]
    pub unsafe fn register_event(&self) -> Result<u32, String> {
        Ok(*self.register_events(1)?.first().unwrap())
    }

    /// Registers custom SDL events.
    ///
    /// Returns an error, if no more user events can be created.
    pub unsafe fn register_events(&self, nr: u32) -> Result<Vec<u32>, String> {
        let result = sys::events::SDL_RegisterEvents(nr as ::libc::c_int);
        const ERR_NR: u32 = ::std::u32::MAX - 1;

        match result {
            ERR_NR => Err("No more user events can be created; SDL_EVENT_LAST reached".to_owned()),
            _ => {
                let event_ids = (result..(result + nr)).collect();
                Ok(event_ids)
            }
        }
    }

    /// Register a custom event
    ///
    /// It returns an error when the same type is registered twice.
    ///
    /// # Example
    /// See [push_custom_event](#method.push_custom_event)
    #[inline(always)]
    pub fn register_custom_event<T: ::std::any::Any>(&self) -> Result<(), String> {
        use std::any::TypeId;
        let event_id = *(unsafe { self.register_events(1) })?.first().unwrap();
        let mut cet = CUSTOM_EVENT_TYPES.lock().unwrap();
        let type_id = TypeId::of::<Box<T>>();

        if cet.type_id_to_sdl_id.contains_key(&type_id) {
            return Err("The same event type can not be registered twice!".to_owned());
        }

        cet.sdl_id_to_type_id.insert(event_id, type_id);
        cet.type_id_to_sdl_id.insert(type_id, event_id);

        Ok(())
    }

    /// Push a custom event
    ///
    /// If the event type ``T`` was not registered using
    /// [register_custom_event](#method.register_custom_event),
    /// this method will panic.
    ///
    /// # Example: pushing and receiving a custom event
    /// ```
    /// struct SomeCustomEvent {
    ///     a: i32
    /// }
    ///
    /// let sdl = sdl3::init().unwrap();
    /// let ev = sdl.event().unwrap();
    /// let mut ep = sdl.event_pump().unwrap();
    ///
    /// ev.register_custom_event::<SomeCustomEvent>().unwrap();
    ///
    /// let event = SomeCustomEvent { a: 42 };
    ///
    /// ev.push_custom_event(event);
    ///
    /// let received = ep.poll_event().unwrap(); // or within a for event in ep.poll_iter()
    /// if received.is_user_event() {
    ///     let e2 = received.as_user_event_type::<SomeCustomEvent>().unwrap();
    ///     assert_eq!(e2.a, 42);
    /// }
    /// ```
    pub fn push_custom_event<T: ::std::any::Any>(&self, event: T) -> Result<(), String> {
        self.event_sender().push_custom_event(event)
    }

    /// Create an event sender that can be sent to other threads.
    ///
    /// An `EventSender` will not keep the event subsystem alive. If the event subsystem is
    /// shut down calls to `push_event` and `push_custom_event` will return errors.
    pub fn event_sender(&self) -> EventSender {
        EventSender { _priv: () }
    }

    /// Create an event watcher which is called every time an event is added to event queue.
    ///
    /// The watcher is disabled when the return value is dropped.
    /// Just calling this function without binding to a variable immediately disables the watcher.
    /// In order to make it persistent, you have to bind in a variable and keep it until it's no
    /// longer needed.
    ///
    /// # Example: dump every event to stderr
    /// ```
    /// let sdl = sdl3::init().unwrap();
    /// let ev = sdl.event().unwrap();
    ///
    /// // `let _ = ...` is insufficient, as it is dropped immediately.
    /// let _event_watch = ev.add_event_watch(|event| {
    ///     dbg!(event);
    /// });
    /// ```
    pub fn add_event_watch<'a, CB: EventWatchCallback + 'a>(
        &self,
        callback: CB,
    ) -> EventWatch<'a, CB> {
        EventWatch::add(callback)
    }
}

/// Types of events that can be delivered.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum EventType {
    First = sys::events::SDL_EVENT_FIRST.0,

    Quit = sys::events::SDL_EVENT_QUIT.0,
    AppTerminating = sys::events::SDL_EVENT_TERMINATING.0,
    AppLowMemory = sys::events::SDL_EVENT_LOW_MEMORY.0,
    AppWillEnterBackground = sys::events::SDL_EVENT_WILL_ENTER_BACKGROUND.0,
    AppDidEnterBackground = sys::events::SDL_EVENT_DID_ENTER_BACKGROUND.0,
    AppWillEnterForeground = sys::events::SDL_EVENT_WILL_ENTER_FOREGROUND.0,
    AppDidEnterForeground = sys::events::SDL_EVENT_DID_ENTER_FOREGROUND.0,

    DisplayAdded = sys::events::SDL_EVENT_DISPLAY_ADDED.0,
    DisplayRemoved = sys::events::SDL_EVENT_DISPLAY_REMOVED.0,
    DisplayOrientation = sys::events::SDL_EVENT_DISPLAY_ORIENTATION.0,

    WindowShown = sys::events::SDL_EVENT_WINDOW_SHOWN.0,
    WindowHidden = sys::events::SDL_EVENT_WINDOW_HIDDEN.0,
    WindowExposed = sys::events::SDL_EVENT_WINDOW_EXPOSED.0,
    WindowMoved = sys::events::SDL_EVENT_WINDOW_MOVED.0,
    WindowResized = sys::events::SDL_EVENT_WINDOW_RESIZED.0,
    WindowPixelSizeChanged = sys::events::SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED.0,
    WindowMinimized = sys::events::SDL_EVENT_WINDOW_MINIMIZED.0,
    WindowMaximized = sys::events::SDL_EVENT_WINDOW_MAXIMIZED.0,
    WindowRestored = sys::events::SDL_EVENT_WINDOW_RESTORED.0,
    WindowMouseEnter = sys::events::SDL_EVENT_WINDOW_MOUSE_ENTER.0,
    WindowMouseLeave = sys::events::SDL_EVENT_WINDOW_MOUSE_LEAVE.0,
    WindowFocusGained = sys::events::SDL_EVENT_WINDOW_FOCUS_GAINED.0,
    WindowFocusLost = sys::events::SDL_EVENT_WINDOW_FOCUS_LOST.0,
    WindowCloseRequested = sys::events::SDL_EVENT_WINDOW_CLOSE_REQUESTED.0,
    WindowHitTest = sys::events::SDL_EVENT_WINDOW_HIT_TEST.0,
    WindowICCProfileChanged = sys::events::SDL_EVENT_WINDOW_ICCPROF_CHANGED.0,
    WindowDisplayChanged = sys::events::SDL_EVENT_WINDOW_DISPLAY_CHANGED.0,

    // TODO: SysWM = sys::events::SDL_EVENT_SYSWM .0,
    KeyDown = sys::events::SDL_EVENT_KEY_DOWN.0,
    KeyUp = sys::events::SDL_EVENT_KEY_UP.0,
    TextEditing = sys::events::SDL_EVENT_TEXT_EDITING.0,
    TextInput = sys::events::SDL_EVENT_TEXT_INPUT.0,

    MouseMotion = sys::events::SDL_EVENT_MOUSE_MOTION.0,
    MouseButtonDown = sys::events::SDL_EVENT_MOUSE_BUTTON_DOWN.0,
    MouseButtonUp = sys::events::SDL_EVENT_MOUSE_BUTTON_UP.0,
    MouseWheel = sys::events::SDL_EVENT_MOUSE_WHEEL.0,

    JoyAxisMotion = sys::events::SDL_EVENT_JOYSTICK_AXIS_MOTION.0,
    JoyHatMotion = sys::events::SDL_EVENT_JOYSTICK_HAT_MOTION.0,
    JoyButtonDown = sys::events::SDL_EVENT_JOYSTICK_BUTTON_DOWN.0,
    JoyButtonUp = sys::events::SDL_EVENT_JOYSTICK_BUTTON_UP.0,
    JoyDeviceAdded = sys::events::SDL_EVENT_JOYSTICK_ADDED.0,
    JoyDeviceRemoved = sys::events::SDL_EVENT_JOYSTICK_REMOVED.0,

    ControllerAxisMotion = sys::events::SDL_EVENT_GAMEPAD_AXIS_MOTION.0,
    ControllerButtonDown = sys::events::SDL_EVENT_GAMEPAD_BUTTON_DOWN.0,
    ControllerButtonUp = sys::events::SDL_EVENT_GAMEPAD_BUTTON_UP.0,
    ControllerDeviceAdded = sys::events::SDL_EVENT_GAMEPAD_ADDED.0,
    ControllerDeviceRemoved = sys::events::SDL_EVENT_GAMEPAD_REMOVED.0,
    ControllerDeviceRemapped = sys::events::SDL_EVENT_GAMEPAD_REMAPPED.0,
    ControllerTouchpadDown = sys::events::SDL_EVENT_GAMEPAD_TOUCHPAD_DOWN.0,
    ControllerTouchpadMotion = sys::events::SDL_EVENT_GAMEPAD_TOUCHPAD_MOTION.0,
    ControllerTouchpadUp = sys::events::SDL_EVENT_GAMEPAD_TOUCHPAD_UP.0,
    #[cfg(feature = "hidapi")]
    ControllerSensorUpdated = sys::events::SDL_EVENT_GAMEPAD_SENSOR_UPDATE.0,

    FingerDown = sys::events::SDL_EVENT_FINGER_DOWN.0,
    FingerUp = sys::events::SDL_EVENT_FINGER_UP.0,
    FingerMotion = sys::events::SDL_EVENT_FINGER_MOTION.0,
    // gestures have been removed from SD3: https://github.com/libsdl-org/SDL_gesture
    ClipboardUpdate = sys::events::SDL_EVENT_CLIPBOARD_UPDATE.0,
    DropFile = sys::events::SDL_EVENT_DROP_FILE.0,
    DropText = sys::events::SDL_EVENT_DROP_TEXT.0,
    DropBegin = sys::events::SDL_EVENT_DROP_BEGIN.0,
    DropComplete = sys::events::SDL_EVENT_DROP_COMPLETE.0,

    AudioDeviceAdded = sys::events::SDL_EVENT_AUDIO_DEVICE_ADDED.0,
    AudioDeviceRemoved = sys::events::SDL_EVENT_AUDIO_DEVICE_REMOVED.0,

    RenderTargetsReset = sys::events::SDL_EVENT_RENDER_TARGETS_RESET.0,
    RenderDeviceReset = sys::events::SDL_EVENT_RENDER_DEVICE_RESET.0,

    User = sys::events::SDL_EVENT_USER.0,
    Last = sys::events::SDL_EVENT_LAST.0,
}

impl From<EventType> for u32 {
    fn from(t: EventType) -> u32 {
        t as u32
    }
}

impl TryFrom<u32> for EventType {
    type Error = ();

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        use self::EventType::*;
        use crate::sys::events::*;

        Ok(match unsafe { transmute(n) } {
            SDL_EVENT_FIRST => First,

            SDL_EVENT_QUIT => Quit,
            SDL_EVENT_TERMINATING => AppTerminating,
            SDL_EVENT_LOW_MEMORY => AppLowMemory,
            SDL_EVENT_WILL_ENTER_BACKGROUND => AppWillEnterBackground,
            SDL_EVENT_DID_ENTER_BACKGROUND => AppDidEnterBackground,
            SDL_EVENT_WILL_ENTER_FOREGROUND => AppWillEnterForeground,
            SDL_EVENT_DID_ENTER_FOREGROUND => AppDidEnterForeground,

            SDL_EVENT_DISPLAY_ADDED => DisplayAdded,
            SDL_EVENT_DISPLAY_REMOVED => DisplayRemoved,
            SDL_EVENT_DISPLAY_ORIENTATION => DisplayOrientation,

            SDL_EVENT_WINDOW_SHOWN => WindowShown,
            SDL_EVENT_WINDOW_HIDDEN => WindowHidden,
            SDL_EVENT_WINDOW_EXPOSED => WindowExposed,
            SDL_EVENT_WINDOW_MOVED => WindowMoved,
            SDL_EVENT_WINDOW_RESIZED => WindowResized,
            SDL_EVENT_WINDOW_MINIMIZED => WindowMinimized,
            SDL_EVENT_WINDOW_MAXIMIZED => WindowMaximized,
            SDL_EVENT_WINDOW_RESTORED => WindowRestored,
            SDL_EVENT_WINDOW_MOUSE_ENTER => WindowMouseEnter,
            SDL_EVENT_WINDOW_MOUSE_LEAVE => WindowMouseLeave,
            SDL_EVENT_WINDOW_FOCUS_GAINED => WindowFocusGained,
            SDL_EVENT_WINDOW_FOCUS_LOST => WindowFocusLost,
            SDL_EVENT_WINDOW_CLOSE_REQUESTED => WindowCloseRequested,

            SDL_EVENT_KEY_DOWN => KeyDown,
            SDL_EVENT_KEY_UP => KeyUp,
            SDL_EVENT_TEXT_EDITING => TextEditing,
            SDL_EVENT_TEXT_INPUT => TextInput,

            SDL_EVENT_MOUSE_MOTION => MouseMotion,
            SDL_EVENT_MOUSE_BUTTON_DOWN => MouseButtonDown,
            SDL_EVENT_MOUSE_BUTTON_UP => MouseButtonUp,
            SDL_EVENT_MOUSE_WHEEL => MouseWheel,

            SDL_EVENT_JOYSTICK_AXIS_MOTION => JoyAxisMotion,
            SDL_EVENT_JOYSTICK_HAT_MOTION => JoyHatMotion,
            SDL_EVENT_JOYSTICK_BUTTON_DOWN => JoyButtonDown,
            SDL_EVENT_JOYSTICK_BUTTON_UP => JoyButtonUp,
            SDL_EVENT_JOYSTICK_ADDED => JoyDeviceAdded,
            SDL_EVENT_JOYSTICK_REMOVED => JoyDeviceRemoved,

            SDL_EVENT_GAMEPAD_AXIS_MOTION => ControllerAxisMotion,
            SDL_EVENT_GAMEPAD_BUTTON_DOWN => ControllerButtonDown,
            SDL_EVENT_GAMEPAD_BUTTON_UP => ControllerButtonUp,
            SDL_EVENT_GAMEPAD_ADDED => ControllerDeviceAdded,
            SDL_EVENT_GAMEPAD_REMOVED => ControllerDeviceRemoved,
            SDL_EVENT_GAMEPAD_REMAPPED => ControllerDeviceRemapped,
            SDL_EVENT_GAMEPAD_TOUCHPAD_DOWN => ControllerTouchpadDown,
            SDL_EVENT_GAMEPAD_TOUCHPAD_MOTION => ControllerTouchpadMotion,
            SDL_EVENT_GAMEPAD_TOUCHPAD_UP => ControllerTouchpadUp,
            #[cfg(feature = "hidapi")]
            SDL_EVENT_GAMEPAD_SENSOR_UPDATE => ControllerSensorUpdated,

            SDL_EVENT_FINGER_DOWN => FingerDown,
            SDL_EVENT_FINGER_UP => FingerUp,
            SDL_EVENT_FINGER_MOTION => FingerMotion,

            SDL_EVENT_CLIPBOARD_UPDATE => ClipboardUpdate,
            SDL_EVENT_DROP_FILE => DropFile,
            SDL_EVENT_DROP_TEXT => DropText,
            SDL_EVENT_DROP_BEGIN => DropBegin,
            SDL_EVENT_DROP_COMPLETE => DropComplete,

            SDL_EVENT_AUDIO_DEVICE_ADDED => AudioDeviceAdded,
            SDL_EVENT_AUDIO_DEVICE_REMOVED => AudioDeviceRemoved,

            SDL_EVENT_RENDER_TARGETS_RESET => RenderTargetsReset,
            SDL_EVENT_RENDER_DEVICE_RESET => RenderDeviceReset,

            SDL_EVENT_USER => User,
            SDL_EVENT_LAST => Last,

            _ => return Err(()),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
/// An enum of display events.
pub enum DisplayEvent {
    None,
    Orientation(Orientation),
    Added,
    Removed,
}

impl DisplayEvent {
    #[allow(clippy::match_same_arms)]
    fn from_ll(id: u32, data1: i32) -> DisplayEvent {
        match unsafe { transmute(id) } {
            sys::events::SDL_EVENT_DISPLAY_ORIENTATION => {
                let orientation = if data1 > SDL_DisplayOrientation::PORTRAIT_FLIPPED.0 {
                    Orientation::Unknown
                } else {
                    let sdl_orientation = SDL_DisplayOrientation(data1);
                    Orientation::from_ll(sdl_orientation)
                };
                DisplayEvent::Orientation(orientation)
            }
            sys::events::SDL_EVENT_DISPLAY_ADDED => DisplayEvent::Added,
            sys::events::SDL_EVENT_DISPLAY_REMOVED => DisplayEvent::Removed,
            _ => DisplayEvent::None,
        }
    }

    fn to_ll(&self) -> (u32, i32) {
        match *self {
            DisplayEvent::Orientation(orientation) => (
                sys::events::SDL_EVENT_DISPLAY_ORIENTATION.into(),
                orientation.to_ll().into(),
            ),
            DisplayEvent::Added => (sys::events::SDL_EVENT_DISPLAY_ADDED.into(), 0),
            DisplayEvent::Removed => (sys::events::SDL_EVENT_DISPLAY_REMOVED.into(), 0),
            DisplayEvent::None => panic!("DisplayEvent::None cannot be converted"),
        }
    }

    pub fn is_same_kind_as(&self, other: &DisplayEvent) -> bool {
        match (self, other) {
            (Self::None, Self::None)
            | (Self::Orientation(_), Self::Orientation(_))
            | (Self::Added, Self::Added)
            | (Self::Removed, Self::Removed) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
/// An enum of window events.
pub enum WindowEvent {
    None,
    Shown,
    Hidden,
    Exposed,
    Moved(i32, i32),
    Resized(i32, i32),
    PixelSizeChanged(i32, i32),
    Minimized,
    Maximized,
    Restored,
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    CloseRequested,
    HitTest(i32, i32),
    ICCProfChanged,
    DisplayChanged(i32),
}

impl WindowEvent {
    #[allow(clippy::match_same_arms)]
    fn from_ll(id: u32, data1: i32, data2: i32) -> WindowEvent {
        match EventType::try_from(id) {
            Ok(ev) => match ev {
                EventType::WindowShown => WindowEvent::Shown,
                EventType::WindowHidden => WindowEvent::Hidden,
                EventType::WindowExposed => WindowEvent::Exposed,
                EventType::WindowMoved => WindowEvent::Moved(data1, data2),
                EventType::WindowResized => WindowEvent::Resized(data1, data2),
                EventType::WindowPixelSizeChanged => WindowEvent::PixelSizeChanged(data1, data2),
                EventType::WindowMinimized => WindowEvent::Minimized,
                EventType::WindowMaximized => WindowEvent::Maximized,
                EventType::WindowRestored => WindowEvent::Restored,
                EventType::WindowMouseEnter => WindowEvent::MouseEnter,
                EventType::WindowMouseLeave => WindowEvent::MouseLeave,
                EventType::WindowFocusGained => WindowEvent::FocusGained,
                EventType::WindowFocusLost => WindowEvent::FocusLost,
                EventType::WindowCloseRequested => WindowEvent::CloseRequested,
                EventType::WindowHitTest => WindowEvent::HitTest(data1, data2),
                EventType::WindowICCProfileChanged => WindowEvent::ICCProfChanged,
                EventType::WindowDisplayChanged => WindowEvent::DisplayChanged(data1),
                _ => WindowEvent::None,
            },
            Err(_) => WindowEvent::None,
        }
    }

    fn to_ll(&self) -> (EventType, i32, i32) {
        match *self {
            WindowEvent::None => panic!("Cannot convert WindowEvent::None"),
            WindowEvent::Shown => (EventType::WindowShown, 0, 0),
            WindowEvent::Hidden => (EventType::WindowHidden, 0, 0),
            WindowEvent::Exposed => (EventType::WindowExposed, 0, 0),
            WindowEvent::Moved(d1, d2) => (EventType::WindowMoved, d1, d2),
            WindowEvent::Resized(d1, d2) => (EventType::WindowResized, d1, d2),
            WindowEvent::PixelSizeChanged(d1, d2) => (EventType::WindowPixelSizeChanged, d1, d2),
            WindowEvent::Minimized => (EventType::WindowMinimized, 0, 0),
            WindowEvent::Maximized => (EventType::WindowMaximized, 0, 0),
            WindowEvent::Restored => (EventType::WindowRestored, 0, 0),
            WindowEvent::MouseEnter => (EventType::WindowMouseEnter, 0, 0),
            WindowEvent::MouseLeave => (EventType::WindowMouseLeave, 0, 0),
            WindowEvent::FocusGained => (EventType::WindowFocusGained, 0, 0),
            WindowEvent::FocusLost => (EventType::WindowFocusLost, 0, 0),
            WindowEvent::CloseRequested => (EventType::WindowCloseRequested, 0, 0),
            WindowEvent::HitTest(d1, d2) => (EventType::WindowHitTest, d1, d2),
            WindowEvent::ICCProfChanged => (EventType::WindowICCProfileChanged, 0, 0),
            WindowEvent::DisplayChanged(d1) => (EventType::WindowDisplayChanged, d1, 0),
        }
    }

    pub fn is_same_kind_as(&self, other: &WindowEvent) -> bool {
        match (self, other) {
            (Self::None, Self::None)
            | (Self::Shown, Self::Shown)
            | (Self::Hidden, Self::Hidden)
            | (Self::Exposed, Self::Exposed)
            | (Self::Moved(_, _), Self::Moved(_, _))
            | (Self::Resized(_, _), Self::Resized(_, _))
            | (Self::PixelSizeChanged(_, _), Self::PixelSizeChanged(_, _))
            | (Self::Minimized, Self::Minimized)
            | (Self::Maximized, Self::Maximized)
            | (Self::Restored, Self::Restored)
            | (Self::MouseEnter, Self::MouseEnter)
            | (Self::MouseLeave, Self::MouseLeave)
            | (Self::FocusGained, Self::FocusGained)
            | (Self::FocusLost, Self::FocusLost)
            | (Self::CloseRequested, Self::CloseRequested)
            | (Self::HitTest(_, _), Self::HitTest(_, _))
            | (Self::ICCProfChanged, Self::ICCProfChanged)
            | (Self::DisplayChanged(_), Self::DisplayChanged(_)) => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Different event types.
pub enum Event {
    Quit {
        timestamp: u64,
    },
    AppTerminating {
        timestamp: u64,
    },
    AppLowMemory {
        timestamp: u64,
    },
    AppWillEnterBackground {
        timestamp: u64,
    },
    AppDidEnterBackground {
        timestamp: u64,
    },
    AppWillEnterForeground {
        timestamp: u64,
    },
    AppDidEnterForeground {
        timestamp: u64,
    },

    Window {
        timestamp: u64,
        window_id: u32,
        win_event: WindowEvent,
    },

    // TODO: SysWMEvent
    KeyDown {
        timestamp: u64,
        window_id: u32,
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
        keymod: Mod,
        repeat: bool,
        which: u32,
        raw: Uint16,
    },
    KeyUp {
        timestamp: u64,
        window_id: u32,
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
        keymod: Mod,
        repeat: bool,
        which: u32,
        raw: Uint16,
    },

    TextEditing {
        timestamp: u64,
        window_id: u32,
        text: String,
        start: i32,
        length: i32,
    },

    TextInput {
        timestamp: u64,
        window_id: u32,
        text: String,
    },

    MouseMotion {
        timestamp: u64,
        window_id: u32,
        which: u32,
        mousestate: MouseState,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    },

    MouseButtonDown {
        timestamp: u64,
        window_id: u32,
        which: u32,
        mouse_btn: MouseButton,
        clicks: u8,
        x: f32,
        y: f32,
    },
    MouseButtonUp {
        timestamp: u64,
        window_id: u32,
        which: u32,
        mouse_btn: MouseButton,
        clicks: u8,
        x: f32,
        y: f32,
    },

    MouseWheel {
        timestamp: u64,
        window_id: u32,
        which: u32,
        x: f32,
        y: f32,
        direction: MouseWheelDirection,
        mouse_x: f32,
        mouse_y: f32,
    },

    JoyAxisMotion {
        timestamp: u64,
        /// The joystick's `id`
        which: u32,
        axis_idx: u8,
        value: i16,
    },

    JoyHatMotion {
        timestamp: u64,
        /// The joystick's `id`
        which: u32,
        hat_idx: u8,
        state: HatState,
    },

    JoyButtonDown {
        timestamp: u64,
        /// The joystick's `id`
        which: u32,
        button_idx: u8,
    },
    JoyButtonUp {
        timestamp: u64,
        /// The joystick's `id`
        which: u32,
        button_idx: u8,
    },

    JoyDeviceAdded {
        timestamp: u64,
        /// The newly added joystick's `joystick_index`
        which: u32,
    },
    JoyDeviceRemoved {
        timestamp: u64,
        /// The joystick's `id`
        which: u32,
    },

    ControllerAxisMotion {
        timestamp: u64,
        /// The controller's joystick `id`
        which: u32,
        axis: Axis,
        value: i16,
    },

    ControllerButtonDown {
        timestamp: u64,
        /// The controller's joystick `id`
        which: u32,
        button: Button,
    },
    ControllerButtonUp {
        timestamp: u64,
        /// The controller's joystick `id`
        which: u32,
        button: Button,
    },

    ControllerDeviceAdded {
        timestamp: u64,
        /// The newly added controller's `joystick_index`
        which: u32,
    },
    ControllerDeviceRemoved {
        timestamp: u64,
        /// The controller's joystick `id`
        which: u32,
    },
    ControllerDeviceRemapped {
        timestamp: u64,
        /// The controller's joystick `id`
        which: u32,
    },

    ControllerTouchpadDown {
        timestamp: u64,
        /// The joystick instance id
        which: u32,
        /// The index of the touchpad
        touchpad: i32,
        /// The index of the finger on the touchpad
        finger: i32,
        /// Normalized in the range 0...1 with 0 being on the left
        x: f32,
        /// Normalized in the range 0...1 with 0 being at the top
        y: f32,
        /// Normalized in the range 0...1
        pressure: f32,
    },
    ControllerTouchpadMotion {
        timestamp: u64,
        /// The joystick instance id
        which: u32,
        /// The index of the touchpad
        touchpad: i32,
        /// The index of the finger on the touchpad
        finger: i32,
        /// Normalized in the range 0...1 with 0 being on the left
        x: f32,
        /// Normalized in the range 0...1 with 0 being at the top
        y: f32,
        /// Normalized in the range 0...1
        pressure: f32,
    },
    ControllerTouchpadUp {
        timestamp: u64,
        /// The joystick instance id
        which: u32,
        /// The index of the touchpad
        touchpad: i32,
        /// The index of the finger on the touchpad
        finger: i32,
        /// Normalized in the range 0...1 with 0 being on the left
        x: f32,
        /// Normalized in the range 0...1 with 0 being at the top
        y: f32,
        /// Normalized in the range 0...1
        pressure: f32,
    },

    /// Triggered when the gyroscope or accelerometer is updated
    #[cfg(feature = "hidapi")]
    ControllerSensorUpdated {
        timestamp: u64,
        which: u32,
        sensor: crate::sensor::SensorType,
        /// Data from the sensor.
        ///
        /// See the `sensor` module for more information.
        data: [f32; 3],
    },

    FingerDown {
        timestamp: u64,
        touch_id: u64,
        finger_id: u64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    FingerUp {
        timestamp: u64,
        touch_id: u64,
        finger_id: u64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    FingerMotion {
        timestamp: u64,
        touch_id: u64,
        finger_id: u64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },

    DollarRecord {
        timestamp: u64,
        touch_id: i64,
        gesture_id: i64,
        num_fingers: u32,
        error: f32,
        x: f32,
        y: f32,
    },

    MultiGesture {
        timestamp: u64,
        touch_id: i64,
        d_theta: f32,
        d_dist: f32,
        x: f32,
        y: f32,
        num_fingers: u16,
    },

    ClipboardUpdate {
        timestamp: u64,
    },

    DropFile {
        timestamp: u64,
        window_id: u32,
        filename: String,
    },
    DropText {
        timestamp: u64,
        window_id: u32,
        filename: String,
    },
    DropBegin {
        timestamp: u64,
        window_id: u32,
    },
    DropComplete {
        timestamp: u64,
        window_id: u32,
    },

    AudioDeviceAdded {
        timestamp: u64,
        which: u32,
        iscapture: bool,
    },
    AudioDeviceRemoved {
        timestamp: u64,
        which: u32,
        iscapture: bool,
    },

    RenderTargetsReset {
        timestamp: u64,
    },
    RenderDeviceReset {
        timestamp: u64,
    },

    User {
        timestamp: u64,
        window_id: u32,
        type_: u32,
        code: i32,
        data1: *mut c_void,
        data2: *mut c_void,
    },

    Unknown {
        timestamp: u64,
        type_: u32,
    },

    Display {
        timestamp: u64,
        display_index: SDL_DisplayID,
        display_event: DisplayEvent,
    },
}

/// This does not auto-derive because `User`'s `data` fields can be used to
/// store pointers to types that are `!Send`. Dereferencing these as pointers
/// requires using `unsafe` and ensuring your own safety guarantees.
unsafe impl Send for Event {}

/// This does not auto-derive because `User`'s `data` fields can be used to
/// store pointers to types that are `!Sync`. Dereferencing these as pointers
/// requires using `unsafe` and ensuring your own safety guarantees.
unsafe impl Sync for Event {}

// TODO: Remove this when from_utf8 is updated in Rust
// This would honestly be nice if it took &self instead of self,
// but Event::User's raw pointers kind of removes that possibility.
impl Event {
    fn to_ll(&self) -> Option<sys::events::SDL_Event> {
        let mut ret = mem::MaybeUninit::uninit();
        match *self {
            Event::User {
                window_id,
                type_,
                code,
                data1,
                data2,
                timestamp,
            } => {
                let event = sys::events::SDL_UserEvent {
                    r#type: sys::events::SDL_EVENT_USER.into(),
                    timestamp,
                    windowID: window_id,
                    code: code as i32,
                    data1,
                    data2,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_UserEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::Quit { timestamp } => {
                let event = sys::events::SDL_QuitEvent {
                    r#type: sys::events::SDL_EVENT_QUIT.into(),
                    timestamp,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_QuitEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::Window {
                timestamp,
                window_id,
                win_event,
            } => {
                let (win_event_id, data1, data2) = win_event.to_ll();
                let event = WindowEvent {
                    type_: win_event_id.into(),
                    timestamp,
                    windowID: window_id,
                    data1,
                    data2,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_WindowEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::KeyDown {
                timestamp,
                window_id,
                keycode,
                scancode,
                keymod,
                repeat,
                which,
                raw,
            } => {
                let event = SDL_KeyboardEvent {
                    r#type: sys::events::SDL_EVENT_KEY_DOWN.into(),
                    timestamp,
                    windowID: window_id,
                    repeat,
                    reserved: 0,
                    scancode: scancode?.into(),
                    which,
                    down: true,
                    key: keycode?.into(),
                    r#mod: keymod.bits(),
                    raw,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_KeyboardEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::KeyUp {
                timestamp,
                window_id,
                keycode,
                scancode,
                keymod,
                repeat,
                which,
                raw,
            } => {
                let event = SDL_KeyboardEvent {
                    r#type: sys::events::SDL_EVENT_KEY_UP.into(),
                    timestamp,
                    windowID: window_id,
                    repeat,
                    reserved: 0,
                    scancode: scancode?.into(),
                    which,
                    down: false,
                    key: keycode?.into(),
                    r#mod: keymod.bits(),
                    raw,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_KeyboardEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                let state = mousestate.to_sdl_state();
                let event = SDL_MouseMotionEvent {
                    r#type: sys::events::SDL_EVENT_MOUSE_MOTION.into(),
                    timestamp,
                    windowID: window_id,
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_MouseMotionEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::MouseButtonDown {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                let event = SDL_MouseButtonEvent {
                    r#type: sys::events::SDL_EVENT_MOUSE_BUTTON_DOWN.into(),
                    timestamp,
                    windowID: window_id,
                    which,
                    button: mouse_btn as u8,
                    down: true,
                    clicks,
                    x,
                    y,
                    padding: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_MouseButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::MouseButtonUp {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                let event = sys::events::SDL_MouseButtonEvent {
                    r#type: sys::events::SDL_EVENT_MOUSE_BUTTON_UP.into(),
                    timestamp,
                    windowID: window_id,
                    which,
                    button: mouse_btn as u8,
                    clicks,
                    padding: 0,
                    x,
                    y,
                    down: false,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_MouseButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::MouseWheel {
                timestamp,
                window_id,
                which,
                x,
                y,
                direction,
                mouse_x,
                mouse_y,
            } => {
                let event = SDL_MouseWheelEvent {
                    r#type: sys::events::SDL_EVENT_MOUSE_WHEEL.into(),
                    timestamp,
                    windowID: window_id,
                    which,
                    x,
                    y,
                    direction: direction.into(),
                    mouse_x: mouse_x,
                    mouse_y: mouse_y,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_MouseWheelEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::JoyAxisMotion {
                timestamp,
                which,
                axis_idx,
                value,
            } => {
                let event = SDL_JoyAxisEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_AXIS_MOTION.into(),
                    timestamp,
                    which,
                    axis: axis_idx,
                    value,
                    padding1: 0,
                    padding2: 0,
                    padding3: 0,
                    padding4: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyAxisEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::JoyHatMotion {
                timestamp,
                which,
                hat_idx,
                state,
            } => {
                let hatvalue = state.to_raw();
                let event = SDL_JoyHatEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_HAT_MOTION.into(),
                    timestamp,
                    which,
                    hat: hat_idx,
                    value: hatvalue,
                    padding1: 0,
                    padding2: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyHatEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::JoyButtonDown {
                timestamp,
                which,
                button_idx,
            } => {
                let event = SDL_JoyButtonEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_BUTTON_DOWN.into(),
                    timestamp,
                    which,
                    button: button_idx,
                    down: true,
                    padding1: 0,
                    padding2: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::JoyButtonUp {
                timestamp,
                which,
                button_idx,
            } => {
                let event = SDL_JoyButtonEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_BUTTON_UP.into(),
                    timestamp,
                    which,
                    button: button_idx,
                    down: false,
                    padding1: 0,
                    padding2: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::JoyDeviceAdded { timestamp, which } => {
                let event = SDL_JoyDeviceEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_ADDED.into(),
                    timestamp,
                    which,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyDeviceEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::JoyDeviceRemoved { timestamp, which } => {
                let event = SDL_JoyDeviceEvent {
                    r#type: sys::events::SDL_EVENT_JOYSTICK_REMOVED.into(),
                    timestamp,
                    which,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_JoyDeviceEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::ControllerAxisMotion {
                timestamp,
                which,
                axis,
                value,
            } => {
                let event = SDL_GamepadAxisEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_AXIS_MOTION.into(),
                    timestamp,
                    which,
                    axis: axis.into(),
                    value,
                    padding1: 0,
                    padding2: 0,
                    padding3: 0,
                    padding4: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadAxisEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }
            Event::ControllerButtonDown {
                timestamp,
                which,
                button,
            } => {
                let event = SDL_GamepadButtonEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_BUTTON_DOWN.into(),
                    timestamp,
                    which,
                    // This conversion turns an i32 into a u8; signed-to-unsigned conversions
                    // are a bit of a code smell, but that appears to be how SDL defines it.
                    button: button.into(),
                    down: true,
                    padding1: 0,
                    padding2: 0,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::ControllerButtonUp {
                timestamp,
                which,
                button,
            } => {
                let event = SDL_GamepadButtonEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_BUTTON_UP.into(),
                    reserved: 0,
                    timestamp,
                    which,
                    button: button.into(),
                    down: false,
                    padding1: 0,
                    padding2: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadButtonEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::ControllerDeviceAdded { timestamp, which } => {
                let event = SDL_GamepadDeviceEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_ADDED.into(),
                    timestamp,
                    which,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadDeviceEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::ControllerDeviceRemoved { timestamp, which } => {
                let event = SDL_GamepadDeviceEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_REMOVED.into(),
                    timestamp,
                    which,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadDeviceEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::ControllerDeviceRemapped { timestamp, which } => {
                let event = SDL_GamepadDeviceEvent {
                    r#type: sys::events::SDL_EVENT_GAMEPAD_REMAPPED.into(),
                    timestamp,
                    which,
                    reserved: 0,
                };
                unsafe {
                    ptr::copy(
                        &event,
                        ret.as_mut_ptr() as *mut sys::events::SDL_GamepadDeviceEvent,
                        1,
                    );
                    Some(ret.assume_init())
                }
            }

            Event::FingerDown { .. }
            | Event::FingerUp { .. }
            | Event::FingerMotion { .. }
            | Event::DollarRecord { .. }
            | Event::MultiGesture { .. }
            | Event::ClipboardUpdate { .. }
            | Event::DropFile { .. }
            | Event::TextEditing { .. }
            | Event::TextInput { .. }
            | Event::Unknown { .. }
            | _ => {
                // don't know how to convert!
                None
            }
        }
    }

    pub fn from_ll(raw: sys::events::SDL_Event) -> Event {
        let raw_type = unsafe { raw.r#type };

        // if event type has not been defined, treat it as a UserEvent
        let event_type: EventType = EventType::try_from(raw_type).unwrap_or(EventType::User);
        unsafe {
            match event_type {
                EventType::WindowShown
                | EventType::WindowHidden
                | EventType::WindowExposed
                | EventType::WindowMoved
                | EventType::WindowResized
                | EventType::WindowPixelSizeChanged
                | EventType::WindowMinimized
                | EventType::WindowMaximized
                | EventType::WindowRestored
                | EventType::WindowMouseEnter
                | EventType::WindowMouseLeave
                | EventType::WindowFocusGained
                | EventType::WindowFocusLost
                | EventType::WindowCloseRequested
                | EventType::WindowHitTest
                | EventType::WindowICCProfileChanged
                | EventType::WindowDisplayChanged => {
                    let event = raw.window;
                    Event::Window {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        win_event: WindowEvent::from_ll(
                            event.r#type.into(),
                            event.data1,
                            event.data2,
                        ),
                    }
                }

                EventType::Quit => {
                    let event = raw.quit;
                    Event::Quit {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppTerminating => {
                    let event = raw.common;
                    Event::AppTerminating {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppLowMemory => {
                    let event = raw.common;
                    Event::AppLowMemory {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppWillEnterBackground => {
                    let event = raw.common;
                    Event::AppWillEnterBackground {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppDidEnterBackground => {
                    let event = raw.common;
                    Event::AppDidEnterBackground {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppWillEnterForeground => {
                    let event = raw.common;
                    Event::AppWillEnterForeground {
                        timestamp: event.timestamp,
                    }
                }
                EventType::AppDidEnterForeground => {
                    let event = raw.common;
                    Event::AppDidEnterForeground {
                        timestamp: event.timestamp,
                    }
                }

                EventType::DisplayOrientation
                | EventType::DisplayAdded
                | EventType::DisplayRemoved => {
                    let event = raw.display;

                    Event::Display {
                        timestamp: event.timestamp,
                        display_index: event.displayID,
                        display_event: DisplayEvent::from_ll(event.r#type.into(), event.data1),
                    }
                }

                // TODO: SysWMEventType
                EventType::KeyDown => {
                    let event = raw.key;

                    Event::KeyDown {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        keycode: Keycode::from_i32(event.key as i32),
                        scancode: Scancode::from_i32(event.scancode.into()),
                        keymod: keyboard::Mod::from_bits_truncate(event.r#mod),
                        repeat: event.repeat,
                        which: event.which,
                        raw: event.raw,
                    }
                }
                EventType::KeyUp => {
                    let event = raw.key;

                    Event::KeyUp {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        keycode: Keycode::from_i32(event.key as i32),
                        scancode: Scancode::from_i32(event.scancode.into()),
                        keymod: keyboard::Mod::from_bits_truncate(event.r#mod),
                        repeat: event.repeat,
                        which: event.which,
                        raw: event.raw,
                    }
                }
                EventType::TextEditing => {
                    let event = raw.edit;

                    // event.text is a *const c_char (pointer to a C string)
                    let c_str: &CStr = unsafe { CStr::from_ptr(event.text) };

                    // Convert the CStr to a Rust &str
                    let text_str = c_str.to_str().expect("Invalid UTF-8 string");
                    let text = text_str.to_owned();

                    Event::TextEditing {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        text,
                        start: event.start,
                        length: event.length,
                    }
                }
                EventType::TextInput => {
                    let event = raw.text;

                    // event.text is a *const c_char (pointer to a C string)
                    let c_str: &CStr = unsafe { CStr::from_ptr(event.text) };

                    // Convert the CStr to a Rust &str
                    let text_str = c_str.to_str().expect("Invalid UTF-8 string");
                    let text = text_str.to_owned();

                    Event::TextInput {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        text,
                    }
                }

                EventType::MouseMotion => {
                    let event = raw.motion;

                    Event::MouseMotion {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        which: event.which.into(),
                        mousestate: mouse::MouseState::from_sdl_state(event.state),
                        x: event.x,
                        y: event.y,
                        xrel: event.xrel,
                        yrel: event.yrel,
                    }
                }
                EventType::MouseButtonDown => {
                    let event = raw.button;

                    Event::MouseButtonDown {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        which: event.which.into(),
                        mouse_btn: mouse::MouseButton::from_ll(event.button),
                        clicks: event.clicks,
                        x: event.x,
                        y: event.y,
                    }
                }
                EventType::MouseButtonUp => {
                    let event = raw.button;

                    Event::MouseButtonUp {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        which: event.which.into(),
                        mouse_btn: mouse::MouseButton::from_ll(event.button),
                        clicks: event.clicks,
                        x: event.x,
                        y: event.y,
                    }
                }
                EventType::MouseWheel => {
                    let event = raw.wheel;

                    Event::MouseWheel {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        which: event.which.into(),
                        x: event.x,
                        y: event.y,
                        direction: event.direction.into(),
                        mouse_x: event.mouse_x,
                        mouse_y: event.mouse_y,
                    }
                }

                EventType::JoyAxisMotion => {
                    let event = raw.jaxis;
                    Event::JoyAxisMotion {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        axis_idx: event.axis,
                        value: event.value,
                    }
                }
                EventType::JoyHatMotion => {
                    let event = raw.jhat;
                    Event::JoyHatMotion {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        hat_idx: event.hat,
                        state: joystick::HatState::from_raw(event.value),
                    }
                }
                EventType::JoyButtonDown => {
                    let event = raw.jbutton;
                    Event::JoyButtonDown {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        button_idx: event.button,
                    }
                }
                EventType::JoyButtonUp => {
                    let event = raw.jbutton;
                    Event::JoyButtonUp {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        button_idx: event.button,
                    }
                }
                EventType::JoyDeviceAdded => {
                    let event = raw.jdevice;
                    Event::JoyDeviceAdded {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                    }
                }
                EventType::JoyDeviceRemoved => {
                    let event = raw.jdevice;
                    Event::JoyDeviceRemoved {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                    }
                }

                EventType::ControllerAxisMotion => {
                    let event = raw.gaxis;
                    let axis = gamepad::Axis::from_ll(transmute(event.axis as i32)).unwrap();

                    Event::ControllerAxisMotion {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        axis,
                        value: event.value,
                    }
                }
                EventType::ControllerButtonDown => {
                    let event = raw.gbutton;
                    let button = gamepad::Button::from_ll(transmute(event.button as i32)).unwrap();

                    Event::ControllerButtonDown {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        button,
                    }
                }
                EventType::ControllerButtonUp => {
                    let event = raw.gbutton;
                    let button = gamepad::Button::from_ll(transmute(event.button as i32)).unwrap();

                    Event::ControllerButtonUp {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        button,
                    }
                }
                EventType::ControllerDeviceAdded => {
                    let event = raw.gdevice;
                    Event::ControllerDeviceAdded {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                    }
                }
                EventType::ControllerDeviceRemoved => {
                    let event = raw.gdevice;
                    Event::ControllerDeviceRemoved {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                    }
                }
                EventType::ControllerDeviceRemapped => {
                    let event = raw.gdevice;
                    Event::ControllerDeviceRemapped {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                    }
                }
                EventType::ControllerTouchpadDown => {
                    let event = raw.gtouchpad;
                    Event::ControllerTouchpadDown {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        touchpad: event.touchpad.into(),
                        finger: event.finger.into(),
                        x: event.x,
                        y: event.y,
                        pressure: event.pressure,
                    }
                }
                EventType::ControllerTouchpadMotion => {
                    let event = raw.gtouchpad;
                    Event::ControllerTouchpadMotion {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        touchpad: event.touchpad.into(),
                        finger: event.finger.into(),
                        x: event.x,
                        y: event.y,
                        pressure: event.pressure,
                    }
                }
                EventType::ControllerTouchpadUp => {
                    let event = raw.gtouchpad;
                    Event::ControllerTouchpadUp {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        touchpad: event.touchpad.into(),
                        finger: event.finger.into(),
                        x: event.x,
                        y: event.y,
                        pressure: event.pressure,
                    }
                }
                #[cfg(feature = "hidapi")]
                EventType::ControllerSensorUpdated => {
                    let event = raw.gsensor;
                    Event::ControllerSensorUpdated {
                        timestamp: event.timestamp,
                        which: event.which.into(),
                        sensor: crate::sensor::SensorType::from_ll(event.sensor),
                        data: event.data,
                    }
                }

                EventType::FingerDown => {
                    let event = raw.tfinger;
                    Event::FingerDown {
                        timestamp: event.timestamp,
                        touch_id: event.touchID,
                        finger_id: event.fingerID,
                        x: event.x,
                        y: event.y,
                        dx: event.dx,
                        dy: event.dy,
                        pressure: event.pressure,
                    }
                }
                EventType::FingerUp => {
                    let event = raw.tfinger;
                    Event::FingerUp {
                        timestamp: event.timestamp,
                        touch_id: event.touchID,
                        finger_id: event.fingerID,
                        x: event.x,
                        y: event.y,
                        dx: event.dx,
                        dy: event.dy,
                        pressure: event.pressure,
                    }
                }
                EventType::FingerMotion => {
                    let event = raw.tfinger;
                    Event::FingerMotion {
                        timestamp: event.timestamp,
                        touch_id: event.touchID,
                        finger_id: event.fingerID,
                        x: event.x,
                        y: event.y,
                        dx: event.dx,
                        dy: event.dy,
                        pressure: event.pressure,
                    }
                }

                EventType::ClipboardUpdate => {
                    let event = raw.common;
                    Event::ClipboardUpdate {
                        timestamp: event.timestamp,
                    }
                }
                EventType::DropFile => {
                    let event = raw.drop;

                    let buf = CStr::from_ptr(event.data as *const _).to_bytes();
                    let text = String::from_utf8_lossy(buf).to_string();

                    Event::DropFile {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        filename: text,
                    }
                }
                EventType::DropText => {
                    let event = raw.drop;

                    let buf = CStr::from_ptr(event.data as *const _).to_bytes();
                    let text = String::from_utf8_lossy(buf).to_string();

                    Event::DropText {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                        filename: text,
                    }
                }
                EventType::DropBegin => {
                    let event = raw.drop;

                    Event::DropBegin {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                    }
                }
                EventType::DropComplete => {
                    let event = raw.drop;

                    Event::DropComplete {
                        timestamp: event.timestamp,
                        window_id: event.windowID,
                    }
                }
                EventType::AudioDeviceAdded => {
                    let event = raw.adevice;
                    Event::AudioDeviceAdded {
                        timestamp: event.timestamp,
                        which: event.which,
                        // false if an audio output device, true if an audio capture device
                        iscapture: event.recording,
                    }
                }
                EventType::AudioDeviceRemoved => {
                    let event = raw.adevice;
                    Event::AudioDeviceRemoved {
                        timestamp: event.timestamp,
                        which: event.which,
                        // false if an audio output device, true if an audio capture device
                        iscapture: event.recording,
                    }
                }

                EventType::RenderTargetsReset => Event::RenderTargetsReset {
                    timestamp: raw.common.timestamp,
                },
                EventType::RenderDeviceReset => Event::RenderDeviceReset {
                    timestamp: raw.common.timestamp,
                },

                EventType::First => panic!("Unused event, EventType::First, was encountered"),
                EventType::Last => panic!("Unusable event, EventType::Last, was encountered"),

                // If we have no other match and the event type is >= 32768
                // this is a user event
                EventType::User => {
                    if raw_type < 32_768 {
                        // The type is unknown to us.
                        // It's a newer sdl3 type.
                        let event = raw.common;

                        Event::Unknown {
                            timestamp: event.timestamp,
                            type_: event.r#type,
                        }
                    } else {
                        let event = raw.user;

                        Event::User {
                            timestamp: event.timestamp,
                            window_id: event.windowID,
                            type_: raw_type,
                            code: event.code,
                            data1: event.data1,
                            data2: event.data2,
                        }
                    }
                }
            }
        } // close unsafe & match
    }

    pub fn is_user_event(&self) -> bool {
        match *self {
            Event::User { .. } => true,
            _ => false,
        }
    }

    pub fn as_user_event_type<T: ::std::any::Any>(&self) -> Option<T> {
        use std::any::TypeId;
        let type_id = TypeId::of::<Box<T>>();

        let (event_id, event_box_ptr) = match *self {
            Event::User { type_, data1, .. } => (type_, data1),
            _ => return None,
        };

        let cet = CUSTOM_EVENT_TYPES.lock().unwrap();

        let event_type_id = match cet.sdl_id_to_type_id.get(&event_id) {
            Some(id) => id,
            None => {
                panic!("internal error; could not find typeid")
            }
        };

        if &type_id != event_type_id {
            return None;
        }

        let event_box: Box<T> = unsafe { Box::from_raw(event_box_ptr as *mut T) };

        Some(*event_box)
    }

    /// Returns `true` if they are the same "kind" of events.
    ///
    /// # Example:
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev1 = Event::JoyButtonDown {
    ///     timestamp: 0,
    ///     which: 0,
    ///     button_idx: 0,
    /// };
    /// let ev2 = Event::JoyButtonDown {
    ///     timestamp: 1,
    ///     which: 1,
    ///     button_idx: 1,
    /// };
    ///
    /// assert!(ev1 != ev2); // The events aren't equal (they contain different values).
    /// assert!(ev1.is_same_kind_as(&ev2)); // But they are of the same kind!
    /// ```
    pub fn is_same_kind_as(&self, other: &Event) -> bool {
        match (self, other) {
            (Self::Quit { .. }, Self::Quit { .. })
            | (Self::AppTerminating { .. }, Self::AppTerminating { .. })
            | (Self::AppLowMemory { .. }, Self::AppLowMemory { .. })
            | (Self::AppWillEnterBackground { .. }, Self::AppWillEnterBackground { .. })
            | (Self::AppDidEnterBackground { .. }, Self::AppDidEnterBackground { .. })
            | (Self::AppWillEnterForeground { .. }, Self::AppWillEnterForeground { .. })
            | (Self::AppDidEnterForeground { .. }, Self::AppDidEnterForeground { .. })
            | (Self::Display { .. }, Self::Display { .. })
            | (Self::Window { .. }, Self::Window { .. })
            | (Self::KeyDown { .. }, Self::KeyDown { .. })
            | (Self::KeyUp { .. }, Self::KeyUp { .. })
            | (Self::TextEditing { .. }, Self::TextEditing { .. })
            | (Self::TextInput { .. }, Self::TextInput { .. })
            | (Self::MouseMotion { .. }, Self::MouseMotion { .. })
            | (Self::MouseButtonDown { .. }, Self::MouseButtonDown { .. })
            | (Self::MouseButtonUp { .. }, Self::MouseButtonUp { .. })
            | (Self::MouseWheel { .. }, Self::MouseWheel { .. })
            | (Self::JoyAxisMotion { .. }, Self::JoyAxisMotion { .. })
            | (Self::JoyHatMotion { .. }, Self::JoyHatMotion { .. })
            | (Self::JoyButtonDown { .. }, Self::JoyButtonDown { .. })
            | (Self::JoyButtonUp { .. }, Self::JoyButtonUp { .. })
            | (Self::JoyDeviceAdded { .. }, Self::JoyDeviceAdded { .. })
            | (Self::JoyDeviceRemoved { .. }, Self::JoyDeviceRemoved { .. })
            | (Self::ControllerAxisMotion { .. }, Self::ControllerAxisMotion { .. })
            | (Self::ControllerButtonDown { .. }, Self::ControllerButtonDown { .. })
            | (Self::ControllerButtonUp { .. }, Self::ControllerButtonUp { .. })
            | (Self::ControllerDeviceAdded { .. }, Self::ControllerDeviceAdded { .. })
            | (Self::ControllerDeviceRemoved { .. }, Self::ControllerDeviceRemoved { .. })
            | (Self::ControllerDeviceRemapped { .. }, Self::ControllerDeviceRemapped { .. })
            | (Self::FingerDown { .. }, Self::FingerDown { .. })
            | (Self::FingerUp { .. }, Self::FingerUp { .. })
            | (Self::FingerMotion { .. }, Self::FingerMotion { .. })
            | (Self::DollarRecord { .. }, Self::DollarRecord { .. })
            | (Self::MultiGesture { .. }, Self::MultiGesture { .. })
            | (Self::ClipboardUpdate { .. }, Self::ClipboardUpdate { .. })
            | (Self::DropFile { .. }, Self::DropFile { .. })
            | (Self::DropText { .. }, Self::DropText { .. })
            | (Self::DropBegin { .. }, Self::DropBegin { .. })
            | (Self::DropComplete { .. }, Self::DropComplete { .. })
            | (Self::AudioDeviceAdded { .. }, Self::AudioDeviceAdded { .. })
            | (Self::AudioDeviceRemoved { .. }, Self::AudioDeviceRemoved { .. })
            | (Self::RenderTargetsReset { .. }, Self::RenderTargetsReset { .. })
            | (Self::RenderDeviceReset { .. }, Self::RenderDeviceReset { .. })
            | (Self::User { .. }, Self::User { .. })
            | (Self::Unknown { .. }, Self::Unknown { .. }) => true,
            #[cfg(feature = "hidapi")]
            (Self::ControllerSensorUpdated { .. }, Self::ControllerSensorUpdated { .. }) => true,
            _ => false,
        }
    }

    /// Returns the `timestamp` field of the event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::JoyButtonDown {
    ///     timestamp: 12,
    ///     which: 0,
    ///     button_idx: 0,
    /// };
    /// assert!(ev.get_timestamp() == 12);
    /// ```
    pub fn get_timestamp(&self) -> u64 {
        *match self {
            Self::Quit { timestamp, .. } => timestamp,
            Self::Window { timestamp, .. } => timestamp,
            Self::AppTerminating { timestamp, .. } => timestamp,
            Self::AppLowMemory { timestamp, .. } => timestamp,
            Self::AppWillEnterBackground { timestamp, .. } => timestamp,
            Self::AppDidEnterBackground { timestamp, .. } => timestamp,
            Self::AppWillEnterForeground { timestamp, .. } => timestamp,
            Self::AppDidEnterForeground { timestamp, .. } => timestamp,
            Self::Display { timestamp, .. } => timestamp,
            Self::KeyDown { timestamp, .. } => timestamp,
            Self::KeyUp { timestamp, .. } => timestamp,
            Self::TextEditing { timestamp, .. } => timestamp,
            Self::TextInput { timestamp, .. } => timestamp,
            Self::MouseMotion { timestamp, .. } => timestamp,
            Self::MouseButtonDown { timestamp, .. } => timestamp,
            Self::MouseButtonUp { timestamp, .. } => timestamp,
            Self::MouseWheel { timestamp, .. } => timestamp,
            Self::JoyAxisMotion { timestamp, .. } => timestamp,
            Self::JoyHatMotion { timestamp, .. } => timestamp,
            Self::JoyButtonDown { timestamp, .. } => timestamp,
            Self::JoyButtonUp { timestamp, .. } => timestamp,
            Self::JoyDeviceAdded { timestamp, .. } => timestamp,
            Self::JoyDeviceRemoved { timestamp, .. } => timestamp,
            Self::ControllerAxisMotion { timestamp, .. } => timestamp,
            Self::ControllerButtonDown { timestamp, .. } => timestamp,
            Self::ControllerButtonUp { timestamp, .. } => timestamp,
            Self::ControllerDeviceAdded { timestamp, .. } => timestamp,
            Self::ControllerDeviceRemoved { timestamp, .. } => timestamp,
            Self::ControllerDeviceRemapped { timestamp, .. } => timestamp,
            Self::ControllerTouchpadDown { timestamp, .. } => timestamp,
            Self::ControllerTouchpadMotion { timestamp, .. } => timestamp,
            Self::ControllerTouchpadUp { timestamp, .. } => timestamp,
            #[cfg(feature = "hidapi")]
            Self::ControllerSensorUpdated { timestamp, .. } => timestamp,
            Self::FingerDown { timestamp, .. } => timestamp,
            Self::FingerUp { timestamp, .. } => timestamp,
            Self::FingerMotion { timestamp, .. } => timestamp,
            Self::DollarRecord { timestamp, .. } => timestamp,
            Self::MultiGesture { timestamp, .. } => timestamp,
            Self::ClipboardUpdate { timestamp, .. } => timestamp,
            Self::DropFile { timestamp, .. } => timestamp,
            Self::DropText { timestamp, .. } => timestamp,
            Self::DropBegin { timestamp, .. } => timestamp,
            Self::DropComplete { timestamp, .. } => timestamp,
            Self::AudioDeviceAdded { timestamp, .. } => timestamp,
            Self::AudioDeviceRemoved { timestamp, .. } => timestamp,
            Self::RenderTargetsReset { timestamp, .. } => timestamp,
            Self::RenderDeviceReset { timestamp, .. } => timestamp,
            Self::User { timestamp, .. } => timestamp,
            Self::Unknown { timestamp, .. } => timestamp,
        }
    }

    /// Returns the `window_id` field of the event if it's present (not all events have it!).
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::JoyButtonDown {
    ///     timestamp: 0,
    ///     which: 0,
    ///     button_idx: 0,
    /// };
    /// assert!(ev.get_window_id() == None);
    ///
    /// let another_ev = Event::DropBegin {
    ///     timestamp: 0,
    ///     window_id: 3,
    /// };
    /// assert!(another_ev.get_window_id() == Some(3));
    /// ```
    pub fn get_window_id(&self) -> Option<u32> {
        match self {
            Self::Window { window_id, .. } => Some(*window_id),
            Self::KeyDown { window_id, .. } => Some(*window_id),
            Self::KeyUp { window_id, .. } => Some(*window_id),
            Self::TextEditing { window_id, .. } => Some(*window_id),
            Self::TextInput { window_id, .. } => Some(*window_id),
            Self::MouseMotion { window_id, .. } => Some(*window_id),
            Self::MouseButtonDown { window_id, .. } => Some(*window_id),
            Self::MouseButtonUp { window_id, .. } => Some(*window_id),
            Self::MouseWheel { window_id, .. } => Some(*window_id),
            Self::DropFile { window_id, .. } => Some(*window_id),
            Self::DropText { window_id, .. } => Some(*window_id),
            Self::DropBegin { window_id, .. } => Some(*window_id),
            Self::DropComplete { window_id, .. } => Some(*window_id),
            Self::User { window_id, .. } => Some(*window_id),
            _ => None,
        }
    }

    /// Returns `true` if this is a window event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(ev.is_window());
    ///
    /// let ev = Event::AppLowMemory {
    ///     timestamp: 0,
    /// };
    /// assert!(ev.is_window());
    ///
    /// let another_ev = Event::TextInput {
    ///     timestamp: 0,
    ///     window_id: 0,
    ///     text: String::new(),
    /// };
    /// assert!(another_ev.is_window() == false); // Not a window event!
    /// ```
    pub fn is_window(&self) -> bool {
        match self {
            Self::Quit { .. }
            | Self::AppTerminating { .. }
            | Self::AppLowMemory { .. }
            | Self::AppWillEnterBackground { .. }
            | Self::AppDidEnterBackground { .. }
            | Self::AppWillEnterForeground { .. }
            | Self::AppDidEnterForeground { .. }
            | Self::Window { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a keyboard event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    /// use sdl3::keyboard::Mod;
    ///
    /// let ev = Event::KeyDown {
    ///     timestamp: 0,
    ///     window_id: 0,
    ///     keycode: None,
    ///     scancode: None,
    ///     keymod: Mod::empty(),
    ///     repeat: false,
    ///     which: 0,
    ///     raw: 0,
    /// };
    /// assert!(ev.is_keyboard());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_keyboard() == false); // Not a keyboard event!
    /// ```
    pub fn is_keyboard(&self) -> bool {
        match self {
            Self::KeyDown { .. } | Self::KeyUp { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a text event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::TextInput {
    ///     timestamp: 0,
    ///     window_id: 0,
    ///     text: String::new(),
    /// };
    /// assert!(ev.is_text());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_text() == false); // Not a text event!
    /// ```
    pub fn is_text(&self) -> bool {
        match self {
            Self::TextEditing { .. } | Self::TextInput { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a mouse event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    /// use sdl3::mouse::MouseWheelDirection;
    ///
    /// let ev = Event::MouseWheel {
    ///     timestamp: 0,
    ///     window_id: 0,
    ///     which: 0,
    ///     mouse_x: 0.0,
    ///     mouse_y: 0.0,
    ///     x: 0.0,
    ///     y: 0.0,
    ///     direction: MouseWheelDirection::Normal,
    /// };
    /// assert!(ev.is_mouse());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_mouse() == false); // Not a mouse event!
    /// ```
    pub fn is_mouse(&self) -> bool {
        match self {
            Self::MouseMotion { .. }
            | Self::MouseButtonDown { .. }
            | Self::MouseButtonUp { .. }
            | Self::MouseWheel { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a controller event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::ControllerDeviceAdded {
    ///     timestamp: 0,
    ///     which: 0,
    /// };
    /// assert!(ev.is_controller());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_controller() == false); // Not a controller event!
    /// ```
    pub fn is_controller(&self) -> bool {
        match self {
            Self::ControllerAxisMotion { .. }
            | Self::ControllerButtonDown { .. }
            | Self::ControllerButtonUp { .. }
            | Self::ControllerDeviceAdded { .. }
            | Self::ControllerDeviceRemoved { .. }
            | Self::ControllerDeviceRemapped { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a joy event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::JoyButtonUp {
    ///     timestamp: 0,
    ///     which: 0,
    ///     button_idx: 0,
    /// };
    /// assert!(ev.is_joy());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_joy() == false); // Not a joy event!
    /// ```
    pub fn is_joy(&self) -> bool {
        match self {
            Self::JoyAxisMotion { .. }
            | Self::JoyHatMotion { .. }
            | Self::JoyButtonDown { .. }
            | Self::JoyButtonUp { .. }
            | Self::JoyDeviceAdded { .. }
            | Self::JoyDeviceRemoved { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a finger event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::FingerMotion {
    ///     timestamp: 0,
    ///     touch_id: 0,
    ///     finger_id: 0,
    ///     x: 0.,
    ///     y: 0.,
    ///     dx: 0.,
    ///     dy: 0.,
    ///     pressure: 0.,
    /// };
    /// assert!(ev.is_finger());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_finger() == false); // Not a finger event!
    /// ```
    pub fn is_finger(&self) -> bool {
        match self {
            Self::FingerDown { .. } | Self::FingerUp { .. } | Self::FingerMotion { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a drop event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::DropBegin {
    ///     timestamp: 0,
    ///     window_id: 3,
    /// };
    /// assert!(ev.is_drop());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_drop() == false); // Not a drop event!
    /// ```
    pub fn is_drop(&self) -> bool {
        match self {
            Self::DropFile { .. }
            | Self::DropText { .. }
            | Self::DropBegin { .. }
            | Self::DropComplete { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is an audio event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::AudioDeviceAdded {
    ///     timestamp: 0,
    ///     which: 3,
    ///     iscapture: false,
    /// };
    /// assert!(ev.is_audio());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_audio() == false); // Not an audio event!
    /// ```
    pub fn is_audio(&self) -> bool {
        match self {
            Self::AudioDeviceAdded { .. } | Self::AudioDeviceRemoved { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a render event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::RenderTargetsReset {
    ///     timestamp: 0,
    /// };
    /// assert!(ev.is_render());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_render() == false); // Not a render event!
    /// ```
    pub fn is_render(&self) -> bool {
        match self {
            Self::RenderTargetsReset { .. } | Self::RenderDeviceReset { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a user event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::User {
    ///     timestamp: 0,
    ///     window_id: 0,
    ///     type_: 0,
    ///     code: 0,
    ///     data1: ::std::ptr::null_mut(),
    ///     data2: ::std::ptr::null_mut(),
    /// };
    /// assert!(ev.is_user());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_user() == false); // Not a user event!
    /// ```
    pub fn is_user(&self) -> bool {
        match self {
            Self::User { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if this is an unknown event.
    ///
    /// # Example
    ///
    /// ```
    /// use sdl3::event::Event;
    ///
    /// let ev = Event::Unknown {
    ///     timestamp: 0,
    ///     type_: 0,
    /// };
    /// assert!(ev.is_unknown());
    ///
    /// let another_ev = Event::Quit {
    ///     timestamp: 0,
    /// };
    /// assert!(another_ev.is_unknown() == false); // Not an unknown event!
    /// ```
    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown { .. } => true,
            _ => false,
        }
    }
}

unsafe fn poll_event() -> Option<Event> {
    let mut raw = mem::MaybeUninit::uninit();
    let has_pending = sys::events::SDL_PollEvent(raw.as_mut_ptr());

    if has_pending {
        Some(Event::from_ll(raw.assume_init()))
    } else {
        None
    }
}

unsafe fn wait_event() -> Event {
    let mut raw = mem::MaybeUninit::uninit();
    let success = sys::events::SDL_WaitEvent(raw.as_mut_ptr());

    if success {
        Event::from_ll(raw.assume_init())
    } else {
        panic!("{}", get_error())
    }
}

unsafe fn wait_event_timeout(timeout: u32) -> Option<Event> {
    let mut raw = mem::MaybeUninit::uninit();
    let success = sys::events::SDL_WaitEventTimeout(raw.as_mut_ptr(), timeout as c_int);

    if success {
        Some(Event::from_ll(raw.assume_init()))
    } else {
        None
    }
}

impl crate::EventPump {
    /// Polls for currently pending events.
    ///
    /// If no events are pending, `None` is returned.
    pub fn poll_event(&mut self) -> Option<Event> {
        unsafe { poll_event() }
    }

    /// Returns a polling iterator that calls `poll_event()`.
    /// The iterator will terminate once there are no more pending events.
    ///
    /// # Example
    /// ```no_run
    /// let sdl_context = sdl3::init().unwrap();
    /// let mut event_pump = sdl_context.event_pump().unwrap();
    ///
    /// for event in event_pump.poll_iter() {
    ///     use sdl3::event::Event;
    ///     match event {
    ///         Event::KeyDown {..} => { /*...*/ }
    ///         _ => ()
    ///     }
    /// }
    /// ```
    pub fn poll_iter(&mut self) -> EventPollIterator {
        EventPollIterator {
            _marker: PhantomData,
        }
    }

    /// Pumps the event loop, gathering events from the input devices.
    #[doc(alias = "SDL_PumpEvents")]
    pub fn pump_events(&mut self) {
        unsafe {
            sys::events::SDL_PumpEvents();
        };
    }

    /// Waits indefinitely for the next available event.
    pub fn wait_event(&mut self) -> Event {
        unsafe { wait_event() }
    }

    /// Waits until the specified timeout (in milliseconds) for the next available event.
    pub fn wait_event_timeout(&mut self, timeout: u32) -> Option<Event> {
        unsafe { wait_event_timeout(timeout) }
    }

    /// Returns a waiting iterator that calls `wait_event()`.
    ///
    /// Note: The iterator will never terminate.
    pub fn wait_iter(&mut self) -> EventWaitIterator {
        EventWaitIterator {
            _marker: PhantomData,
        }
    }

    /// Returns a waiting iterator that calls `wait_event_timeout()`.
    ///
    /// Note: The iterator will never terminate, unless waiting for an event
    /// exceeds the specified timeout.
    pub fn wait_timeout_iter(&mut self, timeout: u32) -> EventWaitTimeoutIterator {
        EventWaitTimeoutIterator {
            _marker: PhantomData,
            timeout,
        }
    }

    #[inline]
    pub fn keyboard_state(&self) -> crate::keyboard::KeyboardState {
        crate::keyboard::KeyboardState::new(self)
    }

    #[inline]
    pub fn mouse_state(&self) -> crate::mouse::MouseState {
        crate::mouse::MouseState::new(self)
    }

    #[inline]
    pub fn relative_mouse_state(&self) -> crate::mouse::RelativeMouseState {
        crate::mouse::RelativeMouseState::new(self)
    }
}

/// An iterator that calls `EventPump::poll_event()`.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct EventPollIterator<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for EventPollIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        unsafe { poll_event() }
    }
}

/// An iterator that calls `EventPump::wait_event()`.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct EventWaitIterator<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for EventWaitIterator<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        unsafe { Some(wait_event()) }
    }
}

/// An iterator that calls `EventPump::wait_event_timeout()`.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct EventWaitTimeoutIterator<'a> {
    _marker: PhantomData<&'a ()>,
    timeout: u32,
}

impl<'a> Iterator for EventWaitTimeoutIterator<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        unsafe { wait_event_timeout(self.timeout) }
    }
}

#[cfg(test)]
mod test {
    use super::super::gamepad::{Axis, Button};
    use super::super::joystick::HatState;
    use super::super::keyboard::{Keycode, Mod, Scancode};
    use super::super::mouse::{MouseButton, MouseState, MouseWheelDirection};
    use super::super::video::Orientation;
    use super::DisplayEvent;
    use super::Event;
    use super::WindowEvent;

    // Tests a round-trip conversion from an Event type to
    // the SDL event type and back, to make sure it's sane.
    #[test]
    fn test_to_from_ll() {
        {
            let e = Event::Quit { timestamp: 0 };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::Display {
                timestamp: 0,
                display_index: 1,
                display_event: DisplayEvent::Orientation(Orientation::LandscapeFlipped),
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::Window {
                timestamp: 0,
                window_id: 0,
                win_event: WindowEvent::Resized(1, 2),
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::KeyDown {
                timestamp: 0,
                window_id: 1,
                keycode: None,
                scancode: Some(Scancode::Q),
                keymod: Mod::all(),
                repeat: false,
                which: 0,
                raw: 0,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::KeyUp {
                timestamp: 123,
                window_id: 0,
                keycode: Some(Keycode::R),
                scancode: Some(Scancode::R),
                keymod: Mod::empty(),
                repeat: true,
                which: 0,
                raw: 0,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::MouseMotion {
                timestamp: 0,
                window_id: 0,
                which: 1,
                mousestate: MouseState::from_sdl_state(1),
                x: 3.,
                y: 91.,
                xrel: -1.,
                yrel: 43.,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::MouseButtonDown {
                timestamp: 5634,
                window_id: 2,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: 543.,
                y: 345.,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::MouseButtonUp {
                timestamp: 0,
                window_id: 2,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: 543.,
                y: 345.,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::MouseWheel {
                timestamp: 1,
                window_id: 0,
                which: 32,
                x: 23.,
                y: 91.,
                direction: MouseWheelDirection::Flipped,
                mouse_x: 2.,
                mouse_y: 3.,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyAxisMotion {
                timestamp: 0,
                which: 1,
                axis_idx: 1,
                value: 12,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyHatMotion {
                timestamp: 0,
                which: 3,
                hat_idx: 1,
                state: HatState::Left,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyButtonDown {
                timestamp: 0,
                which: 0,
                button_idx: 3,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyButtonUp {
                timestamp: 9876,
                which: 1,
                button_idx: 2,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyDeviceAdded {
                timestamp: 0,
                which: 1,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::JoyDeviceRemoved {
                timestamp: 0,
                which: 2,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerAxisMotion {
                timestamp: 53,
                which: 0,
                axis: Axis::LeftX,
                value: 3,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerButtonDown {
                timestamp: 0,
                which: 1,
                button: Button::Guide,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerButtonUp {
                timestamp: 654214,
                which: 0,
                button: Button::DPadRight,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerDeviceAdded {
                timestamp: 543,
                which: 3,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerDeviceRemoved {
                timestamp: 555,
                which: 3,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
        {
            let e = Event::ControllerDeviceRemapped {
                timestamp: 654,
                which: 0,
            };
            let e2 = Event::from_ll(e.clone().to_ll().unwrap());
            assert_eq!(e, e2);
        }
    }

    #[test]
    fn test_from_ll_keymod_keydown_unknown_bits() {
        let mut raw_event = Event::KeyDown {
            timestamp: 0,
            window_id: 1,
            keycode: None,
            scancode: Some(Scancode::Q),
            keymod: Mod::empty(),
            repeat: false,
            which: 0,
            raw: 0,
        }
        .to_ll()
        .unwrap();

        // Simulate SDL setting bits unknown to us, see PR #780
        unsafe {
            raw_event.key.r#mod = 0xffff;
        }

        if let Event::KeyDown { keymod, .. } = Event::from_ll(raw_event) {
            assert_eq!(keymod, Mod::all());
        } else {
            panic!()
        }
    }

    #[test]
    fn test_from_ll_keymod_keyup_unknown_bits() {
        let mut raw_event = Event::KeyUp {
            timestamp: 0,
            window_id: 1,
            keycode: None,
            scancode: Some(Scancode::Q),
            keymod: Mod::empty(),
            repeat: false,
            which: 0,
            raw: 0,
        }
        .to_ll()
        .unwrap();

        // Simulate SDL setting bits unknown to us, see PR #780
        unsafe {
            raw_event.key.r#mod = 0xffff;
        }

        if let Event::KeyUp { keymod, .. } = Event::from_ll(raw_event) {
            assert_eq!(keymod, Mod::all());
        } else {
            panic!()
        }
    }
}

/// A sendible type that can push events to the event queue.
pub struct EventSender {
    _priv: (),
}

impl EventSender {
    /// Pushes an event to the event queue.
    #[doc(alias = "SDL_PushEvent")]
    pub fn push_event(&self, event: Event) -> Result<(), String> {
        match event.to_ll() {
            Some(mut raw_event) => {
                let ok = unsafe { sys::events::SDL_PushEvent(&mut raw_event) };
                if ok {
                    Ok(())
                } else {
                    Err(get_error())
                }
            }
            None => Err("Cannot push unsupported event type to the queue".to_owned()),
        }
    }

    /// Push a custom event
    ///
    /// If the event type ``T`` was not registered using
    /// [EventSubsystem::register_custom_event]
    /// (../struct.EventSubsystem.html#method.register_custom_event),
    /// this method will panic.
    ///
    /// # Example: pushing and receiving a custom event
    /// ```
    /// struct SomeCustomEvent {
    ///     a: i32
    /// }
    ///
    /// let sdl = sdl3::init().unwrap();
    /// let ev = sdl.event().unwrap();
    /// let mut ep = sdl.event_pump().unwrap();
    ///
    /// ev.register_custom_event::<SomeCustomEvent>().unwrap();
    ///
    /// let event = SomeCustomEvent { a: 42 };
    ///
    /// ev.push_custom_event(event);
    ///
    /// let received = ep.poll_event().unwrap(); // or within a for event in ep.poll_iter()
    /// if received.is_user_event() {
    ///     let e2 = received.as_user_event_type::<SomeCustomEvent>().unwrap();
    ///     assert_eq!(e2.a, 42);
    /// }
    /// ```
    pub fn push_custom_event<T: ::std::any::Any>(&self, event: T) -> Result<(), String> {
        use std::any::TypeId;
        let cet = CUSTOM_EVENT_TYPES.lock().unwrap();
        let type_id = TypeId::of::<Box<T>>();

        let user_event_id = *match cet.type_id_to_sdl_id.get(&type_id) {
            Some(id) => id,
            None => {
                return Err("Type is not registered as a custom event type!".to_owned());
            }
        };

        let event_box = Box::new(event);
        let event = Event::User {
            timestamp: 0,
            window_id: 0,
            type_: user_event_id,
            code: 0,
            data1: Box::into_raw(event_box) as *mut c_void,
            data2: ::std::ptr::null_mut(),
        };
        drop(cet);

        self.push_event(event)?;

        Ok(())
    }
}

/// A callback trait for [`EventSubsystem::add_event_watch`].
pub trait EventWatchCallback {
    fn callback(&mut self, event: Event) -> ();
}

/// An handler for the event watch callback.
/// One must bind this struct in a variable as long as you want to keep the callback active.
/// For further information, see [`EventSubsystem::add_event_watch`].
pub struct EventWatch<'a, CB: EventWatchCallback + 'a> {
    activated: bool,
    callback: Box<CB>,
    _phantom: PhantomData<&'a CB>,
}

impl<'a, CB: EventWatchCallback + 'a> EventWatch<'a, CB> {
    fn add(callback: CB) -> EventWatch<'a, CB> {
        let f = Box::new(callback);
        let mut watch = EventWatch {
            activated: false,
            callback: f,
            _phantom: PhantomData,
        };
        watch.activate();
        watch
    }

    /// Activates the event watch.
    /// Does nothing if it is already activated.
    pub fn activate(&mut self) {
        if !self.activated {
            self.activated = true;
            unsafe { sys::events::SDL_AddEventWatch(self.filter(), self.callback()) };
        }
    }

    /// Deactivates the event watch.
    /// Does nothing if it is already activated.
    pub fn deactivate(&mut self) {
        if self.activated {
            self.activated = false;
            unsafe { sys::events::SDL_RemoveEventWatch(self.filter(), self.callback()) };
        }
    }

    /// Returns if the event watch is activated.
    pub fn activated(&self) -> bool {
        self.activated
    }

    /// Set the activation state of the event watch.
    pub fn set_activated(&mut self, activate: bool) {
        if activate {
            self.activate();
        } else {
            self.deactivate();
        }
    }

    fn filter(&self) -> SDL_EventFilter {
        Some(event_callback_marshall::<CB>)
    }

    fn callback(&mut self) -> *mut c_void {
        &mut *self.callback as *mut _ as *mut c_void
    }
}

impl<'a, CB: EventWatchCallback + 'a> Drop for EventWatch<'a, CB> {
    fn drop(&mut self) {
        self.deactivate();
    }
}

extern "C" fn event_callback_marshall<CB: EventWatchCallback>(
    user_data: *mut c_void,
    event: *mut sdl3_sys::events::SDL_Event,
) -> bool {
    let f: &mut CB = unsafe { &mut *(user_data as *mut _) };
    let event = Event::from_ll(unsafe { *event });
    f.callback(event);
    false
}

impl<F: FnMut(Event) -> ()> EventWatchCallback for F {
    fn callback(&mut self, event: Event) -> () {
        self(event)
    }
}
