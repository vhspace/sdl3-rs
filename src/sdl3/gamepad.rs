use libc::{c_char, c_void};

use std::convert::Into;
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::io;
use std::mem::transmute;
use std::path::Path;

#[cfg(feature = "hidapi")]
use crate::sensor::SensorType;
#[cfg(feature = "hidapi")]
use std::convert::TryInto;

use crate::common::IntegerOrSdlError;
use crate::get_error;
use crate::guid::Guid;
use crate::iostream::IOStream;
use crate::joystick::{ConnectionState, JoystickId, PowerInfo, PowerLevel};
use crate::sys;
use crate::Error;
use crate::GamepadSubsystem;

use sys::joystick::SDL_GetJoystickID;

#[derive(Debug, Clone)]
pub enum AddMappingError {
    InvalidMapping(NulError),
    InvalidFilePath(Error),
    ReadError(Error),
    SdlError(Error),
}

impl fmt::Display for AddMappingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AddMappingError::*;

        match *self {
            InvalidMapping(ref e) => write!(f, "Null error: {}", e),
            InvalidFilePath(ref value) => write!(f, "Invalid file path ({})", value),
            ReadError(ref e) => write!(f, "Read error: {}", e),
            SdlError(ref e) => write!(f, "SDL error: {}", e),
        }
    }
}

impl error::Error for AddMappingError {
    fn description(&self) -> &str {
        use self::AddMappingError::*;

        match *self {
            InvalidMapping(_) => "invalid mapping",
            InvalidFilePath(_) => "invalid file path",
            ReadError(_) => "read error",
            SdlError(ref e) => &e.0,
        }
    }
}

impl GamepadSubsystem {
    /// Retrieve the total number of attached gamepads identified by SDL.
    #[doc(alias = "SDL_GetGamepads")]
    pub fn gamepads(&self) -> Result<Vec<JoystickId>, Error> {
        let mut num_gamepads: i32 = 0;
        unsafe {
            // see: https://github.com/libsdl-org/SDL/blob/main/docs/README-migration.md#sdl_joystickh
            let gamepad_ids = sys::gamepad::SDL_GetGamepads(&mut num_gamepads);
            if gamepad_ids.is_null() {
                Err(get_error())
            } else {
                let mut instances = Vec::new();
                for i in 0..num_gamepads {
                    let id = *gamepad_ids.offset(i as isize);
                    instances.push(id);
                }
                sys::stdinc::SDL_free(gamepad_ids as *mut c_void);
                Ok(instances)
            }
        }
    }

    /// Return true if the joystick at index `joystick_id` is a game controller.
    #[inline]
    #[doc(alias = "SDL_IsGamepad")]
    pub fn is_gamepad(&self, joystick_id: JoystickId) -> bool {
        unsafe { sys::gamepad::SDL_IsGamepad(joystick_id) }
    }

    /// Return true if there's any gamepad connected
    #[inline]
    #[doc(alias = "SDL_HasGamepad")]
    pub fn has_gamepad(&self) -> bool {
        unsafe { sys::gamepad::SDL_HasGamepad() }
    }

    /// Attempt to open the gamepad at index `joystick_id` and return it.
    /// Gamepad IDs are the same as joystick IDs and the maximum number can
    /// be retrieved using the `SDL_GetJoysticks` function.
    #[doc(alias = "SDL_OpenGamepad")]
    pub fn open(&self, joystick_id: JoystickId) -> Result<Gamepad, Error> {
        let gamepad = unsafe { sys::gamepad::SDL_OpenGamepad(joystick_id) };

        if gamepad.is_null() {
            Err(get_error())
        } else {
            Ok(Gamepad {
                subsystem: self.clone(),
                raw: gamepad,
            })
        }
    }

    /// Attempt to get the opened gamepad at index `joystick_id` and return it.
    #[doc(alias = "SDL_GetGamepad")]
    pub fn get(&self, joystick_id: JoystickId) -> Result<Gamepad, Error> {
        let gamepad = unsafe { sys::gamepad::SDL_GetGamepadFromID(joystick_id) };

        if gamepad.is_null() {
            Err(get_error())
        } else {
            Ok(Gamepad {
                subsystem: self.clone(),
                raw: gamepad,
            })
        }
    }

    /// Attempt to gamepad associated with a player index and return it.
    #[doc(alias = "SDL_GetGamepadFromPlayerIndex")]
    pub fn get_from_player_index(&self, player_index: u16) -> Result<Gamepad, Error> {
        let gamepad = unsafe { sys::gamepad::SDL_GetGamepadFromPlayerIndex(player_index as i32) };

        if gamepad.is_null() {
            Err(get_error())
        } else {
            Ok(Gamepad {
                subsystem: self.clone(),
                raw: gamepad,
            })
        }
    }

    /// Return the name of the controller at index `joystick_index`.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadNameForID")]
    pub fn name_for_id(&self, joystick_id: JoystickId) -> Result<String, IntegerOrSdlError> {
        use crate::common::IntegerOrSdlError::*;
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadNameForID(joystick_id) };

        if c_str.is_null() {
            Err(SdlError(get_error()))
        } else {
            Ok(unsafe {
                CStr::from_ptr(c_str as *const _)
                    .to_str()
                    .unwrap()
                    .to_owned()
            })
        }
    }

    /// Return the implementation-dependent path of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadPathForID")]
    pub fn path_for_id(&self, joystick_id: JoystickId) -> Result<String, Error> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadPathForID(joystick_id) };
        c_str_to_string_or_err(c_str)
    }

    /// Return the player index of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadPlayerForID")]
    pub fn player_index_for_id(&self, joystick_id: JoystickId) -> Option<u16> {
        let player_index = unsafe { sys::gamepad::SDL_GetGamepadPlayerIndexForID(joystick_id) };

        if player_index == -1 {
            None
        } else {
            Some(player_index as u16)
        }
    }

    /// Return the implementation-dependent GUID of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadGUIDForID")]
    pub fn guid_for_id(&self, joystick_id: JoystickId) -> Guid {
        let guid = unsafe { sys::gamepad::SDL_GetGamepadGUIDForID(joystick_id) };
        Guid { raw: guid }
    }

    /// Return the USB vendor ID of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadVendorForID")]
    pub fn vendor_for_id(&self, joystick_id: JoystickId) -> Option<u16> {
        let vendor_id = unsafe { sys::gamepad::SDL_GetGamepadVendorForID(joystick_id) };
        if vendor_id == 0 {
            None
        } else {
            Some(vendor_id)
        }
    }

    /// Return the USB product ID of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadProductForID")]
    pub fn product_for_id(&self, joystick_id: JoystickId) -> Option<u16> {
        let product_id = unsafe { sys::gamepad::SDL_GetGamepadProductForID(joystick_id) };
        if product_id == 0 {
            None
        } else {
            Some(product_id)
        }
    }

    /// Return the product version of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadProductForID")]
    pub fn product_version_for_id(&self, joystick_id: JoystickId) -> Option<u16> {
        let version = unsafe { sys::gamepad::SDL_GetGamepadProductVersionForID(joystick_id) };
        if version == 0 {
            None
        } else {
            Some(version)
        }
    }

    /// Return the type of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadTypeForID")]
    pub fn type_for_id(&self, joystick_id: JoystickId) -> GamepadType {
        let raw_type = unsafe { sys::gamepad::SDL_GetGamepadTypeForID(joystick_id) };
        GamepadType::from_ll(raw_type)
    }

    /// Return the type of a gamepad, ignoring any mapping override.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetRealGamepadTypeForID")]
    pub fn real_type_for_id(&self, joystick_id: JoystickId) -> GamepadType {
        let raw_type = unsafe { sys::gamepad::SDL_GetRealGamepadTypeForID(joystick_id) };
        GamepadType::from_ll(raw_type)
    }

    /// Return the mapping of a gamepad.
    /// This can be called before any gamepads are opened.
    #[doc(alias = "SDL_GetGamepadMappingForID")]
    pub fn mapping_for_id(&self, joystick_id: JoystickId) -> Option<String> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadMappingForID(joystick_id) };
        c_str_to_string_or_none(c_str)
    }

    #[doc(alias = "SDL_SetGamepadMapping")]
    pub fn set_mapping(
        &mut self,
        joystick_id: JoystickId,
        mapping: &str,
    ) -> Result<(), AddMappingError> {
        use self::AddMappingError::*;
        let mapping = match CString::new(mapping) {
            Ok(s) => s,
            Err(err) => return Err(InvalidMapping(err)),
        };

        let result = unsafe {
            sys::gamepad::SDL_SetGamepadMapping(joystick_id, mapping.as_ptr() as *const c_char)
        };

        if !result {
            Err(SdlError(get_error()))
        } else {
            Ok(())
        }
    }

    /// If state is `true` controller events are processed, otherwise
    /// they're ignored.
    #[doc(alias = "SDL_SetGamepadEventsEnabled")]
    pub fn set_events_processing_state(&self, state: bool) {
        unsafe { sys::gamepad::SDL_SetGamepadEventsEnabled(state) };
    }

    /// Return `true` if controller events are processed.
    #[doc(alias = "SDL_GamepadEventsEnabled")]
    pub fn event_processing_state(&self) -> bool {
        unsafe { sys::gamepad::SDL_GamepadEventsEnabled() }
    }

    /// Add a new controller input mapping from a mapping string.
    #[doc(alias = "SDL_AddGamepadMapping")]
    pub fn add_mapping(&self, mapping: &str) -> Result<MappingStatus, AddMappingError> {
        use self::AddMappingError::*;
        let mapping = match CString::new(mapping) {
            Ok(s) => s,
            Err(err) => return Err(InvalidMapping(err)),
        };

        let result =
            unsafe { sys::gamepad::SDL_AddGamepadMapping(mapping.as_ptr() as *const c_char) };

        match result {
            1 => Ok(MappingStatus::Added),
            0 => Ok(MappingStatus::Updated),
            _ => Err(SdlError(get_error())),
        }
    }

    /// Load controller input mappings from a file.
    pub fn load_mappings<P: AsRef<Path>>(&self, path: P) -> Result<i32, AddMappingError> {
        use self::AddMappingError::*;

        let rw = IOStream::from_file(path, "r").map_err(InvalidFilePath)?;
        self.load_mappings_from_rw(rw)
    }

    /// Load controller input mappings from a [`Read`](std::io::Read) object.
    pub fn load_mappings_from_read<R: io::Read>(
        &self,
        read: &mut R,
    ) -> Result<i32, AddMappingError> {
        use self::AddMappingError::*;

        let mut buffer = Vec::with_capacity(1024);
        let rw = IOStream::from_read(read, &mut buffer).map_err(ReadError)?;
        self.load_mappings_from_rw(rw)
    }

    /// Load controller input mappings from an SDL [`IOStream`] object.
    #[doc(alias = "SDL_AddGamepadMappingsFromIO")]
    pub fn load_mappings_from_rw(&self, rw: IOStream<'_>) -> Result<i32, AddMappingError> {
        use self::AddMappingError::*;

        let result = unsafe { sys::gamepad::SDL_AddGamepadMappingsFromIO(rw.raw(), false) };
        match result {
            -1 => Err(SdlError(get_error())),
            _ => Ok(result),
        }
    }

    #[doc(alias = "SDL_GetGamepadMappingForGUID")]
    pub fn mapping_for_guid(&self, guid: Guid) -> Result<String, Error> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadMappingForGUID(guid.raw()) };

        c_str_to_string_or_err(c_str)
    }

    #[doc(alias = "SDL_ReloadGamepadMappings")]
    pub fn reload_mappings(&self) -> Result<(), Error> {
        let res = unsafe { sys::gamepad::SDL_ReloadGamepadMappings() };
        if !res {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    #[doc(alias = "SDL_GetGamepadMappings")]
    pub fn mappings(&self) -> Result<Vec<String>, Error> {
        let mut num_mappings: i32 = 0;
        unsafe {
            let raw_mappings = sys::gamepad::SDL_GetGamepadMappings(&mut num_mappings);
            if raw_mappings.is_null() {
                Err(get_error())
            } else {
                let mut mappings = Vec::new();
                for i in 0..num_mappings {
                    let mapping = *raw_mappings.offset(i as isize);
                    mappings.push(c_str_to_string(mapping));
                }

                sys::stdinc::SDL_free(raw_mappings as *mut c_void);
                Ok(mappings)
            }
        }
    }

    #[inline]
    /// Force controller update when not using the event loop
    #[doc(alias = "SDL_UpdateGamepads")]
    pub fn update(&self) {
        unsafe { sys::gamepad::SDL_UpdateGamepads() };
    }

    /// Return the label of a button on a gamepad
    #[doc(alias = "SDL_GetGamepadButtonLabelForType")]
    pub fn button_label_for_gamepad_type(
        &self,
        gamepad_type: GamepadType,
        button: Button,
    ) -> ButtonLabel {
        let raw_gamepad_type: sys::gamepad::SDL_GamepadType;
        unsafe {
            raw_gamepad_type = transmute(gamepad_type);
        }

        let raw_button: sys::gamepad::SDL_GamepadButton;
        unsafe {
            raw_button = transmute(button);
        }

        let raw =
            unsafe { sys::gamepad::SDL_GetGamepadButtonLabelForType(raw_gamepad_type, raw_button) };
        unsafe { transmute(raw) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(i32)]
pub enum GamepadType {
    Unknown = sys::gamepad::SDL_GamepadType::UNKNOWN.0 as i32,
    Standard = sys::gamepad::SDL_GamepadType::STANDARD.0 as i32,
    Xbox360 = sys::gamepad::SDL_GamepadType::XBOX360.0 as i32,
    XboxOne = sys::gamepad::SDL_GamepadType::XBOXONE.0 as i32,
    PS3 = sys::gamepad::SDL_GamepadType::PS3.0 as i32,
    PS4 = sys::gamepad::SDL_GamepadType::PS4.0 as i32,
    PS5 = sys::gamepad::SDL_GamepadType::PS5.0 as i32,
    NintendoSwitchPro = sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_PRO.0 as i32,
    NintendoSwitchJoyconLeft = sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_LEFT.0 as i32,
    NintendoSwitchJoyconRight =
        sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_RIGHT.0 as i32,
    NintendoSwitchJoyconPair = sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_PAIR.0 as i32,
}

impl GamepadType {
    /// Return the GamepadType from a string description.
    #[doc(alias = "SDL_GetGamepadTypeFromString")]
    pub fn from_string(type_str: &str) -> GamepadType {
        let raw = match CString::new(type_str) {
            Ok(type_str) => unsafe {
                sys::gamepad::SDL_GetGamepadTypeFromString(type_str.as_ptr() as *const c_char)
            },
            // string contains a nul byte - it won't match anything.
            Err(_) => sys::gamepad::SDL_GamepadType::UNKNOWN,
        };
        unsafe { transmute(raw) }
    }

    /// Return a string for a given GamepadType
    #[doc(alias = "SDL_GetGamepadStringForType")]
    pub fn string(self) -> String {
        let raw_type: sys::gamepad::SDL_GamepadType;
        unsafe {
            raw_type = transmute(self);
        }
        let string = unsafe { sys::gamepad::SDL_GetGamepadStringForType(raw_type) };
        c_str_to_string(string)
    }

    pub fn from_ll(bitflags: sys::gamepad::SDL_GamepadType) -> GamepadType {
        match bitflags {
            sys::gamepad::SDL_GamepadType::UNKNOWN => GamepadType::Unknown,
            sys::gamepad::SDL_GamepadType::STANDARD => GamepadType::Standard,
            sys::gamepad::SDL_GamepadType::XBOX360 => GamepadType::Xbox360,
            sys::gamepad::SDL_GamepadType::XBOXONE => GamepadType::XboxOne,
            sys::gamepad::SDL_GamepadType::PS3 => GamepadType::PS3,
            sys::gamepad::SDL_GamepadType::PS4 => GamepadType::PS4,
            sys::gamepad::SDL_GamepadType::PS5 => GamepadType::PS5,
            sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_PRO => GamepadType::NintendoSwitchPro,
            sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_LEFT => {
                GamepadType::NintendoSwitchJoyconLeft
            }
            sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_RIGHT => {
                GamepadType::NintendoSwitchJoyconRight
            }
            sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_PAIR => {
                GamepadType::NintendoSwitchJoyconPair
            }

            _ => GamepadType::Unknown,
        }
    }

    pub fn to_ll(self) -> sys::gamepad::SDL_GamepadType {
        match self {
            GamepadType::Unknown => sys::gamepad::SDL_GamepadType::UNKNOWN,
            GamepadType::Standard => sys::gamepad::SDL_GamepadType::STANDARD,
            GamepadType::Xbox360 => sys::gamepad::SDL_GamepadType::XBOX360,
            GamepadType::XboxOne => sys::gamepad::SDL_GamepadType::XBOXONE,
            GamepadType::PS3 => sys::gamepad::SDL_GamepadType::PS3,
            GamepadType::PS4 => sys::gamepad::SDL_GamepadType::PS4,
            GamepadType::PS5 => sys::gamepad::SDL_GamepadType::PS5,
            GamepadType::NintendoSwitchPro => sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_PRO,
            GamepadType::NintendoSwitchJoyconLeft => {
                sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_LEFT
            }
            GamepadType::NintendoSwitchJoyconRight => {
                sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_RIGHT
            }
            GamepadType::NintendoSwitchJoyconPair => {
                sys::gamepad::SDL_GamepadType::NINTENDO_SWITCH_JOYCON_PAIR
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(i32)]
pub enum Axis {
    LeftX = sys::gamepad::SDL_GAMEPAD_AXIS_LEFTX.0,
    LeftY = sys::gamepad::SDL_GAMEPAD_AXIS_LEFTY.0,
    RightX = sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTX.0,
    RightY = sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTY.0,
    TriggerLeft = sys::gamepad::SDL_GAMEPAD_AXIS_LEFT_TRIGGER.0,
    TriggerRight = sys::gamepad::SDL_GAMEPAD_AXIS_RIGHT_TRIGGER.0,
}

impl Axis {
    /// Return the Axis from a string description in the same format
    /// used by the game controller mapping strings.
    #[doc(alias = "SDL_GetGamepadAxisFromString")]
    pub fn from_string(axis: &str) -> Option<Axis> {
        let id = match CString::new(axis) {
            Ok(axis) => unsafe {
                sys::gamepad::SDL_GetGamepadAxisFromString(axis.as_ptr() as *const c_char)
            },
            // string contains a nul byte - it won't match anything.
            Err(_) => sys::gamepad::SDL_GAMEPAD_AXIS_INVALID,
        };

        Axis::from_ll(id)
    }

    /// Return a string for a given axis in the same format using by
    /// the game controller mapping strings
    #[doc(alias = "SDL_GetGamepadStringForAxis")]
    pub fn string(self) -> String {
        let axis: sys::gamepad::SDL_GamepadAxis;
        unsafe {
            axis = transmute(self);
        }

        let string = unsafe { sys::gamepad::SDL_GetGamepadStringForAxis(axis) };

        c_str_to_string(string)
    }

    pub fn from_ll(bitflags: sys::gamepad::SDL_GamepadAxis) -> Option<Axis> {
        Some(match bitflags {
            sys::gamepad::SDL_GAMEPAD_AXIS_INVALID => return None,
            sys::gamepad::SDL_GAMEPAD_AXIS_LEFTX => Axis::LeftX,
            sys::gamepad::SDL_GAMEPAD_AXIS_LEFTY => Axis::LeftY,
            sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTX => Axis::RightX,
            sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTY => Axis::RightY,
            sys::gamepad::SDL_GAMEPAD_AXIS_LEFT_TRIGGER => Axis::TriggerLeft,
            sys::gamepad::SDL_GAMEPAD_AXIS_RIGHT_TRIGGER => Axis::TriggerRight,
            _ => return None,
        })
    }

    pub fn to_ll(self) -> sys::gamepad::SDL_GamepadAxis {
        match self {
            Axis::LeftX => sys::gamepad::SDL_GAMEPAD_AXIS_LEFTX,
            Axis::LeftY => sys::gamepad::SDL_GAMEPAD_AXIS_LEFTY,
            Axis::RightX => sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTX,
            Axis::RightY => sys::gamepad::SDL_GAMEPAD_AXIS_RIGHTY,
            Axis::TriggerLeft => sys::gamepad::SDL_GAMEPAD_AXIS_LEFT_TRIGGER,
            Axis::TriggerRight => sys::gamepad::SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
        }
    }
}

impl From<Axis> for u8 {
    fn from(axis: Axis) -> u8 {
        axis as u8
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(i32)]
pub enum Button {
    North = sys::gamepad::SDL_GAMEPAD_BUTTON_NORTH.0,
    East = sys::gamepad::SDL_GAMEPAD_BUTTON_EAST.0,
    South = sys::gamepad::SDL_GAMEPAD_BUTTON_SOUTH.0,
    West = sys::gamepad::SDL_GAMEPAD_BUTTON_WEST.0,
    Back = sys::gamepad::SDL_GAMEPAD_BUTTON_BACK.0,
    Guide = sys::gamepad::SDL_GAMEPAD_BUTTON_GUIDE.0,
    Start = sys::gamepad::SDL_GAMEPAD_BUTTON_START.0,
    LeftStick = sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_STICK.0,
    RightStick = sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_STICK.0,
    LeftShoulder = sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_SHOULDER.0,
    RightShoulder = sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER.0,
    DPadUp = sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_UP.0,
    DPadDown = sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_DOWN.0,
    DPadLeft = sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_LEFT.0,
    DPadRight = sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_RIGHT.0,
    Misc1 = sys::gamepad::SDL_GAMEPAD_BUTTON_MISC1.0,
    Misc2 = sys::gamepad::SDL_GAMEPAD_BUTTON_MISC2.0,
    Misc3 = sys::gamepad::SDL_GAMEPAD_BUTTON_MISC3.0,
    Misc4 = sys::gamepad::SDL_GAMEPAD_BUTTON_MISC4.0,
    Misc5 = sys::gamepad::SDL_GAMEPAD_BUTTON_MISC5.0,
    RightPaddle1 = sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1.0,
    LeftPaddle1 = sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE1.0,
    RightPaddle2 = sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2.0,
    LeftPaddle2 = sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE2.0,
    Touchpad = sys::gamepad::SDL_GAMEPAD_BUTTON_TOUCHPAD.0,
}

impl Button {
    /// Return the Button from a string description in the same format
    /// used by the game controller mapping strings.
    #[doc(alias = "SDL_GetGamepadButtonFromString")]
    pub fn from_string(button: &str) -> Option<Button> {
        let id = match CString::new(button) {
            Ok(button) => unsafe {
                sys::gamepad::SDL_GetGamepadButtonFromString(button.as_ptr() as *const c_char)
            },
            // string contains a nul byte - it won't match anything.
            Err(_) => sys::gamepad::SDL_GAMEPAD_BUTTON_INVALID,
        };

        Button::from_ll(id)
    }

    /// Return a string for a given button in the same format using by
    /// the game controller mapping strings
    #[doc(alias = "SDL_GetGamepadStringForButton")]
    pub fn string(self) -> String {
        let button: sys::gamepad::SDL_GamepadButton;
        unsafe {
            button = transmute(self);
        }

        let string = unsafe { sys::gamepad::SDL_GetGamepadStringForButton(button) };

        c_str_to_string(string)
    }

    pub fn from_ll(bitflags: sys::gamepad::SDL_GamepadButton) -> Option<Button> {
        Some(match bitflags {
            sys::gamepad::SDL_GAMEPAD_BUTTON_INVALID => return None,
            sys::gamepad::SDL_GAMEPAD_BUTTON_NORTH => Button::North,
            sys::gamepad::SDL_GAMEPAD_BUTTON_EAST => Button::East,
            sys::gamepad::SDL_GAMEPAD_BUTTON_SOUTH => Button::South,
            sys::gamepad::SDL_GAMEPAD_BUTTON_WEST => Button::West,
            sys::gamepad::SDL_GAMEPAD_BUTTON_BACK => Button::Back,
            sys::gamepad::SDL_GAMEPAD_BUTTON_GUIDE => Button::Guide,
            sys::gamepad::SDL_GAMEPAD_BUTTON_START => Button::Start,
            sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_STICK => Button::LeftStick,
            sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_STICK => Button::RightStick,
            sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_SHOULDER => Button::LeftShoulder,
            sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER => Button::RightShoulder,
            sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_UP => Button::DPadUp,
            sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_DOWN => Button::DPadDown,
            sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_LEFT => Button::DPadLeft,
            sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_RIGHT => Button::DPadRight,
            sys::gamepad::SDL_GAMEPAD_BUTTON_MISC1 => Button::Misc1,
            sys::gamepad::SDL_GAMEPAD_BUTTON_MISC2 => Button::Misc2,
            sys::gamepad::SDL_GAMEPAD_BUTTON_MISC3 => Button::Misc3,
            sys::gamepad::SDL_GAMEPAD_BUTTON_MISC4 => Button::Misc4,
            sys::gamepad::SDL_GAMEPAD_BUTTON_MISC5 => Button::Misc5,
            sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE1 => Button::LeftPaddle1,
            sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1 => Button::RightPaddle1,
            sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE2 => Button::LeftPaddle2,
            sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2 => Button::RightPaddle2,
            sys::gamepad::SDL_GAMEPAD_BUTTON_TOUCHPAD => Button::Touchpad,
            _ => return None,
        })
    }

    pub fn to_ll(self) -> sys::gamepad::SDL_GamepadButton {
        match self {
            Button::North => sys::gamepad::SDL_GAMEPAD_BUTTON_NORTH,
            Button::East => sys::gamepad::SDL_GAMEPAD_BUTTON_EAST,
            Button::South => sys::gamepad::SDL_GAMEPAD_BUTTON_SOUTH,
            Button::West => sys::gamepad::SDL_GAMEPAD_BUTTON_WEST,
            Button::Back => sys::gamepad::SDL_GAMEPAD_BUTTON_BACK,
            Button::Guide => sys::gamepad::SDL_GAMEPAD_BUTTON_GUIDE,
            Button::Start => sys::gamepad::SDL_GAMEPAD_BUTTON_START,
            Button::LeftStick => sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_STICK,
            Button::RightStick => sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_STICK,
            Button::LeftShoulder => sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_SHOULDER,
            Button::RightShoulder => sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER,
            Button::DPadUp => sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_UP,
            Button::DPadDown => sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_DOWN,
            Button::DPadLeft => sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_LEFT,
            Button::DPadRight => sys::gamepad::SDL_GAMEPAD_BUTTON_DPAD_RIGHT,
            Button::Misc1 => sys::gamepad::SDL_GAMEPAD_BUTTON_MISC1,
            Button::Misc2 => sys::gamepad::SDL_GAMEPAD_BUTTON_MISC2,
            Button::Misc3 => sys::gamepad::SDL_GAMEPAD_BUTTON_MISC3,
            Button::Misc4 => sys::gamepad::SDL_GAMEPAD_BUTTON_MISC4,
            Button::Misc5 => sys::gamepad::SDL_GAMEPAD_BUTTON_MISC5,
            Button::LeftPaddle1 => sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE1,
            Button::RightPaddle1 => sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1,
            Button::LeftPaddle2 => sys::gamepad::SDL_GAMEPAD_BUTTON_LEFT_PADDLE2,
            Button::RightPaddle2 => sys::gamepad::SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2,
            Button::Touchpad => sys::gamepad::SDL_GAMEPAD_BUTTON_TOUCHPAD,
        }
    }
}

impl From<Button> for u8 {
    fn from(button: Button) -> u8 {
        button as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ButtonLabel {
    Unknown = sys::gamepad::SDL_GamepadButtonLabel::UNKNOWN.0,
    A = sys::gamepad::SDL_GamepadButtonLabel::A.0,
    B = sys::gamepad::SDL_GamepadButtonLabel::B.0,
    X = sys::gamepad::SDL_GamepadButtonLabel::X.0,
    Y = sys::gamepad::SDL_GamepadButtonLabel::Y.0,
    Cross = sys::gamepad::SDL_GamepadButtonLabel::CROSS.0,
    Circle = sys::gamepad::SDL_GamepadButtonLabel::CIRCLE.0,
    Square = sys::gamepad::SDL_GamepadButtonLabel::SQUARE.0,
    Triangle = sys::gamepad::SDL_GamepadButtonLabel::TRIANGLE.0,
}

impl ButtonLabel {
    pub fn from_ll(bitflags: sys::gamepad::SDL_GamepadButtonLabel) -> Option<ButtonLabel> {
        Some(match bitflags {
            sys::gamepad::SDL_GamepadButtonLabel::UNKNOWN => ButtonLabel::Unknown,
            sys::gamepad::SDL_GamepadButtonLabel::A => ButtonLabel::A,
            sys::gamepad::SDL_GamepadButtonLabel::B => ButtonLabel::B,
            sys::gamepad::SDL_GamepadButtonLabel::X => ButtonLabel::X,
            sys::gamepad::SDL_GamepadButtonLabel::Y => ButtonLabel::Y,
            sys::gamepad::SDL_GamepadButtonLabel::CROSS => ButtonLabel::Cross,
            sys::gamepad::SDL_GamepadButtonLabel::CIRCLE => ButtonLabel::Circle,
            sys::gamepad::SDL_GamepadButtonLabel::SQUARE => ButtonLabel::Square,
            sys::gamepad::SDL_GamepadButtonLabel::TRIANGLE => ButtonLabel::Triangle,
            _ => return None,
        })
    }

    pub fn to_ll(self) -> sys::gamepad::SDL_GamepadButtonLabel {
        match self {
            ButtonLabel::Unknown => sys::gamepad::SDL_GamepadButtonLabel::UNKNOWN,
            ButtonLabel::A => sys::gamepad::SDL_GamepadButtonLabel::A,
            ButtonLabel::B => sys::gamepad::SDL_GamepadButtonLabel::B,
            ButtonLabel::X => sys::gamepad::SDL_GamepadButtonLabel::X,
            ButtonLabel::Y => sys::gamepad::SDL_GamepadButtonLabel::Y,
            ButtonLabel::Cross => sys::gamepad::SDL_GamepadButtonLabel::CROSS,
            ButtonLabel::Circle => sys::gamepad::SDL_GamepadButtonLabel::CIRCLE,
            ButtonLabel::Square => sys::gamepad::SDL_GamepadButtonLabel::SQUARE,
            ButtonLabel::Triangle => sys::gamepad::SDL_GamepadButtonLabel::TRIANGLE,
        }
    }
}

/// Possible return values for `add_mapping`
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MappingStatus {
    Added = 1,
    Updated = 0,
}

/// Wrapper around the `SDL_Gamepad` object
pub struct Gamepad {
    subsystem: GamepadSubsystem,
    raw: *mut sys::gamepad::SDL_Gamepad,
}

impl Gamepad {
    #[inline]
    pub fn subsystem(&self) -> &GamepadSubsystem {
        &self.subsystem
    }

    /// Return the name of the controller or an empty string if no
    /// name is found.
    #[doc(alias = "SDL_GetGamepadName")]
    pub fn name(&self) -> Option<String> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadName(self.raw) };
        c_str_to_string_or_none(c_str)
    }

    /// Return the implementation-dependant path for an opened gamepad.
    #[doc(alias = "SDL_GetGamepadPath")]
    pub fn path(&self) -> Option<String> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadPath(self.raw) };
        c_str_to_string_or_none(c_str)
    }

    /// Return the type of an opened gamepad.
    #[doc(alias = "SDL_GetGamepadType")]
    pub fn r#type(&self) -> GamepadType {
        let raw_type = unsafe { sys::gamepad::SDL_GetGamepadType(self.raw) };
        GamepadType::from_ll(raw_type)
    }

    /// Return the type of an opened gamepad, ignoring any mapping override.
    #[doc(alias = "SDL_GetRealGamepadType")]
    pub fn real_type(&self) -> GamepadType {
        let raw_type = unsafe { sys::gamepad::SDL_GetRealGamepadType(self.raw) };
        GamepadType::from_ll(raw_type)
    }

    /// Return the player index of an opened gamepad.
    #[doc(alias = "SDL_GetGamepadPlayerIndex")]
    pub fn player_index(&self) -> Option<u16> {
        let c_int = unsafe { sys::gamepad::SDL_GetGamepadPlayerIndex(self.raw) };
        if c_int <= -1 {
            None
        } else {
            Some(c_int as u16)
        }
    }

    /// Set the player index of an opened gamepad.
    #[doc(alias = "SDL_SetGamepadPlayerIndex")]
    pub fn set_player_index(&self, player_index: u16) -> Result<(), Error> {
        let result =
            unsafe { sys::gamepad::SDL_SetGamepadPlayerIndex(self.raw, player_index as i32) };
        if !result {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Unset the player index of an opened gamepad.
    #[doc(alias = "SDL_SetGamepadPlayerIndex")]
    pub fn unset_player_index(&self) -> Result<(), Error> {
        let result = unsafe { sys::gamepad::SDL_SetGamepadPlayerIndex(self.raw, -1) };
        if !result {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Return the USB vendor ID of an opened gamepad, if available.
    #[doc(alias = "SDL_GetGamepadVendor")]
    pub fn vendor_id(&self) -> Option<u16> {
        let vendor_id = unsafe { sys::gamepad::SDL_GetGamepadVendor(self.raw) };
        if vendor_id == 0 {
            None
        } else {
            Some(vendor_id)
        }
    }

    /// Return the USB product ID of an opened gamepad, if available.
    #[doc(alias = "SDL_GetGamepadProduct")]
    pub fn product_id(&self) -> Option<u16> {
        let product_id = unsafe { sys::gamepad::SDL_GetGamepadProduct(self.raw) };
        if product_id == 0 {
            None
        } else {
            Some(product_id)
        }
    }

    /// Return the product version of an opened gamepad, if available.
    #[doc(alias = "SDL_GetGamepadProductVersion")]
    pub fn product_version(&self) -> Option<u16> {
        let product_version = unsafe { sys::gamepad::SDL_GetGamepadProductVersion(self.raw) };
        if product_version == 0 {
            None
        } else {
            Some(product_version)
        }
    }

    /// Return the firmware version of an opened gamepad, if available.
    #[doc(alias = "SDL_GetGamepadFirmwareVersion")]
    pub fn firmware_version(&self) -> Option<u16> {
        let firmware_version = unsafe { sys::gamepad::SDL_GetGamepadFirmwareVersion(self.raw) };
        if firmware_version == 0 {
            None
        } else {
            Some(firmware_version)
        }
    }

    /// Return the serial number of an opened gamepad, if available.
    #[doc(alias = "SDL_GetGamepadSerial")]
    pub fn serial_number(&self) -> Option<String> {
        let c_str = unsafe { sys::gamepad::SDL_GetGamepadSerial(self.raw) };
        c_str_to_string_or_none(c_str)
    }

    /// Return the connection state of a gamepad
    #[doc(alias = "SDL_GetGamepadConnectionState")]
    pub fn connection_state(&self) -> Result<ConnectionState, Error> {
        let raw = unsafe { sys::gamepad::SDL_GetGamepadConnectionState(self.raw) };
        if raw == sys::joystick::SDL_JoystickConnectionState::INVALID {
            Err(get_error())
        } else {
            Ok(ConnectionState::from_ll(raw))
        }
    }

    /// Return the battery state of a gamepad
    #[doc(alias = "SDL_GetGamepadPowerInfo")]
    pub fn power_info(&self) -> PowerInfo {
        let mut percentage: i32 = 0;
        let result = unsafe { sys::gamepad::SDL_GetGamepadPowerInfo(self.raw, &mut percentage) };
        let state = PowerLevel::from_ll(result);
        PowerInfo { state, percentage }
    }

    /// Return a String describing the controller's button and axis
    /// mappings
    #[doc(alias = "SDL_GetGamepadMapping")]
    pub fn mapping(&self) -> Option<String> {
        let raw_mapping = unsafe { sys::gamepad::SDL_GetGamepadMapping(self.raw) };
        let mapping = c_str_to_string_or_none(raw_mapping);
        unsafe { sys::stdinc::SDL_free(raw_mapping as *mut c_void) };
        mapping
    }

    #[doc(alias = "SDL_SetGamepadMapping")]
    pub fn set_mapping(&mut self, mapping: &str) -> Result<(), AddMappingError> {
        use self::AddMappingError::*;
        let mapping = match CString::new(mapping) {
            Ok(s) => s,
            Err(err) => return Err(InvalidMapping(err)),
        };

        let joystick_id = match self.id() {
            Ok(id) => id,
            Err(err) => return Err(SdlError(err)),
        };

        let result = unsafe {
            sys::gamepad::SDL_SetGamepadMapping(joystick_id, mapping.as_ptr() as *const c_char)
        };

        if !result {
            Err(SdlError(get_error()))
        } else {
            Ok(())
        }
    }

    /// Return true if the controller has been opened and currently
    /// connected.
    #[doc(alias = "SDL_GamepadConnected")]
    pub fn connected(&self) -> bool {
        unsafe { sys::gamepad::SDL_GamepadConnected(self.raw) }
    }

    /// Return the joystick id of an opened gamepad.
    #[doc(alias = "SDL_GetGamepadID")]
    pub fn id(&self) -> Result<JoystickId, Error> {
        let result = unsafe { sys::gamepad::SDL_GetGamepadID(self.raw) };
        if result == 0 {
            Err(get_error())
        } else {
            Ok(result)
        }
    }

    /// Return whether the gamepad has a given axis.
    #[doc(alias = "SDL_GamepadHasAxis")]
    pub fn has_axis(&self, axis: Axis) -> bool {
        let raw_axis: sys::gamepad::SDL_GamepadAxis;
        unsafe {
            raw_axis = transmute(axis);
        }
        unsafe { sys::gamepad::SDL_GamepadHasAxis(self.raw, raw_axis) }
    }

    /// Get the position of the given `axis`
    #[doc(alias = "SDL_GetGamepadAxis")]
    pub fn axis(&self, axis: Axis) -> i16 {
        // This interface is a bit messed up: 0 is a valid position
        // but can also mean that an error occured.
        // Fortunately, an error can only occur if the controller pointer is NULL.
        // There should be no apparent reason for this to change in the future.

        let raw_axis: sys::gamepad::SDL_GamepadAxis;
        unsafe {
            raw_axis = transmute(axis);
        }

        unsafe { sys::gamepad::SDL_GetGamepadAxis(self.raw, raw_axis) }
    }

    /// Return whether the gamepad has a given button.
    #[doc(alias = "SDL_GamepadHasButton")]
    pub fn has_button(&self, button: Button) -> bool {
        let raw_button: sys::gamepad::SDL_GamepadButton;
        unsafe {
            raw_button = transmute(button);
        }
        unsafe { sys::gamepad::SDL_GamepadHasButton(self.raw, raw_button) }
    }

    /// Returns `true` if `button` is pressed.
    #[doc(alias = "SDL_GetGamepadButton")]
    pub fn button(&self, button: Button) -> bool {
        // This interface is a bit messed up: 0 is a valid position
        // but can also mean that an error occured.
        // Fortunately, an error can only occur if the controller pointer is NULL.
        // There should be no apparent reason for this to change in the future.

        let raw_button: sys::gamepad::SDL_GamepadButton;
        unsafe {
            raw_button = transmute(button);
        }

        unsafe { sys::gamepad::SDL_GetGamepadButton(self.raw, raw_button) }
    }

    /// Return the label of a button on this gamepad
    #[doc(alias = "SDL_GetGamepadButtonLabel")]
    pub fn button_label_for_gamepad_type(&self, button: Button) -> ButtonLabel {
        let raw_button: sys::gamepad::SDL_GamepadButton;
        unsafe {
            raw_button = transmute(button);
        }

        let raw = unsafe { sys::gamepad::SDL_GetGamepadButtonLabel(self.raw, raw_button) };
        unsafe { transmute(raw) }
    }

    /// Return the number of touchpads on this gamepad
    #[doc(alias = "SDL_GetNumGamepadTouchpads")]
    pub fn touchpads_count(&self) -> u16 {
        unsafe { sys::gamepad::SDL_GetNumGamepadTouchpads(self.raw) as u16 }
    }

    /// Return the number of supported simultaneous fingers on a touchpad on this gamepad
    #[doc(alias = "SDL_GetNumGamepadTouchpadFingers")]
    pub fn supported_touchpad_fingers(&self, touchpad: u16) -> u16 {
        unsafe { sys::gamepad::SDL_GetNumGamepadTouchpadFingers(self.raw, touchpad as i32) as u16 }
    }

    /// Set the rumble motors to their specified intensities, if supported.
    /// Automatically resets back to zero after `duration_ms` milliseconds have passed.
    ///
    /// # Notes
    ///
    /// The value range for the intensities is 0 to 0xFFFF.
    ///
    /// Do *not* use `std::u32::MAX` or similar for `duration_ms` if you want
    /// the rumble effect to keep playing for a long time, as this results in
    /// the effect ending immediately after starting due to an overflow.
    /// Use some smaller, "huge enough" number instead.
    #[doc(alias = "SDL_RumbleGamepad")]
    pub fn set_rumble(
        &mut self,
        low_frequency_rumble: u16,
        high_frequency_rumble: u16,
        duration_ms: u32,
    ) -> Result<(), IntegerOrSdlError> {
        let result = unsafe {
            sys::gamepad::SDL_RumbleGamepad(
                self.raw,
                low_frequency_rumble,
                high_frequency_rumble,
                duration_ms,
            )
        };

        if !result {
            Err(IntegerOrSdlError::SdlError(get_error()))
        } else {
            Ok(())
        }
    }

    /// Start a rumble effect in the game controller's triggers.
    #[doc(alias = "SDL_RumbleGamepadTriggers")]
    pub fn set_rumble_triggers(
        &mut self,
        left_rumble: u16,
        right_rumble: u16,
        duration_ms: u32,
    ) -> Result<(), IntegerOrSdlError> {
        let result = unsafe {
            sys::gamepad::SDL_RumbleGamepadTriggers(
                self.raw,
                left_rumble,
                right_rumble,
                duration_ms,
            )
        };

        if !result {
            Err(IntegerOrSdlError::SdlError(get_error()))
        } else {
            Ok(())
        }
    }

    /// Query whether a game controller has a RGB LED.
    #[doc(alias = "SDL_PROP_JOYSTICK_CAP_RGB_LED_BOOLEAN")]
    pub unsafe fn has_led(&self) -> bool {
        let props = sys::gamepad::SDL_GetGamepadProperties(self.raw);
        sys::properties::SDL_GetBooleanProperty(
            props,
            sys::gamepad::SDL_PROP_GAMEPAD_CAP_RGB_LED_BOOLEAN,
            false,
        )
    }

    /// Query whether a game controller has rumble support.
    #[doc(alias = "SDL_PROP_GAMEPAD_CAP_RUMBLE_BOOLEAN")]
    pub unsafe fn has_rumble(&self) -> bool {
        let props = sys::gamepad::SDL_GetGamepadProperties(self.raw);
        sys::properties::SDL_GetBooleanProperty(
            props,
            sys::gamepad::SDL_PROP_GAMEPAD_CAP_RUMBLE_BOOLEAN,
            false,
        )
    }

    /// Query whether a game controller has rumble support on triggers.
    #[doc(alias = "SDL_PROP_GAMEPAD_CAP_TRIGGER_RUMBLE_BOOLEAN")]
    pub unsafe fn has_rumble_triggers(&self) -> bool {
        let props = sys::gamepad::SDL_GetGamepadProperties(self.raw);
        sys::properties::SDL_GetBooleanProperty(
            props,
            sys::gamepad::SDL_PROP_GAMEPAD_CAP_TRIGGER_RUMBLE_BOOLEAN,
            false,
        )
    }

    /// Update a game controller's LED color.
    #[doc(alias = "SDL_SetGamepadLED")]
    pub fn set_led(&mut self, red: u8, green: u8, blue: u8) -> Result<(), IntegerOrSdlError> {
        let result = unsafe { sys::gamepad::SDL_SetGamepadLED(self.raw, red, green, blue) };

        if result {
            Ok(())
        } else {
            Err(IntegerOrSdlError::SdlError(get_error()))
        }
    }

    /// Send a controller specific effect packet.
    #[doc(alias = "SDL_SendGamepadEffect")]
    pub fn send_effect(&mut self, data: &[u8]) -> Result<(), Error> {
        let result = unsafe {
            sys::gamepad::SDL_SendGamepadEffect(
                self.raw,
                data.as_ptr() as *const libc::c_void,
                data.len() as i32,
            )
        };

        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

#[cfg(feature = "hidapi")]
impl Gamepad {
    #[doc(alias = "SDL_GamepadHasSensor")]
    pub unsafe fn has_sensor(&self, sensor_type: crate::sensor::SensorType) -> bool {
        unsafe { sys::gamepad::SDL_GamepadHasSensor(self.raw, sensor_type.into()) }
    }

    #[doc(alias = "SDL_GamepadSensorEnabled")]
    pub fn sensor_enabled(&self, sensor_type: crate::sensor::SensorType) -> bool {
        unsafe { sys::gamepad::SDL_GamepadSensorEnabled(self.raw, sensor_type.into()) }
    }

    #[doc(alias = "SDL_SetGamepadSensorEnabled")]
    pub fn sensor_set_enabled(
        &self,
        sensor_type: crate::sensor::SensorType,
        enabled: bool,
    ) -> Result<(), IntegerOrSdlError> {
        let result = unsafe {
            sys::gamepad::SDL_SetGamepadSensorEnabled(
                self.raw,
                sensor_type.into(),
                if enabled { true } else { false },
            )
        };

        if !result {
            Err(IntegerOrSdlError::SdlError(get_error()))
        } else {
            Ok(())
        }
    }

    /// Get the data rate (number of events per second) of a game controller sensor.
    #[doc(alias = "SDL_GetGamepadSensorDataRate")]
    pub fn sensor_get_data_rate(&self, sensor_type: SensorType) -> f32 {
        unsafe { sys::gamepad::SDL_GetGamepadSensorDataRate(self.raw, sensor_type.into()) }
    }

    /// Get data from a sensor.
    ///
    /// The number of data points depends on the sensor. Both Gyroscope and
    /// Accelerometer return 3 values, one for each axis.
    #[doc(alias = "SDL_GetGamepadSensorData")]
    pub fn sensor_get_data(
        &self,
        sensor_type: SensorType,
        data: &mut [f32],
    ) -> Result<(), IntegerOrSdlError> {
        let result = unsafe {
            sys::gamepad::SDL_GetGamepadSensorData(
                self.raw,
                sensor_type.into(),
                data.as_mut_ptr(),
                data.len().try_into().unwrap(),
            )
        };

        if !result {
            Err(IntegerOrSdlError::SdlError(get_error()))
        } else {
            Ok(())
        }
    }
}

impl Drop for Gamepad {
    #[doc(alias = "SDL_CloseGamepad")]
    fn drop(&mut self) {
        unsafe { sys::gamepad::SDL_CloseGamepad(self.raw) }
    }
}

/// Convert C string `c_str` to a String. Return an empty string if
/// `c_str` is NULL.
fn c_str_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        String::new()
    } else {
        unsafe {
            CStr::from_ptr(c_str as *const _)
                .to_str()
                .unwrap()
                .to_owned()
        }
    }
}

/// Convert C string `c_str` to a String. Return an SDL error if
/// `c_str` is NULL.
fn c_str_to_string_or_none(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(unsafe {
            CStr::from_ptr(c_str as *const _)
                .to_str()
                .unwrap()
                .to_owned()
        })
    }
}

/// Convert C string `c_str` to a String. Return an SDL error if
/// `c_str` is NULL.
fn c_str_to_string_or_err(c_str: *const c_char) -> Result<String, Error> {
    if c_str.is_null() {
        Err(get_error())
    } else {
        Ok(unsafe {
            CStr::from_ptr(c_str as *const _)
                .to_str()
                .unwrap()
                .to_owned()
        })
    }
}
