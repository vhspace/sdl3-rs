use crate::get_error;
use crate::surface::SurfaceRef;
use crate::sys;
use crate::video;
use crate::Error;
use crate::EventPump;
use std::convert::TryInto;
use std::mem::transmute;
use sys::mouse::{
    SDL_GetWindowRelativeMouseMode, SDL_MouseWheelDirection, SDL_SetWindowRelativeMouseMode,
};
use sys::video::SDL_GetWindowID;

mod relative;
pub use self::relative::RelativeMouseState;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum SystemCursor {
    Arrow = sys::mouse::SDL_SYSTEM_CURSOR_DEFAULT.0 as u32,
    IBeam = sys::mouse::SDL_SYSTEM_CURSOR_TEXT.0 as u32,
    Wait = sys::mouse::SDL_SYSTEM_CURSOR_WAIT.0 as u32,
    Crosshair = sys::mouse::SDL_SYSTEM_CURSOR_CROSSHAIR.0 as u32,
    WaitArrow = sys::mouse::SDL_SYSTEM_CURSOR_PROGRESS.0 as u32,
    SizeNWSE = sys::mouse::SDL_SYSTEM_CURSOR_NWSE_RESIZE.0 as u32,
    SizeNESW = sys::mouse::SDL_SYSTEM_CURSOR_NESW_RESIZE.0 as u32,
    SizeWE = sys::mouse::SDL_SYSTEM_CURSOR_EW_RESIZE.0 as u32,
    SizeNS = sys::mouse::SDL_SYSTEM_CURSOR_NS_RESIZE.0 as u32,
    SizeAll = sys::mouse::SDL_SYSTEM_CURSOR_MOVE.0 as u32,
    No = sys::mouse::SDL_SYSTEM_CURSOR_NOT_ALLOWED.0 as u32,
    Hand = sys::mouse::SDL_SYSTEM_CURSOR_POINTER.0 as u32,
}

pub struct Cursor {
    raw: *mut sys::mouse::SDL_Cursor,
}

impl Drop for Cursor {
    #[inline]
    #[doc(alias = "SDL_DestroyCursor")]
    fn drop(&mut self) {
        unsafe { sys::mouse::SDL_DestroyCursor(self.raw) };
    }
}

impl Cursor {
    #[doc(alias = "SDL_CreateCursor")]
    pub fn new(
        data: &[u8],
        mask: &[u8],
        width: i32,
        height: i32,
        hot_x: i32,
        hot_y: i32,
    ) -> Result<Cursor, Error> {
        unsafe {
            let raw = sys::mouse::SDL_CreateCursor(
                data.as_ptr(),
                mask.as_ptr(),
                width,
                height,
                hot_x,
                hot_y,
            );

            if raw.is_null() {
                Err(get_error())
            } else {
                Ok(Cursor { raw })
            }
        }
    }

    // TODO: figure out how to pass Surface in here correctly
    #[doc(alias = "SDL_CreateColorCursor")]
    pub fn from_surface<S: AsRef<SurfaceRef>>(
        surface: S,
        hot_x: i32,
        hot_y: i32,
    ) -> Result<Cursor, Error> {
        unsafe {
            let raw = sys::mouse::SDL_CreateColorCursor(surface.as_ref().raw(), hot_x, hot_y);

            if raw.is_null() {
                Err(get_error())
            } else {
                Ok(Cursor { raw })
            }
        }
    }

    #[doc(alias = "SDL_CreateSystemCursor")]
    pub fn from_system(cursor: SystemCursor) -> Result<Cursor, Error> {
        unsafe {
            let raw = sys::mouse::SDL_CreateSystemCursor(transmute(cursor as u32));

            if raw.is_null() {
                Err(get_error())
            } else {
                Ok(Cursor { raw })
            }
        }
    }

    #[doc(alias = "SDL_SetCursor")]
    pub fn set(&self) {
        unsafe {
            sys::mouse::SDL_SetCursor(self.raw);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseWheelDirection {
    Normal,
    Flipped,
    Unknown(u32),
}

impl From<MouseWheelDirection> for SDL_MouseWheelDirection {
    fn from(direction: MouseWheelDirection) -> SDL_MouseWheelDirection {
        match direction {
            MouseWheelDirection::Normal => sys::mouse::SDL_MOUSEWHEEL_NORMAL,
            MouseWheelDirection::Flipped => sys::mouse::SDL_MOUSEWHEEL_FLIPPED,
            MouseWheelDirection::Unknown(_) => sys::mouse::SDL_MOUSEWHEEL_NORMAL,
        }
    }
}

// 0 and 1 are not fixed values in the SDL source code.  This value is defined as an enum which is then cast to a Uint32.
// The enum in C is defined as such:

/**
 * \brief Scroll direction types for the Scroll event
 */
//typedef enum
//{
//    SDL_MOUSEWHEEL_NORMAL,    /**< The scroll direction is normal */
//    SDL_MOUSEWHEEL_FLIPPED    /**< The scroll direction is flipped / natural */
//} SDL_MouseWheelDirection;

// Since no value is given in the enum definition these values are auto assigned by the C compiler starting at 0.
// Normally I would prefer to use the enum rather than hard code what it is implied to represent however
// the mouse wheel direction value could be described equally as well by a bool, so I don't think changes
// to this enum in the C source code are going to be a problem.

impl MouseWheelDirection {
    #[inline]
    pub fn from_ll(direction: u32) -> MouseWheelDirection {
        match direction {
            0 => MouseWheelDirection::Normal,
            1 => MouseWheelDirection::Flipped,
            _ => MouseWheelDirection::Unknown(direction),
        }
    }
    #[inline]
    pub fn to_ll(self) -> u32 {
        match self {
            MouseWheelDirection::Normal => 0,
            MouseWheelDirection::Flipped => 1,
            MouseWheelDirection::Unknown(direction) => direction,
        }
    }
}

impl From<SDL_MouseWheelDirection> for MouseWheelDirection {
    fn from(direction: SDL_MouseWheelDirection) -> MouseWheelDirection {
        match direction {
            sys::mouse::SDL_MOUSEWHEEL_NORMAL => MouseWheelDirection::Normal,
            sys::mouse::SDL_MOUSEWHEEL_FLIPPED => MouseWheelDirection::Flipped,
            _ => MouseWheelDirection::Unknown(direction.0.try_into().unwrap()),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseButton {
    Unknown = 0,
    Left = sys::mouse::SDL_BUTTON_LEFT as u8,
    Middle = sys::mouse::SDL_BUTTON_MIDDLE as u8,
    Right = sys::mouse::SDL_BUTTON_RIGHT as u8,
    X1 = sys::mouse::SDL_BUTTON_X1 as u8,
    X2 = sys::mouse::SDL_BUTTON_X2 as u8,
}

impl MouseButton {
    #[inline]
    pub fn from_ll(button: u8) -> MouseButton {
        match button as i32 {
            sys::mouse::SDL_BUTTON_LEFT => MouseButton::Left,
            sys::mouse::SDL_BUTTON_MIDDLE => MouseButton::Middle,
            sys::mouse::SDL_BUTTON_RIGHT => MouseButton::Right,
            sys::mouse::SDL_BUTTON_X1 => MouseButton::X1,
            sys::mouse::SDL_BUTTON_X2 => MouseButton::X2,
            _ => MouseButton::Unknown,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseState {
    mouse_state: u32,
    x: f32,
    y: f32,
}

impl MouseState {
    #[doc(alias = "SDL_GetMouseState")]
    pub fn new(_e: &EventPump) -> MouseState {
        let mut x = 0.;
        let mut y = 0.;
        let mouse_state: u32 = unsafe { sys::mouse::SDL_GetMouseState(&mut x, &mut y) };

        MouseState { mouse_state, x, y }
    }

    pub fn from_sdl_state(state: u32) -> MouseState {
        MouseState {
            mouse_state: state,
            x: 0.,
            y: 0.,
        }
    }
    pub fn to_sdl_state(&self) -> u32 {
        self.mouse_state
    }

    fn button_mask(&self, button: i32) -> u32 {
        1 << (button - 1)
    }

    /// Returns true if the left mouse button is pressed.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::mouse::MouseButton;
    ///
    /// fn is_left_pressed(e: &sdl3::EventPump) -> bool {
    ///     e.mouse_state().left()
    /// }
    /// ```
    pub fn left(&self) -> bool {
        (self.mouse_state & self.button_mask(sys::mouse::SDL_BUTTON_LEFT)) != 0
    }

    /// Tests if the middle mouse button was pressed.
    pub fn middle(&self) -> bool {
        (self.mouse_state & self.button_mask(sys::mouse::SDL_BUTTON_MIDDLE)) != 0
    }

    /// Tests if the right mouse button was pressed.
    pub fn right(&self) -> bool {
        (self.mouse_state & self.button_mask(sys::mouse::SDL_BUTTON_RIGHT)) != 0
    }

    /// Tests if the X1 mouse button was pressed.
    pub fn x1(&self) -> bool {
        (self.mouse_state & self.button_mask(sys::mouse::SDL_BUTTON_X1)) != 0
    }

    /// Tests if the X2 mouse button was pressed.
    pub fn x2(&self) -> bool {
        (self.mouse_state & self.button_mask(sys::mouse::SDL_BUTTON_X2)) != 0
    }

    /// Returns the x coordinate of the state
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Returns the y coordinate of the state
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Returns true if the mouse button is pressed.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::mouse::MouseButton;
    ///
    /// fn is_left_pressed(e: &sdl3::EventPump) -> bool {
    ///     e.mouse_state().is_mouse_button_pressed(MouseButton::Left)
    /// }
    /// ```
    pub fn is_mouse_button_pressed(&self, mouse_button: MouseButton) -> bool {
        let mask = 1 << ((mouse_button as u32) - 1);
        self.mouse_state & mask != 0
    }

    /// Returns an iterator all mouse buttons with a boolean indicating if the scancode is pressed.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::mouse::MouseButton;
    /// use std::collections::HashMap;
    ///
    /// fn mouse_button_set(e: &sdl3::EventPump) -> HashMap<MouseButton, bool> {
    ///     e.mouse_state().mouse_buttons().collect()
    /// }
    ///
    /// fn find_first_pressed(e: &sdl3::EventPump) -> bool {
    ///     for (key,value) in mouse_button_set(e) {
    ///         return value != false
    ///     }
    ///     false
    /// }
    ///
    /// ```
    pub fn mouse_buttons(&self) -> MouseButtonIterator {
        MouseButtonIterator {
            cur_button: 1,
            mouse_state: &self.mouse_state,
        }
    }

    /// Returns an iterator of pressed mouse buttons.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::mouse::MouseButton;
    /// use std::collections::HashSet;
    ///
    /// fn pressed_mouse_button_set(e: &sdl3::EventPump) -> HashSet<MouseButton> {
    ///     e.mouse_state().pressed_mouse_buttons().collect()
    /// }
    ///
    /// fn newly_pressed(old: &HashSet<MouseButton>, new: &HashSet<MouseButton>) -> HashSet<MouseButton> {
    ///     new - old
    ///     // sugar for: new.difference(old).collect()
    /// }
    /// ```
    pub fn pressed_mouse_buttons(&self) -> PressedMouseButtonIterator {
        PressedMouseButtonIterator {
            iter: self.mouse_buttons(),
        }
    }
}

pub struct MouseButtonIterator<'a> {
    cur_button: u8,
    mouse_state: &'a u32,
}

impl Iterator for MouseButtonIterator<'_> {
    type Item = (MouseButton, bool);

    fn next(&mut self) -> Option<(MouseButton, bool)> {
        if self.cur_button < MouseButton::X2 as u8 + 1 {
            let mouse_button = self.cur_button;
            let mask = 1 << ((self.cur_button as u32) - 1);
            let pressed = self.mouse_state & mask != 0;
            self.cur_button += 1;
            Some((MouseButton::from_ll(mouse_button), pressed))
        } else {
            None
        }
    }
}

pub struct PressedMouseButtonIterator<'a> {
    iter: MouseButtonIterator<'a>,
}

impl Iterator for PressedMouseButtonIterator<'_> {
    type Item = MouseButton;

    fn next(&mut self) -> Option<MouseButton> {
        for (mouse_button, pressed) in self.iter.by_ref() {
            if pressed {
                return Some(mouse_button);
            }
        }
        None
    }
}

impl crate::Sdl {
    #[inline]
    pub fn mouse(&self) -> MouseUtil {
        MouseUtil {
            _sdldrop: self.sdldrop(),
        }
    }
}

/// Mouse utility functions. Access with `Sdl::mouse()`.
///
/// ```no_run
/// let sdl_context = sdl3::init().unwrap();
///
/// // Hide the cursor
/// sdl_context.mouse().show_cursor(false);
/// ```
pub struct MouseUtil {
    _sdldrop: crate::SdlDrop,
}

impl MouseUtil {
    /// Gets the id of the window which currently has mouse focus.
    #[doc(alias = "SDL_GetMouseFocus")]
    pub fn focused_window_id(&self) -> Option<u32> {
        let raw = unsafe { sys::mouse::SDL_GetMouseFocus() };
        if raw.is_null() {
            None
        } else {
            let id = unsafe { SDL_GetWindowID(raw) };
            Some(id)
        }
    }

    #[doc(alias = "SDL_WarpMouseInWindow")]
    pub fn warp_mouse_in_window(&self, window: &video::Window, x: f32, y: f32) {
        unsafe {
            sys::mouse::SDL_WarpMouseInWindow(window.raw(), x, y);
        }
    }

    #[doc(alias = "SDL_SetWindowRelativeMouseMode")]
    pub fn set_relative_mouse_mode(&self, window: &video::Window, on: bool) {
        unsafe {
            SDL_SetWindowRelativeMouseMode(window.raw(), on);
        }
    }

    #[doc(alias = "SDL_GetWindowRelativeMouseMode")]
    pub fn relative_mouse_mode(&self, window: &video::Window) -> bool {
        unsafe { SDL_GetWindowRelativeMouseMode(window.raw()) }
    }

    #[doc(alias = "SDL_CursorVisible")]
    pub fn is_cursor_showing(&self) -> bool {
        unsafe { sys::mouse::SDL_CursorVisible() }
    }

    #[doc(alias = "SDL_ShowCursor")]
    pub fn show_cursor(&self, show: bool) {
        unsafe {
            if show {
                sys::mouse::SDL_ShowCursor();
            } else {
                sys::mouse::SDL_HideCursor();
            }
        }
    }

    #[doc(alias = "SDL_CaptureMouse")]
    pub fn capture(&self, enable: bool) {
        unsafe {
            sys::mouse::SDL_CaptureMouse(enable);
        }
    }
}
