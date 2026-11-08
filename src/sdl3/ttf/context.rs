use crate::iostream::IOStream;
use crate::version::Version;
use crate::{get_error, Error};
use sdl3_ttf_sys::ttf;
use std::ffi::CString;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use super::font::Font;

/// A context manager for `SDL3_TTF` to manage C code initialization and clean-up.
/// SDL3_TTF is (do a full check on this) thread-safe, so we let it be `Send` and `Sync`
#[must_use]
pub struct Sdl3TtfContext;

// Clean up the context once it goes out of scope
impl Drop for Sdl3TtfContext {
    fn drop(&mut self) {
        let prev_count = TTF_COUNT.fetch_sub(1, Ordering::Relaxed);
        assert!(prev_count > 0);
        if prev_count == 1 {
            unsafe {
                ttf::TTF_Quit();
            }
        }
    }
}

static TTF_COUNT: AtomicU32 = AtomicU32::new(0);

impl Sdl3TtfContext {
    /// Initializes the truetype font API and returns a context manager which will
    /// clean up the library once it goes out of scope.
    #[doc(alias = "TTF_Init")]
    fn new() -> Result<Self, Error> {
        if TTF_COUNT.fetch_add(1, Ordering::Relaxed) == 0 {
            let result;

            unsafe {
                result = ttf::TTF_Init();
            }

            if !result {
                TTF_COUNT.store(0, Ordering::Relaxed);
                return Err(get_error());
            }
        }

        Ok(Self)
    }

    /// Loads a font from the given file with the given size in points.
    #[doc(alias = "TTF_OpenFont")]
    pub fn load_font<P: AsRef<Path>>(
        &self,
        path: P,
        point_size: f32,
    ) -> Result<Font<'static>, Error> {
        unsafe {
            let cstring = CString::new(path.as_ref().to_str().unwrap()).unwrap();
            let raw = ttf::TTF_OpenFont(cstring.as_ptr(), point_size);
            if raw.is_null() {
                Err(get_error())
            } else {
                Ok(Font::new(self.clone(), raw, None))
            }
        }
    }

    /// Loads a font from the given SDL3 iostream object with the given size in
    /// points.
    #[doc(alias = "TTF_OpenFontIO")]
    pub fn load_font_from_iostream<'r>(
        &self,
        iostream: IOStream<'r>,
        point_size: f32,
    ) -> Result<Font<'r>, Error> {
        let raw = unsafe { ttf::TTF_OpenFontIO(iostream.raw(), false, point_size) };
        if (raw as *mut ()).is_null() {
            Err(get_error())
        } else {
            Ok(Font::new(self.clone(), raw, iostream.into()))
        }
    }
}

impl Clone for Sdl3TtfContext {
    fn clone(&self) -> Self {
        let prev_count = TTF_COUNT.fetch_add(1, Ordering::Relaxed);
        assert!(prev_count > 0);
        Self
    }
}

/// Returns the version of the dynamically linked `SDL_TTF` library
#[doc(alias = "TTF_Version")]
pub fn get_linked_version() -> Version {
    unsafe { Version::from_ll(ttf::TTF_Version()) }
}

#[inline]
pub fn init() -> Result<Sdl3TtfContext, Error> {
    Sdl3TtfContext::new()
}

/// Returns whether library has been initialized already.
#[doc(alias = "TTF_WasInit")]
pub fn has_been_initialized() -> bool {
    unsafe { ttf::TTF_WasInit() == 1 }
}
