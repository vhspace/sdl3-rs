use crate::get_error;
use crate::rect::Rect;
use crate::video::Window;
use crate::Error;
use crate::EventPump;

use crate::sys;
use std::fmt;
use sys::video::SDL_GetWindowID;

mod keycode;
mod scancode;
pub use self::keycode::Keycode;
pub use self::scancode::Scancode;

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Mod: u16 {
        const NOMOD = 0x0000;
        const LSHIFTMOD = 0x0001;
        const RSHIFTMOD = 0x0002;
        const LEVEL5MOD = 0x0004;
        const LCTRLMOD = 0x0040;
        const RCTRLMOD = 0x0080;
        const LALTMOD = 0x0100;
        const RALTMOD = 0x0200;
        const LGUIMOD = 0x0400;
        const RGUIMOD = 0x0800;
        const NUMMOD = 0x1000;
        const CAPSMOD = 0x2000;
        const MODEMOD = 0x4000;
        const SCROLLMOD = 0x8000;
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
    pub fn scancodes(&self) -> ScancodeIterator<'_> {
        ScancodeIterator {
            index: 0,
            keyboard_state: self.keyboard_state,
        }
    }

    /// Returns an iterator of pressed scancodes.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::keyboard::{Keycode, Mod, Scancode};
    /// use sdl3::sys::keycode::SDL_Keymod;
    /// use std::collections::HashSet;
    ///
    /// fn pressed_scancode_set(e: &sdl3::EventPump) -> HashSet<Scancode> {
    ///     e.keyboard_state().pressed_scancodes().collect()
    /// }
    ///
    /// fn pressed_keycode_set(e: &sdl3::EventPump) -> HashSet<Keycode> {
    ///     e.keyboard_state().pressed_scancodes()
    ///         .filter_map(|scancode| Keycode::from_scancode(scancode, SDL_Keymod(Mod::NOMOD.bits()), false))
    ///         .collect()
    /// }
    ///
    /// fn newly_pressed(old: &HashSet<Scancode>, new: &HashSet<Scancode>) -> HashSet<Scancode> {
    ///     new - old
    ///     // sugar for: new.difference(old).collect()
    /// }
    /// ```
    pub fn pressed_scancodes(&self) -> PressedScancodeIterator<'_> {
        PressedScancodeIterator {
            iter: self.scancodes(),
        }
    }
}

pub struct ScancodeIterator<'a> {
    index: i32,
    keyboard_state: &'a [bool],
}

impl Iterator for ScancodeIterator<'_> {
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

impl Iterator for PressedScancodeIterator<'_> {
    type Item = Scancode;

    fn next(&mut self) -> Option<Scancode> {
        for (scancode, pressed) in self.iter.by_ref() {
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
            Some(id.into())
        }
    }

    #[doc(alias = "SDL_GetModState")]
    pub fn mod_state(&self) -> Mod {
        unsafe { Mod::from_bits(sys::keyboard::SDL_GetModState().0).unwrap() }
    }

    #[doc(alias = "SDL_SetModState")]
    pub fn set_mod_state(&self, flags: Mod) {
        unsafe {
            sys::keyboard::SDL_SetModState(sys::keycode::SDL_Keymod(flags.bits()));
        }
    }
}

/// The type of text being entered, used to hint the platform's on-screen keyboard.
///
/// # Remarks
/// Not every platform honours every value. Passwords additionally disable the
/// input method editor on platforms that support doing so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc(alias = "SDL_TextInputType")]
pub enum TextInputType {
    /// The input is text.
    Text,
    /// The input is a person's name.
    Name,
    /// The input is an e-mail address.
    Email,
    /// The input is a username.
    Username,
    /// The input is a secure password that is hidden.
    PasswordHidden,
    /// The input is a secure password that is visible.
    PasswordVisible,
    /// The input is a number.
    Number,
    /// The input is a secure PIN that is hidden.
    NumberPasswordHidden,
    /// The input is a secure PIN that is visible.
    NumberPasswordVisible,
}

impl TextInputType {
    fn to_ll(self) -> sys::keyboard::SDL_TextInputType {
        use sys::keyboard::SDL_TextInputType as T;
        match self {
            TextInputType::Text => T::TEXT,
            TextInputType::Name => T::TEXT_NAME,
            TextInputType::Email => T::TEXT_EMAIL,
            TextInputType::Username => T::TEXT_USERNAME,
            TextInputType::PasswordHidden => T::TEXT_PASSWORD_HIDDEN,
            TextInputType::PasswordVisible => T::TEXT_PASSWORD_VISIBLE,
            TextInputType::Number => T::NUMBER,
            TextInputType::NumberPasswordHidden => T::NUMBER_PASSWORD_HIDDEN,
            TextInputType::NumberPasswordVisible => T::NUMBER_PASSWORD_VISIBLE,
        }
    }
}

/// How the platform should auto-capitalize entered text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc(alias = "SDL_Capitalization")]
pub enum Capitalization {
    /// No auto-capitalization will be done.
    None,
    /// The first letter of sentences will be capitalized.
    Sentences,
    /// The first letter of words will be capitalized.
    Words,
    /// All letters will be capitalized.
    Letters,
}

impl Capitalization {
    fn to_ll(self) -> sys::keyboard::SDL_Capitalization {
        use sys::keyboard::SDL_Capitalization as C;
        match self {
            Capitalization::None => C::NONE,
            Capitalization::Sentences => C::SENTENCES,
            Capitalization::Words => C::WORDS,
            Capitalization::Letters => C::LETTERS,
        }
    }
}

/// Options for [`TextInputUtil::start_with_options`].
///
/// # Remarks
/// Every field is optional; a `None` leaves the corresponding SDL property unset,
/// so SDL applies its own default. The defaults for capitalization depend on the
/// input type, so prefer leaving it `None` unless you need to override it.
///
/// ```no_run
/// use sdl3::keyboard::{TextInputOptions, TextInputType};
///
/// let options = TextInputOptions {
///     input_type: Some(TextInputType::Email),
///     autocorrect: Some(false),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextInputOptions {
    /// Describes the text being entered. Defaults to [`TextInputType::Text`].
    pub input_type: Option<TextInputType>,
    /// How entered text should be auto-capitalized. The SDL default depends on
    /// `input_type`: sentences for [`TextInputType::Text`], words for
    /// [`TextInputType::Name`], and none for e-mail addresses and usernames.
    pub capitalization: Option<Capitalization>,
    /// Whether to enable auto-completion and auto-correction. Defaults to `true`.
    pub autocorrect: Option<bool>,
    /// Whether multiple lines of text are allowed. This lets the on-screen keyboard
    /// show a newline key instead of a return key, and prevents closing the keyboard
    /// when pressing it. Defaults to `false`.
    pub multiline: Option<bool>,
    /// Android only: the `InputType` value to pass through verbatim, overriding
    /// whatever `input_type`, `capitalization` and `autocorrect` would have produced.
    /// Ignored on other platforms.
    pub android_input_type: Option<i32>,
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
    pub fn set_rect(&self, window: &Window, rect: Rect, cursor: i32) {
        unsafe {
            sys::keyboard::SDL_SetTextInputArea(
                window.raw(),
                rect.raw() as *mut sys::rect::SDL_Rect,
                cursor,
            );
        }
    }

    /// Get the area used to type Unicode text input, as last set by [`Self::set_rect`].
    ///
    /// # Remarks
    /// Returns the rectangle in window coordinates together with the offset, in
    /// pixels, of the text cursor from the left edge of that rectangle.
    #[doc(alias = "SDL_GetTextInputArea")]
    pub fn rect(&self, window: &Window) -> Result<(Rect, i32), Error> {
        let mut rect = sys::rect::SDL_Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        let mut cursor: std::os::raw::c_int = 0;
        let ok =
            unsafe { sys::keyboard::SDL_GetTextInputArea(window.raw(), &mut rect, &mut cursor) };
        if ok {
            Ok((Rect::from_ll(rect), cursor))
        } else {
            Err(get_error())
        }
    }

    /// Start accepting Unicode text input events, describing what is being entered.
    ///
    /// # Remarks
    /// This is [`Self::start`] plus the hints SDL needs to present a suitable
    /// on-screen keyboard: an e-mail field can be given a keyboard with `@` on it,
    /// and a password field can have the input method editor turned off. Every hint
    /// is advisory — platforms that do not support one simply ignore it.
    ///
    /// Text input is not automatically enabled, and you should also use
    /// [`Self::set_rect`] to tell SDL where the text is being entered so that the
    /// candidate window of an input method editor lands next to the cursor.
    ///
    /// ```no_run
    /// use sdl3::keyboard::{TextInputOptions, TextInputType};
    ///
    /// # fn example(video_subsystem: &sdl3::VideoSubsystem, window: &sdl3::video::Window) -> Result<(), sdl3::Error> {
    /// video_subsystem.text_input().start_with_options(
    ///     window,
    ///     TextInputOptions {
    ///         input_type: Some(TextInputType::Email),
    ///         autocorrect: Some(false),
    ///         ..Default::default()
    ///     },
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[doc(alias = "SDL_StartTextInputWithProperties")]
    pub fn start_with_options(
        &self,
        window: &Window,
        options: TextInputOptions,
    ) -> Result<(), Error> {
        unsafe {
            let props = sys::properties::SDL_CreateProperties();
            if props.0 == 0 {
                return Err(get_error());
            }

            if let Some(input_type) = options.input_type {
                sys::properties::SDL_SetNumberProperty(
                    props,
                    sys::keyboard::SDL_PROP_TEXTINPUT_TYPE_NUMBER,
                    input_type.to_ll().0 as i64,
                );
            }
            if let Some(capitalization) = options.capitalization {
                sys::properties::SDL_SetNumberProperty(
                    props,
                    sys::keyboard::SDL_PROP_TEXTINPUT_CAPITALIZATION_NUMBER,
                    capitalization.to_ll().0 as i64,
                );
            }
            if let Some(autocorrect) = options.autocorrect {
                sys::properties::SDL_SetBooleanProperty(
                    props,
                    sys::keyboard::SDL_PROP_TEXTINPUT_AUTOCORRECT_BOOLEAN,
                    autocorrect,
                );
            }
            if let Some(multiline) = options.multiline {
                sys::properties::SDL_SetBooleanProperty(
                    props,
                    sys::keyboard::SDL_PROP_TEXTINPUT_MULTILINE_BOOLEAN,
                    multiline,
                );
            }
            if let Some(android_input_type) = options.android_input_type {
                sys::properties::SDL_SetNumberProperty(
                    props,
                    sys::keyboard::SDL_PROP_TEXTINPUT_ANDROID_INPUTTYPE_NUMBER,
                    android_input_type as i64,
                );
            }

            let ok = sys::keyboard::SDL_StartTextInputWithProperties(window.raw(), props);
            // SDL copies what it needs out of the property group, so it can go now.
            sys::properties::SDL_DestroyProperties(props);
            if ok {
                Ok(())
            } else {
                Err(get_error())
            }
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
