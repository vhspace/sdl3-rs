//! Opening URLs in default system handlers

use crate::get_error;
use std::error;
use std::ffi::{CString, NulError};
use std::fmt;
use sys::misc::SDL_OpenURL;


#[derive(Debug, Clone)]
pub enum OpenUrlError {
    InvalidUrl(NulError),
    SdlError(String),
}

impl fmt::Display for OpenUrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OpenUrlError::*;

        match *self {
            InvalidUrl(ref e) => write!(f, "Invalid URL: {}", e),
            SdlError(ref e) => write!(f, "SDL error: {}", e),
        }
    }
}

impl error::Error for OpenUrlError {
    fn description(&self) -> &str {
        use self::OpenUrlError::*;

        match *self {
            InvalidUrl(_) => "invalid URL",
            SdlError(ref e) => e,
        }
    }
}

/// Opens a URL/URI in the default system-provided application.
///
/// This will most likely open a web browser for http:// and https:// links,
/// the default handler application for file:// links, but this varies
/// between platforms and is not supported on all of them.
/// It might also cause your window to lose focus, or pause your process on mobile.
///
/// There is no way to tell if the system successfully opened the provided URL,
/// an `Ok` result only means that something was launched to try to handle it.
///
/// # Examples
///
/// ```no_run
/// use sdl3::url::open_url;
///
/// open_url("https://github.com/revmischa/sdl3-rs")
///   .expect("Opening URLs not supported on this platform");
/// ```
#[doc(alias = "SDL_OpenURL")]
pub fn open_url(url: &str) -> Result<(), OpenUrlError> {
    use self::OpenUrlError::*;
    let result = unsafe {
        let url = match CString::new(url) {
            Ok(s) => s,
            Err(err) => return Err(InvalidUrl(err)),
        };
        SDL_OpenURL(url.as_ptr())
    };

    if result {
        Ok(())
    } else {
        Err(SdlError(get_error()))
    }
}
