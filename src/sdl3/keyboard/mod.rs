use crate::rect::Rect;
use crate::video::Window;
use crate::EventPump;

use crate::sys;
use std::fmt;
use std::mem::transmute;
use sys::video::SDL_GetWindowID;

mod keycode;
mod scancode;
pub use self::keycode::Keycode;
pub use self::scancode::Scancode;

bitflags! {
    pub struct Mod: u16 {
        const NOMOD = 0x0000;
        const LSHIFTMOD = 0x0001;
        const RSHIFTMOD = 0x0002;
        const LCTRLMOD = 0x0040;
        const RCTRLMOD = 0x0080;
        const LALTMOD = 0x0100;
        const RALTMOD = 0x0200;
        const LGUIMOD = 0x0400;
        const RGUIMOD = 0x0800;
        const NUMMOD = 0x1000;
        const CAPSMOD = 0x2000;
        const MODEMOD = 0x4000;
        const RESERVEDMOD = 0x8000;
    }
}

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x}", *self)
    }
}

pub struct KeyboardState<'a> {
    keyboard_state: &'a [bool],
}

impl<'a> KeyboardState<'a> {
    #[doc(alias = "SDL_GetKeyboardState")]
    pub fn new(_e: &'a EventPump) -> KeyboardState<'a> {
        let keyboard_state = unsafe {
            let mut count = 0;
            let state_ptr = sys::keyboard::SDL_GetKeyboardState(&mut count);

            ::std::slice::from_raw_parts(state_ptr, count as usize)
        };

        KeyboardState { keyboard_state }
    }

    /// Returns true if the scancode is pressed.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::keyboard::Scancode;
    ///
    /// fn is_a_pressed(e: &sdl3::EventPump) -> bool {
    ///     e.keyboard_state().is_scancode_pressed(Scancode::A)
    /// }
    /// ```
    pub fn is_scancode_pressed(&self, scancode: Scancode) -> bool {
        self.keyboard_state[scancode as i32 as usize]
    }

    /// Returns an iterator all scancodes with a boolean indicating if the scancode is pressed.
    pub fn scancodes(&self) -> ScancodeIterator {
        ScancodeIterator {
            index: 0,
            keyboard_state: self.keyboard_state,
        }
    }

    /// Returns an iterator of pressed scancodes.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::keyboard::Keycode;
    /// use sdl3::keyboard::Scancode;
    /// use std::collections::HashSet;
    ///
    /// fn pressed_scancode_set(e: &sdl3::EventPump) -> HashSet<Scancode> {
    ///     e.keyboard_state().pressed_scancodes().collect()
    /// }
    ///
    /// fn pressed_keycode_set(e: &sdl3::EventPump) -> HashSet<Keycode> {
    ///     e.keyboard_state().pressed_scancodes()
    ///         .filter_map(Keycode::from_scancode)
    ///         .collect()
    /// }
    ///
    /// fn newly_pressed(old: &HashSet<Scancode>, new: &HashSet<Scancode>) -> HashSet<Scancode> {
    ///     new - old
    ///     // sugar for: new.difference(old).collect()
    /// }
    /// ```
    pub fn pressed_scancodes(&self) -> PressedScancodeIterator {
        PressedScancodeIterator {
            iter: self.scancodes(),
        }
    }
}

pub struct ScancodeIterator<'a> {
    index: i32,
    keyboard_state: &'a [bool],
}

impl<'a> Iterator for ScancodeIterator<'a> {
    type Item = (Scancode, bool);

    fn next(&mut self) -> Option<(Scancode, bool)> {
        if self.index < self.keyboard_state.len() as i32 {
            let index = self.index;
            self.index += 1;

            if let Some(scancode) = Scancode::from_i32(index) {
                let pressed = self.keyboard_state[index as usize];

                Some((scancode, pressed))
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

pub struct PressedScancodeIterator<'a> {
    iter: ScancodeIterator<'a>,
}

impl<'a> Iterator for PressedScancodeIterator<'a> {
    type Item = Scancode;

    fn next(&mut self) -> Option<Scancode> {
        while let Some((scancode, pressed)) = self.iter.next() {
            if pressed {
                return Some(scancode);
            }
        }

        None
    }
}

impl crate::Sdl {
    #[inline]
    pub fn keyboard(&self) -> KeyboardUtil {
        KeyboardUtil {
            _sdldrop: self.sdldrop(),
        }
    }
}

impl crate::VideoSubsystem {
    #[inline]
    pub fn text_input(&self) -> TextInputUtil {
        TextInputUtil {
            _subsystem: self.clone(),
        }
    }
}

/// Keyboard utility functions. Access with `Sdl::keyboard()`.
///
/// ```no_run
/// let sdl_context = sdl3::init().unwrap();
///
/// let focused = sdl_context.keyboard().focused_window_id().is_some();
/// ```
pub struct KeyboardUtil {
    _sdldrop: crate::SdlDrop,
}

impl KeyboardUtil {
    /// Gets the id of the window which currently has keyboard focus.
    #[doc(alias = "SDL_GetKeyboardFocus")]
    pub fn focused_window_id(&self) -> Option<u32> {
        let raw = unsafe { sys::keyboard::SDL_GetKeyboardFocus() };
        if raw.is_null() {
            None
        } else {
            let id = unsafe { SDL_GetWindowID(raw) };
            Some(id)
        }
    }

    #[doc(alias = "SDL_GetModState")]
    pub fn mod_state(&self) -> Mod {
        unsafe { Mod::from_bits(sys::keyboard::SDL_GetModState() as u16).unwrap() }
    }

    #[doc(alias = "SDL_SetModState")]
    pub fn set_mod_state(&self, flags: Mod) {
        unsafe {
            sys::keyboard::SDL_SetModState(transmute::<u16, sys::keycode::SDL_Keymod>(
                flags.bits(),
            ));
        }
    }
}

/// Text input utility functions. Access with `VideoSubsystem::text_input()`.
///
/// These functions require the video subsystem to be initialized and are not thread-safe.
///
/// ```no_run
/// let sdl_context = sdl3::init().unwrap();
/// let video_subsystem = sdl_context.video().unwrap();
/// let window = video_subsystem.window("Example", 800, 600).build().unwrap();
///
/// // Start accepting text input events...
/// video_subsystem.text_input().start(&window);
/// ```
pub struct TextInputUtil {
    _subsystem: crate::VideoSubsystem,
}

impl TextInputUtil {
    #[doc(alias = "SDL_StartTextInput")]
    pub fn start(&self, window: &Window) {
        unsafe {
            sys::keyboard::SDL_StartTextInput(window.raw());
        }
    }

    #[doc(alias = "SDL_TextInputActive")]
    pub fn is_active(&self, window: &Window) -> bool {
        unsafe { sys::keyboard::SDL_TextInputActive(window.raw()) }
    }

    #[doc(alias = "SDL_StopTextInput")]
    pub fn stop(&self, window: &Window) {
        unsafe {
            sys::keyboard::SDL_StopTextInput(window.raw());
        }
    }

    #[doc(alias = "SDL_SetTextInputArea")]
    pub fn set_rect(&self, window: Window, rect: Rect, cursor: i32) {
        unsafe {
            sys::keyboard::SDL_SetTextInputArea(
                window.raw(),
                rect.raw() as *mut sys::rect::SDL_Rect,
                cursor,
            );
        }
    }

    #[doc(alias = "SDL_HasScreenKeyboardSupport")]
    pub fn has_screen_keyboard_support(&self) -> bool {
        unsafe { sys::keyboard::SDL_HasScreenKeyboardSupport() }
    }

    #[doc(alias = "SDL_ScreenKeyboardShown")]
    pub fn is_screen_keyboard_shown(&self, window: &Window) -> bool {
        unsafe { sys::keyboard::SDL_ScreenKeyboardShown(window.raw()) }
    }
}
