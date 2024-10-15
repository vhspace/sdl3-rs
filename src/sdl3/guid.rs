use std::ffi::{CStr, CString, NulError};
use std::fmt::{Display, Error, Formatter};
use libc::c_char;

/// Wrapper around a `SDL_GUID`, a globally unique identifier
/// for a joystick.
#[derive(Copy, Clone)]
pub struct Guid {
    pub raw: sys::guid::SDL_GUID,
}

impl PartialEq for Guid {
    fn eq(&self, other: &Guid) -> bool {
        self.raw.data == other.raw.data
    }
}

impl Eq for Guid {}

impl Guid {
    /// Create a GUID from a string representation.
    #[doc(alias = "SDL_StringToGUID")]
    pub fn from_string(guid: &str) -> Result<Guid, NulError> {
        let guid = CString::new(guid)?;

        let raw = unsafe { sys::guid::SDL_StringToGUID(guid.as_ptr() as *const c_char) };

        Ok(Guid { raw })
    }

    /// Return `true` if GUID is full 0s
    pub fn is_zero(&self) -> bool {
        for &i in &self.raw.data {
            if i != 0 {
                return false;
            }
        }

        true
    }

    /// Return a String representation of GUID
    #[doc(alias = "SDL_GUIDToString")]
    pub fn string(&self) -> String {
        // Doc says "buf should supply at least 33bytes". I took that
        // to mean that 33bytes should be enough in all cases, but
        // maybe I'm wrong?
        let mut buf = [0; 33];

        let len = buf.len() as i32;
        let c_str = buf.as_mut_ptr();

        unsafe {
            sys::guid::SDL_GUIDToString(self.raw, c_str, len);
        }

        // The buffer should always be NUL terminated (the
        // documentation doesn't explicitly say it but I checked the
        // code)
        if c_str.is_null() {
            String::new()
        } else {
            unsafe {
                CStr::from_ptr(c_str as *const _)
                    .to_str()
                    .unwrap()
                    .to_string()
            }
        }
    }

    /// Return a copy of the internal GUID
    pub fn raw(self) -> sys::joystick::SDL_GUID {
        self.raw
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.string())
    }
}