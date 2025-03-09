use crate::iostream::IOStream;
use crate::version::Version;
use crate::{get_error, Error};
use sdl3_ttf_sys::ttf;
use std::error;
use std::fmt;
use std::io;
use std::path::Path;

use super::font::{internal_load_font, internal_load_font_from_ll, Font};

/// A context manager for `SDL2_TTF` to manage C code initialization and clean-up.
#[must_use]
pub struct Sdl3TtfContext;

// Clean up the context once it goes out of scope
impl Drop for Sdl3TtfContext {
    fn drop(&mut self) {
        unsafe {
            ttf::TTF_Quit();
        }
    }
}

impl Sdl3TtfContext {
    /// Loads a font from the given file with the given size in points.
    pub fn load_font<'ttf, P: AsRef<Path>>(
        &'ttf self,
        path: P,
        point_size: f32,
    ) -> Result<Font<'ttf, 'static>, Error> {
        internal_load_font(path, point_size)
    }

    /// Loads a font from the given SDL2 iostream object with the given size in
    /// points.
    pub fn load_font_from_iostream<'ttf, 'r>(
        &'ttf self,
        iostream: IOStream<'r>,
        point_size: f32,
    ) -> Result<Font<'ttf, 'r>, Error> {
        let raw = unsafe { ttf::TTF_OpenFontIO(iostream.raw(), false, point_size) };
        if (raw as *mut ()).is_null() {
            Err(get_error())
        } else {
            Ok(internal_load_font_from_ll(raw, Some(iostream)))
        }
    }
}

/// Returns the version of the dynamically linked `SDL_TTF` library
pub fn get_linked_version() -> Version {
    unsafe { Version::from_ll(ttf::TTF_Version()) }
}

/// An error for when `sdl2_ttf` is attempted initialized twice
/// Necessary for context management, unless we find a way to have a singleton
#[derive(Debug)]
pub enum InitError {
    InitializationError(io::Error),
    AlreadyInitializedError,
}

impl error::Error for InitError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            InitError::AlreadyInitializedError => None,
            InitError::InitializationError(ref error) => Some(error),
        }
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str("SDL2_TTF has already been initialized")
    }
}

/// Initializes the truetype font API and returns a context manager which will
/// clean up the library once it goes out of scope.
pub fn init() -> Result<Sdl3TtfContext, InitError> {
    unsafe {
        if ttf::TTF_WasInit() == 1 {
            Err(InitError::AlreadyInitializedError)
        } else if ttf::TTF_Init() {
            Ok(Sdl3TtfContext)
        } else {
            Err(InitError::InitializationError(io::Error::last_os_error()))
        }
    }
}

/// Returns whether library has been initialized already.
pub fn has_been_initialized() -> bool {
    unsafe { ttf::TTF_WasInit() == 1 }
}
