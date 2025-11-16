/*!
 * Event Handling
 */

use std::any::TypeId;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem;
use std::mem::transmute;
use std::ptr;
use std::sync::Mutex;

use libc::{c_int, c_void};

use crate::get_error;
use crate::Error;

use crate::gamepad::{Axis as GamepadAxis, Button as GamepadButton};
use crate::joystick::HatState;
use crate::keyboard::{Keycode, Mod, Scancode};
use crate::mouse::{MouseButton, MouseState, MouseWheelDirection};
use crate::pen::PenAxis;
use crate::video::{Display, Orientation};

use crate::sys;
use crate::sys::events::{
    SDL_AudioDeviceEvent, SDL_CommonEvent, SDL_DisplayEvent, SDL_Event, SDL_EventType,
    SDL_GamepadAxisEvent, SDL_GamepadButtonEvent, SDL_GamepadDeviceEvent, SDL_GamepadTouchpadEvent,
    SDL_JoyAxisEvent, SDL_JoyButtonEvent, SDL_JoyDeviceEvent, SDL_JoyHatEvent, SDL_KeyboardEvent,
    SDL_MouseButtonEvent, SDL_MouseMotionEvent, SDL_MouseWheelEvent, SDL_PenAxisEvent,
    SDL_PenButtonEvent, SDL_PenMotionEvent, SDL_PenProximityEvent, SDL_PenTouchEvent,
    SDL_QuitEvent, SDL_UserEvent, SDL_WindowEvent,
};
use crate::sys::everything::SDL_DisplayOrientation;
use crate::sys::stdinc::Uint16;

lazy_static::lazy_static! {
    static ref CUSTOM_EVENT_TYPES: Mutex<CustomEventTypeMaps> = Mutex::new(CustomEventTypeMaps::new());
}

#[derive(Debug)]
struct CustomEventTypeMaps {
    sdl_id_to_type_id: HashMap<u32, TypeId>,
    type_id_to_sdl_id: HashMap<TypeId, u32>,
}
impl CustomEventTypeMaps {
    fn new() -> Self {
        Self {
            sdl_id_to_type_id: HashMap::new(),
            type_id_to_sdl_id: HashMap::new(),
        }
    }
}

/* --------------------------- Raw Event Type Mapping --------------------------- */

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
    DisplayMoved = sys::events::SDL_EVENT_DISPLAY_MOVED.0,
    DisplayDesktopModeChanged = sys::events::SDL_EVENT_DISPLAY_DESKTOP_MODE_CHANGED.0,
    DisplayCurrentModeChanged = sys::events::SDL_EVENT_DISPLAY_CURRENT_MODE_CHANGED.0,
    DisplayContentScaleChanged = sys::events::SDL_EVENT_DISPLAY_CONTENT_SCALE_CHANGED.0,

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

    ClipboardUpdate = sys::events::SDL_EVENT_CLIPBOARD_UPDATE.0,
    DropFile = sys::events::SDL_EVENT_DROP_FILE.0,
    DropText = sys::events::SDL_EVENT_DROP_TEXT.0,
    DropBegin = sys::events::SDL_EVENT_DROP_BEGIN.0,
    DropComplete = sys::events::SDL_EVENT_DROP_COMPLETE.0,

    AudioDeviceAdded = sys::events::SDL_EVENT_AUDIO_DEVICE_ADDED.0,
    AudioDeviceRemoved = sys::events::SDL_EVENT_AUDIO_DEVICE_REMOVED.0,

    PenProximityIn = sys::events::SDL_EVENT_PEN_PROXIMITY_IN.0,
    PenProximityOut = sys::events::SDL_EVENT_PEN_PROXIMITY_OUT.0,
    PenDown = sys::events::SDL_EVENT_PEN_DOWN.0,
    PenUp = sys::events::SDL_EVENT_PEN_UP.0,
    PenButtonUp = sys::events::SDL_EVENT_PEN_BUTTON_UP.0,
    PenButtonDown = sys::events::SDL_EVENT_PEN_BUTTON_DOWN.0,
    PenMotion = sys::events::SDL_EVENT_PEN_MOTION.0,
    PenAxis = sys::events::SDL_EVENT_PEN_AXIS.0,

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
impl From<EventType> for SDL_EventType {
    fn from(t: EventType) -> SDL_EventType {
        SDL_EventType(t as u32)
    }
}

impl TryFrom<u32> for EventType {
    type Error = ();
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        use crate::sys::events::*;
        Ok(match SDL_EventType(n) {
            SDL_EVENT_FIRST => EventType::First,

            SDL_EVENT_QUIT => EventType::Quit,
            SDL_EVENT_TERMINATING => EventType::AppTerminating,
            SDL_EVENT_LOW_MEMORY => EventType::AppLowMemory,
            SDL_EVENT_WILL_ENTER_BACKGROUND => EventType::AppWillEnterBackground,
            SDL_EVENT_DID_ENTER_BACKGROUND => EventType::AppDidEnterBackground,
            SDL_EVENT_WILL_ENTER_FOREGROUND => EventType::AppWillEnterForeground,
            SDL_EVENT_DID_ENTER_FOREGROUND => EventType::AppDidEnterForeground,

            SDL_EVENT_DISPLAY_ADDED => EventType::DisplayAdded,
            SDL_EVENT_DISPLAY_REMOVED => EventType::DisplayRemoved,
            SDL_EVENT_DISPLAY_ORIENTATION => EventType::DisplayOrientation,
            SDL_EVENT_DISPLAY_MOVED => EventType::DisplayMoved,
            SDL_EVENT_DISPLAY_DESKTOP_MODE_CHANGED => EventType::DisplayDesktopModeChanged,
            SDL_EVENT_DISPLAY_CURRENT_MODE_CHANGED => EventType::DisplayCurrentModeChanged,
            SDL_EVENT_DISPLAY_CONTENT_SCALE_CHANGED => EventType::DisplayContentScaleChanged,

            SDL_EVENT_WINDOW_SHOWN => EventType::WindowShown,
            SDL_EVENT_WINDOW_HIDDEN => EventType::WindowHidden,
            SDL_EVENT_WINDOW_EXPOSED => EventType::WindowExposed,
            SDL_EVENT_WINDOW_MOVED => EventType::WindowMoved,
            SDL_EVENT_WINDOW_RESIZED => EventType::WindowResized,
            SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => EventType::WindowPixelSizeChanged,
            SDL_EVENT_WINDOW_MINIMIZED => EventType::WindowMinimized,
            SDL_EVENT_WINDOW_MAXIMIZED => EventType::WindowMaximized,
            SDL_EVENT_WINDOW_RESTORED => EventType::WindowRestored,
            SDL_EVENT_WINDOW_MOUSE_ENTER => EventType::WindowMouseEnter,
            SDL_EVENT_WINDOW_MOUSE_LEAVE => EventType::WindowMouseLeave,
            SDL_EVENT_WINDOW_FOCUS_GAINED => EventType::WindowFocusGained,
            SDL_EVENT_WINDOW_FOCUS_LOST => EventType::WindowFocusLost,
            SDL_EVENT_WINDOW_CLOSE_REQUESTED => EventType::WindowCloseRequested,
            SDL_EVENT_WINDOW_HIT_TEST => EventType::WindowHitTest,
            SDL_EVENT_WINDOW_ICCPROF_CHANGED => EventType::WindowICCProfileChanged,
            SDL_EVENT_WINDOW_DISPLAY_CHANGED => EventType::WindowDisplayChanged,

            SDL_EVENT_KEY_DOWN => EventType::KeyDown,
            SDL_EVENT_KEY_UP => EventType::KeyUp,
            SDL_EVENT_TEXT_EDITING => EventType::TextEditing,
            SDL_EVENT_TEXT_INPUT => EventType::TextInput,

            SDL_EVENT_MOUSE_MOTION => EventType::MouseMotion,
            SDL_EVENT_MOUSE_BUTTON_DOWN => EventType::MouseButtonDown,
            SDL_EVENT_MOUSE_BUTTON_UP => EventType::MouseButtonUp,
            SDL_EVENT_MOUSE_WHEEL => EventType::MouseWheel,

            SDL_EVENT_JOYSTICK_AXIS_MOTION => EventType::JoyAxisMotion,
            SDL_EVENT_JOYSTICK_HAT_MOTION => EventType::JoyHatMotion,
            SDL_EVENT_JOYSTICK_BUTTON_DOWN => EventType::JoyButtonDown,
            SDL_EVENT_JOYSTICK_BUTTON_UP => EventType::JoyButtonUp,
            SDL_EVENT_JOYSTICK_ADDED => EventType::JoyDeviceAdded,
            SDL_EVENT_JOYSTICK_REMOVED => EventType::JoyDeviceRemoved,

            SDL_EVENT_GAMEPAD_AXIS_MOTION => EventType::ControllerAxisMotion,
            SDL_EVENT_GAMEPAD_BUTTON_DOWN => EventType::ControllerButtonDown,
            SDL_EVENT_GAMEPAD_BUTTON_UP => EventType::ControllerButtonUp,
            SDL_EVENT_GAMEPAD_ADDED => EventType::ControllerDeviceAdded,
            SDL_EVENT_GAMEPAD_REMOVED => EventType::ControllerDeviceRemoved,
            SDL_EVENT_GAMEPAD_REMAPPED => EventType::ControllerDeviceRemapped,
            SDL_EVENT_GAMEPAD_TOUCHPAD_DOWN => EventType::ControllerTouchpadDown,
            SDL_EVENT_GAMEPAD_TOUCHPAD_MOTION => EventType::ControllerTouchpadMotion,
            SDL_EVENT_GAMEPAD_TOUCHPAD_UP => EventType::ControllerTouchpadUp,
            #[cfg(feature = "hidapi")]
            SDL_EVENT_GAMEPAD_SENSOR_UPDATE => EventType::ControllerSensorUpdated,

            SDL_EVENT_FINGER_DOWN => EventType::FingerDown,
            SDL_EVENT_FINGER_UP => EventType::FingerUp,
            SDL_EVENT_FINGER_MOTION => EventType::FingerMotion,

            SDL_EVENT_CLIPBOARD_UPDATE => EventType::ClipboardUpdate,
            SDL_EVENT_DROP_FILE => EventType::DropFile,
            SDL_EVENT_DROP_TEXT => EventType::DropText,
            SDL_EVENT_DROP_BEGIN => EventType::DropBegin,
            SDL_EVENT_DROP_COMPLETE => EventType::DropComplete,

            SDL_EVENT_AUDIO_DEVICE_ADDED => EventType::AudioDeviceAdded,
            SDL_EVENT_AUDIO_DEVICE_REMOVED => EventType::AudioDeviceRemoved,

            SDL_EVENT_PEN_PROXIMITY_IN => EventType::PenProximityIn,
            SDL_EVENT_PEN_PROXIMITY_OUT => EventType::PenProximityOut,
            SDL_EVENT_PEN_DOWN => EventType::PenDown,
            SDL_EVENT_PEN_UP => EventType::PenUp,
            SDL_EVENT_PEN_BUTTON_UP => EventType::PenButtonUp,
            SDL_EVENT_PEN_BUTTON_DOWN => EventType::PenButtonDown,
            SDL_EVENT_PEN_MOTION => EventType::PenMotion,
            SDL_EVENT_PEN_AXIS => EventType::PenAxis,

            SDL_EVENT_RENDER_TARGETS_RESET => EventType::RenderTargetsReset,
            SDL_EVENT_RENDER_DEVICE_RESET => EventType::RenderDeviceReset,

            SDL_EVENT_USER => EventType::User,
            SDL_EVENT_LAST => EventType::Last,
            _ => return Err(()),
        })
    }
}

/* --------------------------- Hierarchical Kinds --------------------------- */

#[derive(Clone, Debug, PartialEq)]
pub struct QuitEvent {
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    Terminating { timestamp: u64 },
    LowMemory { timestamp: u64 },
    WillEnterBackground { timestamp: u64 },
    DidEnterBackground { timestamp: u64 },
    WillEnterForeground { timestamp: u64 },
    DidEnterForeground { timestamp: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayEvent {
    Orientation {
        timestamp: u64,
        display: Display,
        orientation: Orientation,
    },
    Added {
        timestamp: u64,
        display: Display,
    },
    Removed {
        timestamp: u64,
        display: Display,
    },
    Moved {
        timestamp: u64,
        display: Display,
    },
    DesktopModeChanged {
        timestamp: u64,
        display: Display,
    },
    CurrentModeChanged {
        timestamp: u64,
        display: Display,
    },
    ContentScaleChanged {
        timestamp: u64,
        display: Display,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowEvent {
    pub timestamp: u64,
    pub window_id: u32,
    pub kind: WindowEventKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum WindowEventKind {
    Shown,
    Hidden,
    Exposed,
    Moved { x: i32, y: i32 },
    Resized { w: i32, h: i32 },
    PixelSizeChanged { w: i32, h: i32 },
    Minimized,
    Maximized,
    Restored,
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    CloseRequested,
    HitTest { data1: i32, data2: i32 },
    ICCProfileChanged,
    DisplayChanged { display_index: i32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardEvent {
    pub timestamp: u64,
    pub window_id: u32,
    pub state: KeyState,
    pub keycode: Option<Keycode>,
    pub scancode: Option<Scancode>,
    pub keymod: Mod,
    pub repeat: bool,
    pub which: u32,
    pub raw: Uint16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextEvent {
    Editing {
        timestamp: u64,
        window_id: u32,
        text: String,
        start: i32,
        length: i32,
    },
    Input {
        timestamp: u64,
        window_id: u32,
        text: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MouseButtonState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MouseEvent {
    Motion {
        timestamp: u64,
        window_id: u32,
        which: u32,
        state: MouseState,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    },
    Button {
        timestamp: u64,
        window_id: u32,
        which: u32,
        button: MouseButton,
        clicks: u8,
        state: MouseButtonState,
        x: f32,
        y: f32,
    },
    Wheel {
        timestamp: u64,
        window_id: u32,
        which: u32,
        x: f32,
        y: f32,
        direction: MouseWheelDirection,
        mouse_x: f32,
        mouse_y: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoyButtonState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoyDeviceChange {
    Added,
    Removed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoystickEvent {
    Axis {
        timestamp: u64,
        which: u32,
        axis_index: u8,
        value: i16,
    },
    Hat {
        timestamp: u64,
        which: u32,
        hat_index: u8,
        state: HatState,
    },
    Button {
        timestamp: u64,
        which: u32,
        button_index: u8,
        state: JoyButtonState,
    },
    Device {
        timestamp: u64,
        which: u32,
        change: JoyDeviceChange,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerButtonState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerDeviceChange {
    Added,
    Removed,
    Remapped,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerTouchpadKind {
    Down,
    Motion,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerEvent {
    Axis {
        timestamp: u64,
        which: u32,
        axis: GamepadAxis,
        value: i16,
    },
    Button {
        timestamp: u64,
        which: u32,
        button: GamepadButton,
        state: ControllerButtonState,
    },
    Device {
        timestamp: u64,
        which: u32,
        change: ControllerDeviceChange,
    },
    Touchpad {
        timestamp: u64,
        which: u32,
        touchpad: i32,
        finger: i32,
        kind: ControllerTouchpadKind,
        x: f32,
        y: f32,
        pressure: f32,
    },
    #[cfg(feature = "hidapi")]
    Sensor {
        timestamp: u64,
        which: u32,
        sensor: crate::sensor::SensorType,
        data: [f32; 3],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum FingerState {
    Down,
    Up,
    Motion,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TouchEvent {
    Finger {
        timestamp: u64,
        touch_id: u64,
        finger_id: u64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
        state: FingerState,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DropEvent {
    File {
        timestamp: u64,
        window_id: u32,
        filename: String,
    },
    Text {
        timestamp: u64,
        window_id: u32,
        text: String,
    },
    Begin {
        timestamp: u64,
        window_id: u32,
    },
    Complete {
        timestamp: u64,
        window_id: u32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioDeviceEvent {
    Added {
        timestamp: u64,
        which: u32,
        iscapture: bool,
    },
    Removed {
        timestamp: u64,
        which: u32,
        iscapture: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PenButtonState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PenProximityState {
    In,
    Out,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PenEvent {
    Proximity {
        timestamp: u64,
        which: u32,
        window_id: u32,
        state: PenProximityState,
    },
    Touch {
        timestamp: u64,
        which: u32,
        window_id: u32,
        x: f32,
        y: f32,
        eraser: bool,
        down: bool,
    },
    Motion {
        timestamp: u64,
        which: u32,
        window_id: u32,
        x: f32,
        y: f32,
    },
    Button {
        timestamp: u64,
        which: u32,
        window_id: u32,
        x: f32,
        y: f32,
        button: u8,
        state: PenButtonState,
    },
    Axis {
        timestamp: u64,
        which: u32,
        window_id: u32,
        x: f32,
        y: f32,
        axis: PenAxis,
        value: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderEvent {
    TargetsReset { timestamp: u64 },
    DeviceReset { timestamp: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserEvent {
    pub timestamp: u64,
    pub window_id: u32,
    pub type_id: u32,
    pub code: i32,
    pub data1: *mut c_void,
    pub data2: *mut c_void,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownEvent {
    pub timestamp: u64,
    pub raw_type: u32,
}

/* --------------------------- Top-Level Event --------------------------- */

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Quit(QuitEvent),
    App(AppEvent),
    Display(DisplayEvent),
    Window(WindowEvent),
    Keyboard(KeyboardEvent),
    Text(TextEvent),
    Mouse(MouseEvent),
    Joystick(JoystickEvent),
    Controller(ControllerEvent),
    Touch(TouchEvent),
    Drop(DropEvent),
    Audio(AudioDeviceEvent),
    Pen(PenEvent),
    Render(RenderEvent),
    User(UserEvent),
    Unknown(UnknownEvent),
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

/* --------------------------- Conversion Helpers --------------------------- */

pub struct OwnedRawEvent {
    pub event: SDL_Event,
    // Hold CStrings to keep their memory alive while SDL may read the pointers.
    _buffers: Vec<CString>,
}

impl Drop for OwnedRawEvent {
    fn drop(&mut self) {
        // Pointers become invalid automatically when CStrings drop.
    }
}

impl Event {
    pub fn get_timestamp(&self) -> u64 {
        match self {
            Event::Quit(QuitEvent { timestamp }) => *timestamp,
            Event::App(a) => match a {
                AppEvent::Terminating { timestamp }
                | AppEvent::LowMemory { timestamp }
                | AppEvent::WillEnterBackground { timestamp }
                | AppEvent::DidEnterBackground { timestamp }
                | AppEvent::WillEnterForeground { timestamp }
                | AppEvent::DidEnterForeground { timestamp } => *timestamp,
            },
            Event::Display(d) => match d {
                DisplayEvent::Orientation { timestamp, .. }
                | DisplayEvent::Added { timestamp, .. }
                | DisplayEvent::Removed { timestamp, .. }
                | DisplayEvent::Moved { timestamp, .. }
                | DisplayEvent::DesktopModeChanged { timestamp, .. }
                | DisplayEvent::CurrentModeChanged { timestamp, .. }
                | DisplayEvent::ContentScaleChanged { timestamp, .. } => *timestamp,
            },
            Event::Window(w) => w.timestamp,
            Event::Keyboard(k) => k.timestamp,
            Event::Text(t) => match t {
                TextEvent::Editing { timestamp, .. } | TextEvent::Input { timestamp, .. } => {
                    *timestamp
                }
            },
            Event::Mouse(m) => match m {
                MouseEvent::Motion { timestamp, .. }
                | MouseEvent::Button { timestamp, .. }
                | MouseEvent::Wheel { timestamp, .. } => *timestamp,
            },
            Event::Joystick(j) => match j {
                JoystickEvent::Axis { timestamp, .. }
                | JoystickEvent::Hat { timestamp, .. }
                | JoystickEvent::Button { timestamp, .. }
                | JoystickEvent::Device { timestamp, .. } => *timestamp,
            },
            Event::Controller(c) => match c {
                ControllerEvent::Axis { timestamp, .. } => *timestamp,
                ControllerEvent::Button { timestamp, .. } => *timestamp,
                ControllerEvent::Device { timestamp, .. } => *timestamp,
                ControllerEvent::Touchpad { timestamp, .. } => *timestamp,
                #[cfg(feature = "hidapi")]
                ControllerEvent::Sensor { timestamp, .. } => *timestamp,
            },
            Event::Touch(tch) => match tch {
                TouchEvent::Finger { timestamp, .. } => *timestamp,
            },
            Event::Drop(d) => match d {
                DropEvent::File { timestamp, .. }
                | DropEvent::Text { timestamp, .. }
                | DropEvent::Begin { timestamp, .. }
                | DropEvent::Complete { timestamp, .. } => *timestamp,
            },
            Event::Audio(a) => match a {
                AudioDeviceEvent::Added { timestamp, .. }
                | AudioDeviceEvent::Removed { timestamp, .. } => *timestamp,
            },
            Event::Pen(p) => match p {
                PenEvent::Proximity { timestamp, .. }
                | PenEvent::Touch { timestamp, .. }
                | PenEvent::Motion { timestamp, .. }
                | PenEvent::Button { timestamp, .. }
                | PenEvent::Axis { timestamp, .. } => *timestamp,
            },
            Event::Render(r) => match r {
                RenderEvent::TargetsReset { timestamp }
                | RenderEvent::DeviceReset { timestamp } => *timestamp,
            },
            Event::User(u) => u.timestamp,
            Event::Unknown(u) => u.timestamp,
        }
    }

    pub fn get_window_id(&self) -> Option<u32> {
        match self {
            Event::Window(w) => Some(w.window_id),
            Event::Keyboard(k) => Some(k.window_id),
            Event::Text(t) => match t {
                TextEvent::Editing { window_id, .. } | TextEvent::Input { window_id, .. } => {
                    Some(*window_id)
                }
            },
            Event::Mouse(m) => match m {
                MouseEvent::Motion { window_id, .. }
                | MouseEvent::Button { window_id, .. }
                | MouseEvent::Wheel { window_id, .. } => Some(*window_id),
            },
            Event::Drop(d) => match d {
                DropEvent::File { window_id, .. }
                | DropEvent::Text { window_id, .. }
                | DropEvent::Begin { window_id, .. }
                | DropEvent::Complete { window_id, .. } => Some(*window_id),
            },
            Event::User(u) => Some(u.window_id),
            Event::Pen(p) => match p {
                PenEvent::Proximity { window_id, .. }
                | PenEvent::Touch { window_id, .. }
                | PenEvent::Motion { window_id, .. }
                | PenEvent::Button { window_id, .. }
                | PenEvent::Axis { window_id, .. } => Some(*window_id),
            },
            _ => None,
        }
    }

    pub fn is_keyboard(&self) -> bool {
        matches!(self, Event::Keyboard(_))
    }
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }
    pub fn is_window(&self) -> bool {
        matches!(self, Event::Window(_))
    }
    pub fn is_controller(&self) -> bool {
        matches!(self, Event::Controller(_))
    }
    pub fn is_joystick(&self) -> bool {
        matches!(self, Event::Joystick(_))
    }
    pub fn is_touch(&self) -> bool {
        matches!(self, Event::Touch(_))
    }
    pub fn is_pen(&self) -> bool {
        matches!(self, Event::Pen(_))
    }
    pub fn is_drop(&self) -> bool {
        matches!(self, Event::Drop(_))
    }
    pub fn is_audio(&self) -> bool {
        matches!(self, Event::Audio(_))
    }
    pub fn is_render(&self) -> bool {
        matches!(self, Event::Render(_))
    }
    pub fn is_user(&self) -> bool {
        matches!(self, Event::User(_))
    }
    pub fn is_unknown(&self) -> bool {
        matches!(self, Event::Unknown(_))
    }

    /* ---------------- Category Accessors (as_*) ---------------- */

    pub fn as_quit(&self) -> Option<&QuitEvent> {
        if let Event::Quit(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_app(&self) -> Option<&AppEvent> {
        if let Event::App(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_display(&self) -> Option<&DisplayEvent> {
        if let Event::Display(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_window(&self) -> Option<&WindowEvent> {
        if let Event::Window(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_keyboard(&self) -> Option<&KeyboardEvent> {
        if let Event::Keyboard(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_text(&self) -> Option<&TextEvent> {
        if let Event::Text(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        if let Event::Mouse(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_joystick(&self) -> Option<&JoystickEvent> {
        if let Event::Joystick(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_controller(&self) -> Option<&ControllerEvent> {
        if let Event::Controller(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_touch(&self) -> Option<&TouchEvent> {
        if let Event::Touch(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_drop(&self) -> Option<&DropEvent> {
        if let Event::Drop(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_audio(&self) -> Option<&AudioDeviceEvent> {
        if let Event::Audio(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_pen(&self) -> Option<&PenEvent> {
        if let Event::Pen(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_render(&self) -> Option<&RenderEvent> {
        if let Event::Render(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_user(&self) -> Option<&UserEvent> {
        if let Event::User(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_unknown(&self) -> Option<&UnknownEvent> {
        if let Event::Unknown(e) = self {
            Some(e)
        } else {
            None
        }
    }

    pub fn is_same_kind_as(&self, other: &Event) -> bool {
        use Event::*;
        match (self, other) {
            (Quit(_), Quit(_))
            | (App(_), App(_))
            | (Display(_), Display(_))
            | (Window(_), Window(_))
            | (Keyboard(_), Keyboard(_))
            | (Text(_), Text(_))
            | (Mouse(_), Mouse(_))
            | (Joystick(_), Joystick(_))
            | (Controller(_), Controller(_))
            | (Touch(_), Touch(_))
            | (Drop(_), Drop(_))
            | (Audio(_), Audio(_))
            | (Pen(_), Pen(_))
            | (Render(_), Render(_))
            | (User(_), User(_))
            | (Unknown(_), Unknown(_)) => true,
            _ => false,
        }
    }

    /* ---------------- Raw Conversion (to SDL) ---------------- */
    pub fn to_ll(&self) -> Option<SDL_Event> {
        let mut raw = mem::MaybeUninit::<SDL_Event>::uninit();
        unsafe {
            match self {
                Event::Quit(q) => {
                    let mut e: SDL_QuitEvent = Default::default();
                    e.r#type = SDL_EventType(EventType::Quit as u32);
                    e.timestamp = q.timestamp;
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_QuitEvent, 1);
                    Some(raw.assume_init())
                }
                Event::Window(w) => {
                    let (t, d1, d2) = window_kind_to_ll(&w.kind);
                    let mut e: SDL_WindowEvent = Default::default();
                    e.r#type = SDL_EventType(t as u32);
                    e.timestamp = w.timestamp;
                    e.windowID = w.window_id;
                    e.data1 = d1;
                    e.data2 = d2;
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_WindowEvent, 1);
                    Some(raw.assume_init())
                }
                Event::Keyboard(k) => {
                    let down = matches!(k.state, KeyState::Down);
                    let sc = k.scancode?;
                    let kc = k.keycode?;
                    let mut e: SDL_KeyboardEvent = Default::default();
                    e.r#type = SDL_EventType(match k.state {
                        KeyState::Down => EventType::KeyDown as u32,
                        KeyState::Up => EventType::KeyUp as u32,
                    });
                    e.timestamp = k.timestamp;
                    e.windowID = k.window_id;
                    e.repeat = k.repeat;
                    e.scancode = sc.into();
                    e.which = k.which;
                    e.down = down;
                    e.key = kc.into();
                    e.r#mod = k.keymod.bits();
                    e.raw = k.raw;
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_KeyboardEvent, 1);
                    Some(raw.assume_init())
                }
                Event::Mouse(m) => match m {
                    MouseEvent::Motion {
                        timestamp,
                        window_id,
                        which,
                        state,
                        x,
                        y,
                        xrel,
                        yrel,
                    } => {
                        let mut e: SDL_MouseMotionEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::MouseMotion as u32);
                        e.timestamp = *timestamp;
                        e.windowID = *window_id;
                        e.which = *which;
                        e.state = state.to_sdl_state();
                        e.x = *x;
                        e.y = *y;
                        e.xrel = *xrel;
                        e.yrel = *yrel;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_MouseMotionEvent, 1);
                        Some(raw.assume_init())
                    }
                    MouseEvent::Button {
                        timestamp,
                        window_id,
                        which,
                        button,
                        clicks,
                        state,
                        x,
                        y,
                    } => {
                        let mut e: SDL_MouseButtonEvent = Default::default();
                        e.r#type = SDL_EventType(match state {
                            MouseButtonState::Down => EventType::MouseButtonDown as u32,
                            MouseButtonState::Up => EventType::MouseButtonUp as u32,
                        });
                        e.timestamp = *timestamp;
                        e.windowID = *window_id;
                        e.which = *which;
                        e.button = *button as u8;
                        e.down = matches!(state, MouseButtonState::Down);
                        e.clicks = *clicks;
                        e.x = *x;
                        e.y = *y;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_MouseButtonEvent, 1);
                        Some(raw.assume_init())
                    }
                    MouseEvent::Wheel {
                        timestamp,
                        window_id,
                        which,
                        x,
                        y,
                        direction,
                        mouse_x,
                        mouse_y,
                    } => {
                        let mut e: SDL_MouseWheelEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::MouseWheel as u32);
                        e.timestamp = *timestamp;
                        e.windowID = *window_id;
                        e.which = *which;
                        e.x = *x;
                        e.y = *y;
                        e.direction = (*direction).into();
                        e.mouse_x = *mouse_x;
                        e.mouse_y = *mouse_y;
                        // integer_x / integer_y left at default (0)
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_MouseWheelEvent, 1);
                        Some(raw.assume_init())
                    }
                },
                Event::Joystick(j) => match j {
                    JoystickEvent::Axis {
                        timestamp,
                        which,
                        axis_index,
                        value,
                    } => {
                        let mut e: SDL_JoyAxisEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::JoyAxisMotion as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.axis = *axis_index;
                        e.value = *value;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_JoyAxisEvent, 1);
                        Some(raw.assume_init())
                    }
                    JoystickEvent::Hat {
                        timestamp,
                        which,
                        hat_index,
                        state,
                    } => {
                        let mut e: SDL_JoyHatEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::JoyHatMotion as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.hat = *hat_index;
                        e.value = state.to_raw();
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_JoyHatEvent, 1);
                        Some(raw.assume_init())
                    }
                    JoystickEvent::Button {
                        timestamp,
                        which,
                        button_index,
                        state,
                    } => {
                        let mut e: SDL_JoyButtonEvent = Default::default();
                        e.r#type = SDL_EventType(match state {
                            JoyButtonState::Down => EventType::JoyButtonDown as u32,
                            JoyButtonState::Up => EventType::JoyButtonUp as u32,
                        });
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.button = *button_index;
                        e.down = matches!(state, JoyButtonState::Down);
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_JoyButtonEvent, 1);
                        Some(raw.assume_init())
                    }
                    JoystickEvent::Device {
                        timestamp,
                        which,
                        change,
                    } => {
                        let et = match change {
                            JoyDeviceChange::Added => EventType::JoyDeviceAdded,
                            JoyDeviceChange::Removed => EventType::JoyDeviceRemoved,
                        };
                        let e = SDL_JoyDeviceEvent {
                            r#type: SDL_EventType(et as u32),
                            timestamp: *timestamp,
                            which: *which,
                            reserved: 0,
                        };
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_JoyDeviceEvent, 1);
                        Some(raw.assume_init())
                    }
                },
                Event::Controller(c) => match c {
                    ControllerEvent::Axis {
                        timestamp,
                        which,
                        axis,
                        value,
                    } => {
                        let mut e: SDL_GamepadAxisEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::ControllerAxisMotion as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.axis = (*axis).into();
                        e.value = *value;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_GamepadAxisEvent, 1);
                        Some(raw.assume_init())
                    }
                    ControllerEvent::Button {
                        timestamp,
                        which,
                        button,
                        state,
                    } => {
                        let mut e: SDL_GamepadButtonEvent = Default::default();
                        e.r#type = SDL_EventType(match state {
                            ControllerButtonState::Down => EventType::ControllerButtonDown as u32,
                            ControllerButtonState::Up => EventType::ControllerButtonUp as u32,
                        });
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.button = (*button).into();
                        e.down = matches!(state, ControllerButtonState::Down);
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_GamepadButtonEvent, 1);
                        Some(raw.assume_init())
                    }
                    ControllerEvent::Device {
                        timestamp,
                        which,
                        change,
                    } => {
                        let et = match change {
                            ControllerDeviceChange::Added => EventType::ControllerDeviceAdded,
                            ControllerDeviceChange::Removed => EventType::ControllerDeviceRemoved,
                            ControllerDeviceChange::Remapped => EventType::ControllerDeviceRemapped,
                        };
                        let mut e: SDL_GamepadDeviceEvent = Default::default();
                        e.r#type = SDL_EventType(et as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_GamepadDeviceEvent, 1);
                        Some(raw.assume_init())
                    }
                    ControllerEvent::Touchpad {
                        timestamp,
                        which,
                        touchpad,
                        finger,
                        kind,
                        x,
                        y,
                        pressure,
                    } => {
                        let et = match kind {
                            ControllerTouchpadKind::Down => EventType::ControllerTouchpadDown,
                            ControllerTouchpadKind::Motion => EventType::ControllerTouchpadMotion,
                            ControllerTouchpadKind::Up => EventType::ControllerTouchpadUp,
                        };
                        let mut e: SDL_GamepadTouchpadEvent = Default::default();
                        e.r#type = SDL_EventType(et as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.touchpad = *touchpad;
                        e.finger = *finger;
                        e.x = *x;
                        e.y = *y;
                        e.pressure = *pressure;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_GamepadTouchpadEvent, 1);
                        Some(raw.assume_init())
                    }
                    #[cfg(feature = "hidapi")]
                    ControllerEvent::Sensor {
                        timestamp,
                        which,
                        sensor,
                        data,
                    } => {
                        let mut e: sys::events::SDL_GamepadSensorEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::ControllerSensorUpdated as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.sensor = sensor.to_ll().0;
                        e.data = *data;
                        ptr::copy(
                            &e,
                            raw.as_mut_ptr() as *mut sys::events::SDL_GamepadSensorEvent,
                            1,
                        );
                        Some(raw.assume_init())
                    }
                },
                Event::Display(d) => {
                    let (et, display_id, data1) = match d {
                        DisplayEvent::Orientation {
                            timestamp: _,
                            display,
                            orientation,
                        } => (
                            EventType::DisplayOrientation,
                            display.id,
                            orientation.to_ll().0 as i32,
                        ),
                        DisplayEvent::Added { display, .. } => {
                            (EventType::DisplayAdded, display.id, 0)
                        }
                        DisplayEvent::Removed { display, .. } => {
                            (EventType::DisplayRemoved, display.id, 0)
                        }
                        DisplayEvent::Moved { display, .. } => {
                            (EventType::DisplayMoved, display.id, 0)
                        }
                        DisplayEvent::DesktopModeChanged { display, .. } => {
                            (EventType::DisplayDesktopModeChanged, display.id, 0)
                        }
                        DisplayEvent::CurrentModeChanged { display, .. } => {
                            (EventType::DisplayCurrentModeChanged, display.id, 0)
                        }
                        DisplayEvent::ContentScaleChanged { display, .. } => {
                            (EventType::DisplayContentScaleChanged, display.id, 0)
                        }
                    };
                    let ts = self.get_timestamp();
                    let mut e: SDL_DisplayEvent = Default::default();
                    e.r#type = SDL_EventType(et as u32);
                    e.displayID = display_id;
                    e.timestamp = ts;
                    e.data1 = data1;
                    // data2 left at default (0)
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_DisplayEvent, 1);
                    Some(raw.assume_init())
                }
                Event::Text(_) => {
                    // Cannot safely reconstruct raw text pointer (SDL expects owned C buffer).
                    // For now we decline conversion.
                    None
                }
                Event::Touch(_) => {
                    // Touch events not convertible to raw form here.
                    None
                }
                Event::Drop(_) => {
                    // Similar to Text, cannot reconstruct original C string from owned Rust String.
                    None
                }
                Event::Audio(a) => match a {
                    AudioDeviceEvent::Added {
                        timestamp,
                        which,
                        iscapture,
                    } => {
                        let mut e: SDL_AudioDeviceEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::AudioDeviceAdded as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.recording = *iscapture;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_AudioDeviceEvent, 1);
                        Some(raw.assume_init())
                    }
                    AudioDeviceEvent::Removed {
                        timestamp,
                        which,
                        iscapture,
                    } => {
                        let mut e: SDL_AudioDeviceEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::AudioDeviceRemoved as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.recording = *iscapture;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_AudioDeviceEvent, 1);
                        Some(raw.assume_init())
                    }
                },
                Event::Pen(p) => match p {
                    PenEvent::Proximity {
                        timestamp,
                        which,
                        window_id,
                        state,
                    } => {
                        let et = match state {
                            PenProximityState::In => EventType::PenProximityIn,
                            PenProximityState::Out => EventType::PenProximityOut,
                        };
                        let mut e: SDL_PenProximityEvent = Default::default();
                        e.r#type = SDL_EventType(et as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.windowID = *window_id;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_PenProximityEvent, 1);
                        Some(raw.assume_init())
                    }
                    PenEvent::Touch {
                        timestamp,
                        which,
                        window_id,
                        x,
                        y,
                        eraser,
                        down,
                    } => {
                        let et = if *down {
                            EventType::PenDown
                        } else {
                            EventType::PenUp
                        };
                        let mut e: SDL_PenTouchEvent = Default::default();
                        e.r#type = SDL_EventType(et as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.windowID = *window_id;
                        e.x = *x;
                        e.y = *y;
                        e.eraser = *eraser;
                        e.down = *down;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_PenTouchEvent, 1);
                        Some(raw.assume_init())
                    }
                    PenEvent::Motion {
                        timestamp,
                        which,
                        window_id,
                        x,
                        y,
                    } => {
                        let mut e: SDL_PenMotionEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::PenMotion as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.windowID = *window_id;
                        e.x = *x;
                        e.y = *y;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_PenMotionEvent, 1);
                        Some(raw.assume_init())
                    }
                    PenEvent::Button {
                        timestamp,
                        which,
                        window_id,
                        x,
                        y,
                        button,
                        state,
                    } => {
                        let et = match state {
                            PenButtonState::Down => EventType::PenButtonDown,
                            PenButtonState::Up => EventType::PenButtonUp,
                        };
                        let mut e: SDL_PenButtonEvent = Default::default();
                        e.r#type = SDL_EventType(et as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.windowID = *window_id;
                        e.x = *x;
                        e.y = *y;
                        e.button = *button;
                        e.down = matches!(state, PenButtonState::Down);
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_PenButtonEvent, 1);
                        Some(raw.assume_init())
                    }
                    PenEvent::Axis {
                        timestamp,
                        which,
                        window_id,
                        x,
                        y,
                        axis,
                        value,
                    } => {
                        let mut e: SDL_PenAxisEvent = Default::default();
                        e.r#type = SDL_EventType(EventType::PenAxis as u32);
                        e.timestamp = *timestamp;
                        e.which = *which;
                        e.windowID = *window_id;
                        e.x = *x;
                        e.y = *y;
                        e.axis = axis.to_ll(); // PenAxis -> SDL_PenAxis
                        e.value = *value;
                        ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_PenAxisEvent, 1);
                        Some(raw.assume_init())
                    }
                },
                Event::Render(r) => {
                    let et = match r {
                        RenderEvent::TargetsReset { .. } => EventType::RenderTargetsReset,
                        RenderEvent::DeviceReset { .. } => EventType::RenderDeviceReset,
                    };
                    let ts = self.get_timestamp();
                    let mut e: SDL_CommonEvent = Default::default();
                    e.r#type = et as u32;
                    e.timestamp = ts;
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_CommonEvent, 1);
                    Some(raw.assume_init())
                }
                Event::User(u) => {
                    let mut e: SDL_UserEvent = Default::default();
                    e.r#type = u.type_id;
                    e.timestamp = u.timestamp;
                    e.windowID = u.window_id;
                    e.code = u.code;
                    e.data1 = u.data1;
                    e.data2 = u.data2;
                    ptr::copy(&e, raw.as_mut_ptr() as *mut SDL_UserEvent, 1);
                    Some(raw.assume_init())
                }
                Event::App(_) | Event::Unknown(_) => None,
            }
        }
    }

    /// Owned raw event conversion supporting Text and Drop events by retaining backing C strings.
    /// For all other events it delegates to `to_ll`.
    /// The returned SDL_Event is valid as long as the `OwnedRawEvent` is kept alive.
    pub fn to_ll_owned(&self) -> Option<OwnedRawEvent> {
        use std::os::raw::c_char;

        // Fast path: for non-Text/Drop, reuse existing conversion.
        match self {
            Event::Text(t) => {
                let mut buffers: Vec<CString> = Vec::new();
                match t {
                    TextEvent::Editing {
                        timestamp,
                        window_id,
                        text,
                        start,
                        length,
                    } => {
                        let c = CString::new(text.as_str()).ok()?;
                        let ptr = c.as_ptr();
                        buffers.push(c);
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.edit;
                            e.r#type = SDL_EventType(EventType::TextEditing as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            // Copy text pointer
                            e.text = ptr as *mut c_char;
                            e.start = *start;
                            e.length = *length;
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                    TextEvent::Input {
                        timestamp,
                        window_id,
                        text,
                    } => {
                        let c = CString::new(text.as_str()).ok()?;
                        let ptr = c.as_ptr();
                        buffers.push(c);
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.text;
                            e.r#type = SDL_EventType(EventType::TextInput as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            e.text = ptr as *mut c_char;
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                }
            }
            Event::Drop(d) => {
                let mut buffers: Vec<CString> = Vec::new();
                match d {
                    DropEvent::File {
                        timestamp,
                        window_id,
                        filename,
                    } => {
                        let c = CString::new(filename.as_str()).ok()?;
                        let ptr = c.as_ptr();
                        buffers.push(c);
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.drop;
                            e.r#type = SDL_EventType(EventType::DropFile as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            e.data = ptr as *mut c_char;
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                    DropEvent::Text {
                        timestamp,
                        window_id,
                        text,
                    } => {
                        let c = CString::new(text.as_str()).ok()?;
                        let ptr = c.as_ptr();
                        buffers.push(c);
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.drop;
                            e.r#type = SDL_EventType(EventType::DropText as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            e.data = ptr as *mut c_char;
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                    DropEvent::Begin {
                        timestamp,
                        window_id,
                    } => {
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.drop;
                            e.r#type = SDL_EventType(EventType::DropBegin as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            e.data = std::ptr::null_mut();
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                    DropEvent::Complete {
                        timestamp,
                        window_id,
                    } => {
                        let mut raw_event: SDL_Event = unsafe { std::mem::zeroed() };
                        unsafe {
                            let e = &mut raw_event.drop;
                            e.r#type = SDL_EventType(EventType::DropComplete as u32);
                            e.timestamp = *timestamp;
                            e.windowID = *window_id;
                            e.data = std::ptr::null_mut();
                        }
                        Some(OwnedRawEvent {
                            event: raw_event,
                            _buffers: buffers,
                        })
                    }
                }
            }
            _ => self.to_ll().map(|ev| OwnedRawEvent {
                event: ev,
                _buffers: Vec::new(),
            }),
        }
    }

    /* ---------------- Raw Conversion (from SDL) ---------------- */
    pub fn from_ll(raw: SDL_Event) -> Event {
        let raw_type = unsafe { raw.r#type };
        let et = EventType::try_from(raw_type).unwrap_or(EventType::User);
        unsafe {
            match et {
                EventType::Quit => Event::Quit(QuitEvent {
                    timestamp: raw.quit.timestamp,
                }),
                EventType::AppTerminating => Event::App(AppEvent::Terminating {
                    timestamp: raw.common.timestamp,
                }),
                EventType::AppLowMemory => Event::App(AppEvent::LowMemory {
                    timestamp: raw.common.timestamp,
                }),
                EventType::AppWillEnterBackground => Event::App(AppEvent::WillEnterBackground {
                    timestamp: raw.common.timestamp,
                }),
                EventType::AppDidEnterBackground => Event::App(AppEvent::DidEnterBackground {
                    timestamp: raw.common.timestamp,
                }),
                EventType::AppWillEnterForeground => Event::App(AppEvent::WillEnterForeground {
                    timestamp: raw.common.timestamp,
                }),
                EventType::AppDidEnterForeground => Event::App(AppEvent::DidEnterForeground {
                    timestamp: raw.common.timestamp,
                }),

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
                    let w = raw.window;
                    let kind = match et {
                        EventType::WindowShown => WindowEventKind::Shown,
                        EventType::WindowHidden => WindowEventKind::Hidden,
                        EventType::WindowExposed => WindowEventKind::Exposed,
                        EventType::WindowMoved => WindowEventKind::Moved {
                            x: w.data1,
                            y: w.data2,
                        },
                        EventType::WindowResized => WindowEventKind::Resized {
                            w: w.data1,
                            h: w.data2,
                        },
                        EventType::WindowPixelSizeChanged => WindowEventKind::PixelSizeChanged {
                            w: w.data1,
                            h: w.data2,
                        },
                        EventType::WindowMinimized => WindowEventKind::Minimized,
                        EventType::WindowMaximized => WindowEventKind::Maximized,
                        EventType::WindowRestored => WindowEventKind::Restored,
                        EventType::WindowMouseEnter => WindowEventKind::MouseEnter,
                        EventType::WindowMouseLeave => WindowEventKind::MouseLeave,
                        EventType::WindowFocusGained => WindowEventKind::FocusGained,
                        EventType::WindowFocusLost => WindowEventKind::FocusLost,
                        EventType::WindowCloseRequested => WindowEventKind::CloseRequested,
                        EventType::WindowHitTest => WindowEventKind::HitTest {
                            data1: w.data1,
                            data2: w.data2,
                        },
                        EventType::WindowICCProfileChanged => WindowEventKind::ICCProfileChanged,
                        EventType::WindowDisplayChanged => WindowEventKind::DisplayChanged {
                            display_index: w.data1,
                        },
                        _ => WindowEventKind::Shown,
                    };
                    Event::Window(WindowEvent {
                        timestamp: w.timestamp,
                        window_id: w.windowID,
                        kind,
                    })
                }

                EventType::DisplayOrientation
                | EventType::DisplayAdded
                | EventType::DisplayRemoved
                | EventType::DisplayMoved
                | EventType::DisplayDesktopModeChanged
                | EventType::DisplayCurrentModeChanged
                | EventType::DisplayContentScaleChanged => {
                    let d = raw.display;
                    let display = Display::from_ll(d.displayID);
                    let timestamp = d.timestamp;
                    let ev = match et {
                        EventType::DisplayOrientation => {
                            let orientation =
                                if d.data1 > SDL_DisplayOrientation::PORTRAIT_FLIPPED.0 {
                                    Orientation::Unknown
                                } else {
                                    Orientation::from_ll(SDL_DisplayOrientation(d.data1))
                                };
                            DisplayEvent::Orientation {
                                timestamp,
                                display,
                                orientation,
                            }
                        }
                        EventType::DisplayAdded => DisplayEvent::Added { timestamp, display },
                        EventType::DisplayRemoved => DisplayEvent::Removed { timestamp, display },
                        EventType::DisplayMoved => DisplayEvent::Moved { timestamp, display },
                        EventType::DisplayDesktopModeChanged => {
                            DisplayEvent::DesktopModeChanged { timestamp, display }
                        }
                        EventType::DisplayCurrentModeChanged => {
                            DisplayEvent::CurrentModeChanged { timestamp, display }
                        }
                        EventType::DisplayContentScaleChanged => {
                            DisplayEvent::ContentScaleChanged { timestamp, display }
                        }
                        _ => DisplayEvent::Added { timestamp, display },
                    };
                    Event::Display(ev)
                }

                EventType::KeyDown | EventType::KeyUp => {
                    let k = raw.key;
                    let state = if et == EventType::KeyDown {
                        KeyState::Down
                    } else {
                        KeyState::Up
                    };
                    Event::Keyboard(KeyboardEvent {
                        timestamp: k.timestamp,
                        window_id: k.windowID,
                        state,
                        keycode: Keycode::from_i32(k.key as i32),
                        scancode: Scancode::from_i32(k.scancode.into()),
                        keymod: Mod::from_bits_truncate(k.r#mod),
                        repeat: k.repeat,
                        which: k.which,
                        raw: k.raw,
                    })
                }

                EventType::TextEditing => {
                    let e = raw.edit;
                    let text = CStr::from_ptr(e.text).to_string_lossy().into_owned();
                    Event::Text(TextEvent::Editing {
                        timestamp: e.timestamp,
                        window_id: e.windowID,
                        text,
                        start: e.start,
                        length: e.length,
                    })
                }
                EventType::TextInput => {
                    let e = raw.text;
                    let text = CStr::from_ptr(e.text).to_string_lossy().into_owned();
                    Event::Text(TextEvent::Input {
                        timestamp: e.timestamp,
                        window_id: e.windowID,
                        text,
                    })
                }

                EventType::MouseMotion => {
                    let m = raw.motion;
                    Event::Mouse(MouseEvent::Motion {
                        timestamp: m.timestamp,
                        window_id: m.windowID,
                        which: m.which,
                        state: MouseState::from_sdl_state(m.state),
                        x: m.x,
                        y: m.y,
                        xrel: m.xrel,
                        yrel: m.yrel,
                    })
                }
                EventType::MouseButtonDown | EventType::MouseButtonUp => {
                    let b = raw.button;
                    let state = if et == EventType::MouseButtonDown {
                        MouseButtonState::Down
                    } else {
                        MouseButtonState::Up
                    };
                    Event::Mouse(MouseEvent::Button {
                        timestamp: b.timestamp,
                        window_id: b.windowID,
                        which: b.which,
                        button: crate::mouse::MouseButton::from_ll(b.button),
                        clicks: b.clicks,
                        state,
                        x: b.x,
                        y: b.y,
                    })
                }
                EventType::MouseWheel => {
                    let w = raw.wheel;
                    Event::Mouse(MouseEvent::Wheel {
                        timestamp: w.timestamp,
                        window_id: w.windowID,
                        which: w.which,
                        x: w.x,
                        y: w.y,
                        direction: w.direction.into(),
                        mouse_x: w.mouse_x,
                        mouse_y: w.mouse_y,
                    })
                }

                EventType::JoyAxisMotion => {
                    let j = raw.jaxis;
                    Event::Joystick(JoystickEvent::Axis {
                        timestamp: j.timestamp,
                        which: j.which,
                        axis_index: j.axis,
                        value: j.value,
                    })
                }
                EventType::JoyHatMotion => {
                    let j = raw.jhat;
                    Event::Joystick(JoystickEvent::Hat {
                        timestamp: j.timestamp,
                        which: j.which,
                        hat_index: j.hat,
                        state: HatState::from_raw(j.value),
                    })
                }
                EventType::JoyButtonDown | EventType::JoyButtonUp => {
                    let jb = raw.jbutton;
                    let state = if et == EventType::JoyButtonDown {
                        JoyButtonState::Down
                    } else {
                        JoyButtonState::Up
                    };
                    Event::Joystick(JoystickEvent::Button {
                        timestamp: jb.timestamp,
                        which: jb.which,
                        button_index: jb.button,
                        state,
                    })
                }
                EventType::JoyDeviceAdded => {
                    let jd = raw.jdevice;
                    Event::Joystick(JoystickEvent::Device {
                        timestamp: jd.timestamp,
                        which: jd.which,
                        change: JoyDeviceChange::Added,
                    })
                }
                EventType::JoyDeviceRemoved => {
                    let jd = raw.jdevice;
                    Event::Joystick(JoystickEvent::Device {
                        timestamp: jd.timestamp,
                        which: jd.which,
                        change: JoyDeviceChange::Removed,
                    })
                }

                EventType::ControllerAxisMotion => {
                    let g = raw.gaxis;
                    let axis = crate::gamepad::Axis::from_ll(transmute(g.axis as i32)).unwrap();
                    Event::Controller(ControllerEvent::Axis {
                        timestamp: g.timestamp,
                        which: g.which,
                        axis,
                        value: g.value,
                    })
                }
                EventType::ControllerButtonDown | EventType::ControllerButtonUp => {
                    let b = raw.gbutton;
                    let button =
                        crate::gamepad::Button::from_ll(transmute(b.button as i32)).unwrap();
                    let state = if et == EventType::ControllerButtonDown {
                        ControllerButtonState::Down
                    } else {
                        ControllerButtonState::Up
                    };
                    Event::Controller(ControllerEvent::Button {
                        timestamp: b.timestamp,
                        which: b.which,
                        button,
                        state,
                    })
                }
                EventType::ControllerDeviceAdded => {
                    let d = raw.gdevice;
                    Event::Controller(ControllerEvent::Device {
                        timestamp: d.timestamp,
                        which: d.which,
                        change: ControllerDeviceChange::Added,
                    })
                }
                EventType::ControllerDeviceRemoved => {
                    let d = raw.gdevice;
                    Event::Controller(ControllerEvent::Device {
                        timestamp: d.timestamp,
                        which: d.which,
                        change: ControllerDeviceChange::Removed,
                    })
                }
                EventType::ControllerDeviceRemapped => {
                    let d = raw.gdevice;
                    Event::Controller(ControllerEvent::Device {
                        timestamp: d.timestamp,
                        which: d.which,
                        change: ControllerDeviceChange::Remapped,
                    })
                }
                EventType::ControllerTouchpadDown
                | EventType::ControllerTouchpadMotion
                | EventType::ControllerTouchpadUp => {
                    let t = raw.gtouchpad;
                    let kind = match et {
                        EventType::ControllerTouchpadDown => ControllerTouchpadKind::Down,
                        EventType::ControllerTouchpadMotion => ControllerTouchpadKind::Motion,
                        EventType::ControllerTouchpadUp => ControllerTouchpadKind::Up,
                        _ => ControllerTouchpadKind::Motion,
                    };
                    Event::Controller(ControllerEvent::Touchpad {
                        timestamp: t.timestamp,
                        which: t.which,
                        touchpad: t.touchpad,
                        finger: t.finger,
                        kind,
                        x: t.x,
                        y: t.y,
                        pressure: t.pressure,
                    })
                }
                #[cfg(feature = "hidapi")]
                EventType::ControllerSensorUpdated => {
                    let s = raw.gsensor;
                    Event::Controller(ControllerEvent::Sensor {
                        timestamp: s.timestamp,
                        which: s.which,
                        sensor: crate::sensor::SensorType::from_ll(s.sensor),
                        data: s.data,
                    })
                }

                EventType::FingerDown | EventType::FingerUp | EventType::FingerMotion => {
                    let f = raw.tfinger;
                    let state = match et {
                        EventType::FingerDown => FingerState::Down,
                        EventType::FingerUp => FingerState::Up,
                        _ => FingerState::Motion,
                    };
                    Event::Touch(TouchEvent::Finger {
                        timestamp: f.timestamp,
                        touch_id: f.touchID,
                        finger_id: f.fingerID,
                        x: f.x,
                        y: f.y,
                        dx: f.dx,
                        dy: f.dy,
                        pressure: f.pressure,
                        state,
                    })
                }

                EventType::ClipboardUpdate => Event::Unknown(UnknownEvent {
                    timestamp: raw.common.timestamp,
                    raw_type: et as u32,
                }),

                EventType::DropFile => {
                    let dr = raw.drop;
                    let text = CStr::from_ptr(dr.data as *const _)
                        .to_string_lossy()
                        .into_owned();
                    Event::Drop(DropEvent::File {
                        timestamp: dr.timestamp,
                        window_id: dr.windowID,
                        filename: text,
                    })
                }
                EventType::DropText => {
                    let dr = raw.drop;
                    let text = CStr::from_ptr(dr.data as *const _)
                        .to_string_lossy()
                        .into_owned();
                    Event::Drop(DropEvent::Text {
                        timestamp: dr.timestamp,
                        window_id: dr.windowID,
                        text,
                    })
                }
                EventType::DropBegin => {
                    let dr = raw.drop;
                    Event::Drop(DropEvent::Begin {
                        timestamp: dr.timestamp,
                        window_id: dr.windowID,
                    })
                }
                EventType::DropComplete => {
                    let dr = raw.drop;
                    Event::Drop(DropEvent::Complete {
                        timestamp: dr.timestamp,
                        window_id: dr.windowID,
                    })
                }

                EventType::AudioDeviceAdded => {
                    let a = raw.adevice;
                    Event::Audio(AudioDeviceEvent::Added {
                        timestamp: a.timestamp,
                        which: a.which,
                        iscapture: a.recording,
                    })
                }
                EventType::AudioDeviceRemoved => {
                    let a = raw.adevice;
                    Event::Audio(AudioDeviceEvent::Removed {
                        timestamp: a.timestamp,
                        which: a.which,
                        iscapture: a.recording,
                    })
                }

                EventType::PenProximityIn | EventType::PenProximityOut => {
                    let p = raw.pproximity;
                    let state = if et == EventType::PenProximityIn {
                        PenProximityState::In
                    } else {
                        PenProximityState::Out
                    };
                    Event::Pen(PenEvent::Proximity {
                        timestamp: p.timestamp,
                        which: p.which,
                        window_id: p.windowID,
                        state,
                    })
                }
                EventType::PenDown | EventType::PenUp => {
                    let p = raw.ptouch;
                    let down = et == EventType::PenDown;
                    Event::Pen(PenEvent::Touch {
                        timestamp: p.timestamp,
                        which: p.which,
                        window_id: p.windowID,
                        x: p.x,
                        y: p.y,
                        eraser: p.eraser,
                        down,
                    })
                }
                EventType::PenMotion => {
                    let p = raw.pmotion;
                    Event::Pen(PenEvent::Motion {
                        timestamp: p.timestamp,
                        which: p.which,
                        window_id: p.windowID,
                        x: p.x,
                        y: p.y,
                    })
                }
                EventType::PenButtonDown | EventType::PenButtonUp => {
                    let p = raw.pbutton;
                    let state = if et == EventType::PenButtonDown {
                        PenButtonState::Down
                    } else {
                        PenButtonState::Up
                    };
                    Event::Pen(PenEvent::Button {
                        timestamp: p.timestamp,
                        which: p.which,
                        window_id: p.windowID,
                        x: p.x,
                        y: p.y,
                        button: p.button,
                        state,
                    })
                }
                EventType::PenAxis => {
                    let p = raw.paxis;
                    Event::Pen(PenEvent::Axis {
                        timestamp: p.timestamp,
                        which: p.which,
                        window_id: p.windowID,
                        x: p.x,
                        y: p.y,
                        axis: PenAxis::from_ll(p.axis),
                        value: p.value,
                    })
                }

                EventType::RenderTargetsReset => Event::Render(RenderEvent::TargetsReset {
                    timestamp: raw.common.timestamp,
                }),
                EventType::RenderDeviceReset => Event::Render(RenderEvent::DeviceReset {
                    timestamp: raw.common.timestamp,
                }),

                EventType::First => panic!("Encountered EventType::First sentinel"),
                EventType::Last => panic!("Encountered EventType::Last sentinel"),

                EventType::User => {
                    // Distinguish unknown (built-in future) vs user.
                    if raw_type < 32_768 {
                        Event::Unknown(UnknownEvent {
                            timestamp: raw.common.timestamp,
                            raw_type,
                        })
                    } else {
                        let u = raw.user;
                        Event::User(UserEvent {
                            timestamp: u.timestamp,
                            window_id: u.windowID,
                            type_id: raw_type,
                            code: u.code,
                            data1: u.data1,
                            data2: u.data2,
                        })
                    }
                }
            }
        }
    }

    pub fn is_user_event(&self) -> bool {
        matches!(self, Event::User(_))
    }

    pub fn as_user_event_type<T: 'static>(&self) -> Option<T> {
        let (type_id, data_ptr) = match self {
            Event::User(u) => (u.type_id, u.data1),
            _ => return None,
        };
        let cet = CUSTOM_EVENT_TYPES.lock().unwrap();
        let expected_type_id = cet.sdl_id_to_type_id.get(&type_id)?;
        if *expected_type_id != TypeId::of::<Box<T>>() {
            return None;
        }
        // Safety: Stored as Box<T> originally
        let boxed: Box<T> = unsafe { Box::from_raw(data_ptr as *mut T) };
        Some(*boxed)
    }

    pub fn get_converted_coords<T: crate::render::RenderTarget>(
        &self,
        canvas: &crate::render::Canvas<T>,
    ) -> Option<Event> {
        let mut raw = self.to_ll()?;
        unsafe {
            sys::render::SDL_ConvertEventToRenderCoordinates(canvas.raw(), &mut raw);
        }
        Some(Event::from_ll(raw))
    }

    pub fn convert_coords<T: crate::render::RenderTarget>(
        &mut self,
        canvas: &crate::render::Canvas<T>,
    ) -> bool {
        if let Some(mut raw) = self.to_ll() {
            unsafe {
                sys::render::SDL_ConvertEventToRenderCoordinates(canvas.raw(), &mut raw);
            }
            *self = Event::from_ll(raw);
            true
        } else {
            false
        }
    }
}

/* --------------------------- Window Kind Helper --------------------------- */

fn window_kind_to_ll(kind: &WindowEventKind) -> (EventType, i32, i32) {
    match kind {
        WindowEventKind::Shown => (EventType::WindowShown, 0, 0),
        WindowEventKind::Hidden => (EventType::WindowHidden, 0, 0),
        WindowEventKind::Exposed => (EventType::WindowExposed, 0, 0),
        WindowEventKind::Moved { x, y } => (EventType::WindowMoved, *x, *y),
        WindowEventKind::Resized { w, h } => (EventType::WindowResized, *w, *h),
        WindowEventKind::PixelSizeChanged { w, h } => (EventType::WindowPixelSizeChanged, *w, *h),
        WindowEventKind::Minimized => (EventType::WindowMinimized, 0, 0),
        WindowEventKind::Maximized => (EventType::WindowMaximized, 0, 0),
        WindowEventKind::Restored => (EventType::WindowRestored, 0, 0),
        WindowEventKind::MouseEnter => (EventType::WindowMouseEnter, 0, 0),
        WindowEventKind::MouseLeave => (EventType::WindowMouseLeave, 0, 0),
        WindowEventKind::FocusGained => (EventType::WindowFocusGained, 0, 0),
        WindowEventKind::FocusLost => (EventType::WindowFocusLost, 0, 0),
        WindowEventKind::CloseRequested => (EventType::WindowCloseRequested, 0, 0),
        WindowEventKind::HitTest { data1, data2 } => (EventType::WindowHitTest, *data1, *data2),
        WindowEventKind::ICCProfileChanged => (EventType::WindowICCProfileChanged, 0, 0),
        WindowEventKind::DisplayChanged { display_index } => {
            (EventType::WindowDisplayChanged, *display_index, 0)
        }
    }
}

/* --------------------------- Event Subsystem API --------------------------- */

impl crate::EventSubsystem {
    #[doc(alias = "SDL_FlushEvent")]
    pub fn flush_event(&self, event_type: EventType) {
        unsafe { sys::events::SDL_FlushEvent(event_type.into()) };
    }

    #[doc(alias = "SDL_FlushEvents")]
    pub fn flush_events(&self, min_type: u32, max_type: u32) {
        unsafe { sys::events::SDL_FlushEvents(min_type, max_type) };
    }

    #[doc(alias = "SDL_PeepEvents")]
    pub fn peek_events<B>(&self, max_amount: u32) -> B
    where
        B: std::iter::FromIterator<Event>,
    {
        unsafe {
            let mut sdl_events = Vec::with_capacity(max_amount as usize);
            let result = {
                sys::events::SDL_PeepEvents(
                    sdl_events.as_mut_ptr(),
                    max_amount as c_int,
                    sys::events::SDL_PEEKEVENT,
                    sys::events::SDL_EVENT_FIRST.into(),
                    sys::events::SDL_EVENT_LAST.into(),
                )
            };
            if result < 0 {
                panic!("{}", get_error());
            }
            sdl_events.set_len(result as usize);
            sdl_events.into_iter().map(Event::from_ll).collect()
        }
    }

    pub fn push_event(&self, event: Event) -> Result<(), Error> {
        self.event_sender().push_event(event)
    }

    #[inline(always)]
    pub unsafe fn register_event(&self) -> Result<u32, Error> {
        Ok(*self.register_events(1)?.first().unwrap())
    }

    pub unsafe fn register_events(&self, nr: u32) -> Result<Vec<u32>, Error> {
        let result = sys::events::SDL_RegisterEvents(nr as c_int);
        const ERR_NR: u32 = u32::MAX - 1;
        match result {
            ERR_NR => Err(Error(
                "No more user events can be created; SDL_EVENT_LAST reached".into(),
            )),
            _ => Ok((result..(result + nr)).collect()),
        }
    }

    pub fn register_custom_event<T: 'static>(&self) -> Result<(), Error> {
        let event_id = *(unsafe { self.register_events(1) })?.first().unwrap();
        let mut cet = CUSTOM_EVENT_TYPES.lock().unwrap();
        let type_id = TypeId::of::<Box<T>>();
        if cet.type_id_to_sdl_id.contains_key(&type_id) {
            return Err(Error(
                "The same event type can not be registered twice!".into(),
            ));
        }
        cet.sdl_id_to_type_id.insert(event_id, type_id);
        cet.type_id_to_sdl_id.insert(type_id, event_id);
        Ok(())
    }

    pub fn push_custom_event<T: 'static>(&self, event: T) -> Result<(), Error> {
        self.event_sender().push_custom_event(event)
    }

    pub fn event_sender(&self) -> EventSender {
        EventSender { _priv: () }
    }

    pub fn add_event_watch<'a, CB: EventWatchCallback + 'a>(
        &self,
        callback: CB,
    ) -> EventWatch<'a, CB> {
        EventWatch::add(callback)
    }

    #[doc(alias = "SDL_SetEventEnabled")]
    pub fn set_event_enabled(event_type: EventType, enabled: bool) {
        unsafe { sys::events::SDL_SetEventEnabled(event_type.into(), enabled) };
    }

    #[doc(alias = "SDL_EventEnabled")]
    pub fn event_enabled(event_type: EventType) -> bool {
        unsafe { sys::events::SDL_EventEnabled(event_type.into()) }
    }
}

/* --------------------------- Poll/Wait Core --------------------------- */

unsafe fn poll_event() -> Option<Event> {
    let mut raw = mem::MaybeUninit::<SDL_Event>::uninit();
    let has = sys::events::SDL_PollEvent(raw.as_mut_ptr());
    if has {
        Some(Event::from_ll(raw.assume_init()))
    } else {
        None
    }
}
unsafe fn wait_event() -> Event {
    let mut raw = mem::MaybeUninit::<SDL_Event>::uninit();
    if sys::events::SDL_WaitEvent(raw.as_mut_ptr()) {
        Event::from_ll(raw.assume_init())
    } else {
        panic!("{}", get_error())
    }
}
unsafe fn wait_event_timeout(timeout: u32) -> Option<Event> {
    let mut raw = mem::MaybeUninit::<SDL_Event>::uninit();
    if sys::events::SDL_WaitEventTimeout(raw.as_mut_ptr(), timeout as c_int) {
        Some(Event::from_ll(raw.assume_init()))
    } else {
        None
    }
}

impl crate::EventPump {
    pub fn poll_event(&mut self) -> Option<Event> {
        unsafe { poll_event() }
    }
    pub fn poll_iter(&mut self) -> EventPollIterator {
        EventPollIterator
    }
    #[doc(alias = "SDL_PumpEvents")]
    pub fn pump_events(&mut self) {
        unsafe {
            sys::events::SDL_PumpEvents();
        }
    }
    pub fn wait_event(&mut self) -> Event {
        unsafe { wait_event() }
    }
    pub fn wait_event_timeout(&mut self, timeout: u32) -> Option<Event> {
        unsafe { wait_event_timeout(timeout) }
    }
    pub fn wait_iter(&mut self) -> EventWaitIterator {
        EventWaitIterator
    }
    pub fn wait_timeout_iter(&mut self, timeout: u32) -> EventWaitTimeoutIterator {
        EventWaitTimeoutIterator { timeout }
    }
    pub fn keyboard_state(&self) -> crate::keyboard::KeyboardState {
        crate::keyboard::KeyboardState::new(self)
    }
    pub fn mouse_state(&self) -> crate::mouse::MouseState {
        crate::mouse::MouseState::new(self)
    }
    pub fn relative_mouse_state(&self) -> crate::mouse::RelativeMouseState {
        crate::mouse::RelativeMouseState::new(self)
    }
}

/* --------------------------- Iterators --------------------------- */

pub struct EventPollIterator;
impl Iterator for EventPollIterator {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        unsafe { poll_event() }
    }
}

pub struct EventWaitIterator;
impl Iterator for EventWaitIterator {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        Some(unsafe { wait_event() })
    }
}

pub struct EventWaitTimeoutIterator {
    timeout: u32,
}
impl Iterator for EventWaitTimeoutIterator {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        unsafe { wait_event_timeout(self.timeout) }
    }
}

/* --------------------------- Event Sender --------------------------- */

pub struct EventSender {
    _priv: (),
}

impl EventSender {
    pub fn push_event(&self, event: Event) -> Result<(), Error> {
        match event.to_ll() {
            Some(mut raw) => {
                if unsafe { sys::events::SDL_PushEvent(&mut raw) } {
                    Ok(())
                } else {
                    Err(get_error())
                }
            }
            None => Err(Error(
                "Cannot push unsupported event type to the queue".into(),
            )),
        }
    }

    pub fn push_custom_event<T: 'static>(&self, event: T) -> Result<(), Error> {
        let cet = CUSTOM_EVENT_TYPES.lock().unwrap();
        let type_id = TypeId::of::<Box<T>>();
        let user_event_id = *cet
            .type_id_to_sdl_id
            .get(&type_id)
            .ok_or_else(|| Error("Type is not registered as a custom event type!".into()))?;
        let boxed = Box::new(event);
        let ev = Event::User(UserEvent {
            timestamp: 0,
            window_id: 0,
            type_id: user_event_id,
            code: 0,
            data1: Box::into_raw(boxed) as *mut c_void,
            data2: ptr::null_mut(),
        });
        drop(cet);
        self.push_event(ev)
    }
}

/* --------------------------- Event Watch --------------------------- */

pub trait EventWatchCallback: Send + Sync {
    fn callback(&mut self, event: Event);
}

pub struct EventWatch<'a, CB: EventWatchCallback + 'a> {
    activated: bool,
    callback: Box<CB>,
    _phantom: PhantomData<&'a CB>,
}

impl<'a, CB: EventWatchCallback + 'a> EventWatch<'a, CB> {
    fn add(callback: CB) -> Self {
        let mut w = EventWatch {
            activated: false,
            callback: Box::new(callback),
            _phantom: PhantomData,
        };
        w.activate();
        w
    }

    pub fn activate(&mut self) {
        if !self.activated {
            self.activated = true;
            unsafe { sys::events::SDL_AddEventWatch(self.filter(), self.callback_ptr()) };
        }
    }
    pub fn deactivate(&mut self) {
        if self.activated {
            self.activated = false;
            unsafe { sys::events::SDL_RemoveEventWatch(self.filter(), self.callback_ptr()) };
        }
    }
    pub fn activated(&self) -> bool {
        self.activated
    }
    pub fn set_activated(&mut self, activate: bool) {
        if activate {
            self.activate()
        } else {
            self.deactivate()
        }
    }

    fn filter(&self) -> sys::events::SDL_EventFilter {
        Some(event_callback_marshall::<CB>)
    }
    fn callback_ptr(&mut self) -> *mut c_void {
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
    event: *mut SDL_Event,
) -> bool {
    let cb: &mut CB = unsafe { &mut *(user_data as *mut CB) };
    let ev = Event::from_ll(unsafe { *event });
    cb.callback(ev);
    false
}

impl<F: FnMut(Event) + Send + Sync> EventWatchCallback for F {
    fn callback(&mut self, event: Event) {
        self(event)
    }
}

/* --------------------------- Tests (basic smoke) --------------------------- */

// PenAxis::to_ll moved to pen.rs; duplicate impl removed.

#[cfg(test)]
mod tests {
    #![allow(unused_imports, unused_variables, dead_code)]

    use super::*;
    use crate::keyboard::{Keycode, Scancode};
    use crate::mouse::MouseWheelDirection;

    #[test]
    fn keyboard_round_trip_basic() {
        let ev = Event::Keyboard(KeyboardEvent {
            timestamp: 123,
            window_id: 3,
            state: KeyState::Down,
            keycode: Some(Keycode::A),
            scancode: Some(Scancode::A),
            keymod: Mod::all(),
            repeat: false,
            which: 1,
            raw: 0,
        });
        let raw = ev.to_ll().expect("convertable");
        let ev2 = Event::from_ll(raw);
        assert!(matches!(ev2, Event::Keyboard(_)));
        assert_eq!(ev.get_timestamp(), ev2.get_timestamp());
    }

    #[test]
    fn mouse_button_round_trip() {
        let ev = Event::Mouse(MouseEvent::Button {
            timestamp: 5,
            window_id: 2,
            which: 0,
            button: MouseButton::Left,
            clicks: 2,
            state: MouseButtonState::Down,
            x: 10.0,
            y: 11.0,
        });
        let raw = ev.to_ll().unwrap();
        let back = Event::from_ll(raw);
        assert!(back.is_mouse());
    }

    #[test]
    fn wheel_not_equal_button() {
        let wheel = Event::Mouse(MouseEvent::Wheel {
            timestamp: 1,
            window_id: 0,
            which: 0,
            x: 0.0,
            y: -1.0,
            direction: MouseWheelDirection::Normal,
            mouse_x: 100.0,
            mouse_y: 200.0,
        });
        let btn = Event::Mouse(MouseEvent::Button {
            timestamp: 1,
            window_id: 0,
            which: 0,
            button: MouseButton::Left,
            clicks: 1,
            state: MouseButtonState::Down,
            x: 0.0,
            y: 0.0,
        });
        assert!(!wheel.is_same_kind_as(&Event::Keyboard(KeyboardEvent {
            timestamp: 0,
            window_id: 0,
            state: KeyState::Down,
            keycode: None,
            scancode: None,
            keymod: Mod::empty(),
            repeat: false,
            which: 0,
            raw: 0
        })));
        assert!(wheel.is_same_kind_as(&btn)); // Both Mouse category
    }

    #[test]
    fn joystick_axis_round_trip() {
        let ev = Event::Joystick(JoystickEvent::Axis {
            timestamp: 7,
            which: 11,
            axis_index: 2,
            value: 1234,
        });
        let raw = ev.to_ll().expect("joystick axis convertible");
        let back = Event::from_ll(raw);
        match back {
            Event::Joystick(JoystickEvent::Axis {
                timestamp,
                which,
                axis_index,
                value,
            }) => {
                assert_eq!(timestamp, 7);
                assert_eq!(which, 11);
                assert_eq!(axis_index, 2);
                assert_eq!(value, 1234);
            }
            _ => panic!("Unexpected event variant on joystick axis round trip"),
        }
    }

    #[test]
    fn controller_button_round_trip() {
        let ev = Event::Controller(ControllerEvent::Button {
            timestamp: 9,
            which: 22,
            button: crate::gamepad::Button::South,
            state: ControllerButtonState::Down,
        });
        let raw = ev.to_ll().expect("controller button convertible");
        let back = Event::from_ll(raw);
        match back {
            Event::Controller(ControllerEvent::Button {
                timestamp,
                which,
                button,
                state,
            }) => {
                assert_eq!(timestamp, 9);
                assert_eq!(which, 22);
                assert_eq!(button, crate::gamepad::Button::South);
                assert!(matches!(state, ControllerButtonState::Down));
            }
            _ => panic!("Unexpected event variant on controller button round trip"),
        }
    }

    #[test]
    fn pen_axis_round_trip() {
        let ev = Event::Pen(PenEvent::Axis {
            timestamp: 42,
            which: 5,
            window_id: 99,
            x: 10.0,
            y: 20.0,
            axis: PenAxis::Pressure,
            value: 0.5,
        });
        let raw = ev.to_ll().expect("pen axis convertible");
        let back = Event::from_ll(raw);
        match back {
            Event::Pen(PenEvent::Axis {
                timestamp,
                which,
                window_id,
                x,
                y,
                axis,
                value,
            }) => {
                assert_eq!(timestamp, 42);
                assert_eq!(which, 5);
                assert_eq!(window_id, 99);
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
                assert_eq!(axis, PenAxis::Pressure);
                assert_eq!(value, 0.5);
            }
            _ => panic!("Unexpected event variant on pen axis round trip"),
        }
    }

    #[test]
    fn text_input_owned_conversion() {
        let original_text = "Hello";
        let ev = Event::Text(TextEvent::Input {
            timestamp: 1,
            window_id: 2,
            text: original_text.into(),
        });
        let owned = ev.to_ll_owned().expect("owned text convertible");
        let back = Event::from_ll(owned.event);
        match back {
            Event::Text(TextEvent::Input {
                timestamp,
                window_id,
                text,
            }) => {
                assert_eq!(timestamp, 1);
                assert_eq!(window_id, 2);
                assert_eq!(text, original_text);
            }
            _ => panic!("Unexpected event variant on text input owned conversion"),
        }
    }

    #[test]
    fn drop_file_owned_conversion() {
        let filename = "/tmp/example.txt";
        let ev = Event::Drop(DropEvent::File {
            timestamp: 5,
            window_id: 3,
            filename: filename.into(),
        });
        let owned = ev.to_ll_owned().expect("owned drop file convertible");
        let back = Event::from_ll(owned.event);
        match back {
            Event::Drop(DropEvent::File {
                timestamp,
                window_id,
                filename: f,
            }) => {
                assert_eq!(timestamp, 5);
                assert_eq!(window_id, 3);
                assert_eq!(f, filename);
            }
            _ => panic!("Unexpected event variant on drop file owned conversion"),
        }
    }
}
