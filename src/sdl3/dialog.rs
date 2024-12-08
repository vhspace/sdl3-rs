use crate::get_error;
use crate::sys;
use core::fmt;
use libc::{c_char, c_int, c_void};
use std::ffi::NulError;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use std::str::{FromStr, Utf8Error};
use sys::dialog::SDL_DialogFileFilter;

use crate::video::Window;

#[derive(Debug)]
pub struct DialogFileFilter<'a> {
    pub name: &'a str,
    pub pattern: &'a str,
}

#[derive(Debug, Clone)]
pub enum DialogError {
    FilterError(NulError),
    InvalidFilename(Utf8Error),
    Canceled,
    SdlError(String),
}

impl fmt::Display for DialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::DialogError::*;

        match *self {
            FilterError(ref e) => write!(f, "Could not create filter: {}", e),
            InvalidFilename(ref e) => write!(f, "Invalid filename: {}", e),
            Canceled => write!(f, "Canceled"),
            SdlError(ref e) => write!(f, "SDL error: {}", e),
        }
    }
}

pub type DialogCallback = Box<dyn Fn(Result<Vec<PathBuf>, DialogError>, Option<DialogFileFilter>)>;

struct DialogCallbackData {
    pub callback: DialogCallback,
    pub filter_strings: Option<Vec<(CString, CString)>>,
}

extern "C" fn c_dialog_callback(
    userdata: *mut c_void,
    filelist: *const *const c_char,
    filter: c_int,
) {
    let callback_info_ptr = userdata as *mut DialogCallbackData;
    if filelist.is_null() {
        unsafe {
            return ((*callback_info_ptr).callback)(Err(DialogError::SdlError(get_error())), None);
        }
    }

    let mut files = Vec::new();
    unsafe {
        let mut count = 0;
        loop {
            let file = *filelist.offset(count);
            if file.is_null() {
                break;
            }

            let file = CStr::from_ptr(file);
            match file.to_str() {
                // PathBuf::from_str can not fail
                Ok(file) => files.push(PathBuf::from_str(file).unwrap()),
                Err(e) => {
                    return ((*callback_info_ptr).callback)(
                        Err(DialogError::InvalidFilename(e)),
                        None,
                    )
                }
            };

            count += 1;
        }

        if count == 0 {
            return ((*callback_info_ptr).callback)(Err(DialogError::Canceled), None);
        }
    }
    unsafe {
        if filter < 0 {
            return ((*callback_info_ptr).callback)(Ok(files), None);
        } else {
            // Seemingly not implemented in linux portals, untested
            if let Some(filter_strings) = &(*callback_info_ptr).filter_strings {
                if let Some(filter) = filter_strings.get(filter as usize) {
                    let filter = DialogFileFilter {
                        // We created these from strs, they cannot fail
                        name: filter.0.to_str().unwrap(),
                        pattern: filter.1.to_str().unwrap(),
                    };
                    return ((*callback_info_ptr).callback)(Ok(files), Some(filter));
                }
            }
        }
    }
}

/// Take a slice of DialogFileFilter and transform it into two vecs
/// The filter_strings vec contains the CStrings.
///     filter_strings must not be dropped until the callback is complete.
/// The c_filters vec contains pointers to the CStrings in filter_strings.
macro_rules! filters {
    ($filters:ident, $filter_strings:ident, $c_filters:ident) => {
        let mut $filter_strings = Vec::new();
        for filter in $filters {
            match (CString::new(filter.name), CString::new(filter.pattern)) {
                (Ok(name), Ok(pattern)) => {
                    $filter_strings.push((name, pattern));
                }
                (Err(error), _) | (_, Err(error)) => {
                    return Err(DialogError::FilterError(error));
                }
            }
        }
        let $c_filters: Vec<SDL_DialogFileFilter> = $filter_strings
            .iter()
            .map(|(name, pattern)| SDL_DialogFileFilter {
                name: name.as_ptr(),
                pattern: pattern.as_ptr(),
            })
            .collect();
    };
}

/// If an optional window exists get it's pointer, otherwise get a null pointer.
macro_rules! window_ptr {
    ($window:ident, $window_ptr:ident) => {
        let $window_ptr = $window.map_or(ptr::null_mut(), |win| win.raw());
    };
}

/// Take an optional path parameter and convert it into a CString and a pointer to it.
/// If there is no path the pointer will be null.
macro_rules! default_location_ptr {
    ($default_location:ident, $default_location_ptr:ident) => {
        let default_location = match $default_location {
            Some(path) => Some(CString::new(path.as_ref().to_str().unwrap()).unwrap()),
            None => None,
        };
        let $default_location_ptr = default_location
            .as_ref()
            .map_or(ptr::null(), |path| path.as_ptr());
    };
}

macro_rules! callback_data_ptr {
    ($callback:ident, $filter_strings:expr, $callback_data_ptr:ident) => {
        let callback_data = DialogCallbackData {
            callback: $callback,
            filter_strings: $filter_strings,
        };
        let $callback_data_ptr = Box::into_raw(Box::new(callback_data));
    };
}

#[doc(alias = "SDL_ShowOpenFileDialog")]
pub fn show_open_file_dialog<'a, W>(
    filters: &[DialogFileFilter],
    default_location: Option<impl AsRef<Path>>,
    allow_many: bool,
    window: W,
    callback: DialogCallback,
) -> Result<(), DialogError>
where
    W: Into<Option<&'a Window>>,
{
    let window = window.into();

    filters!(filters, filter_strings, c_filters);

    unsafe {
        window_ptr!(window, window_ptr);
        default_location_ptr!(default_location, default_location_ptr);
        callback_data_ptr!(callback, Some(filter_strings), callback_data_ptr);

        sys::dialog::SDL_ShowOpenFileDialog(
            Some(c_dialog_callback),
            callback_data_ptr as *mut c_void,
            window_ptr,
            c_filters.as_ptr(),
            c_filters.len() as i32,
            default_location_ptr,
            allow_many,
        );
        Ok(())
    }
}

#[doc(alias = "SDL_ShowOpenFolderDialog")]
pub fn show_open_folder_dialog<'a, W>(
    default_location: Option<impl AsRef<Path>>,
    allow_many: bool,
    window: W,
    callback: DialogCallback,
) where
    W: Into<Option<&'a Window>>,
{
    let window = window.into();

    unsafe {
        window_ptr!(window, window_ptr);
        default_location_ptr!(default_location, default_location_ptr);
        callback_data_ptr!(callback, None, callback_data_ptr);

        sys::dialog::SDL_ShowOpenFolderDialog(
            Some(c_dialog_callback),
            callback_data_ptr as *mut c_void,
            window_ptr,
            default_location_ptr,
            allow_many,
        );
    }
}

#[doc(alias = "SDL_ShowSaveFileDialog")]
pub fn show_save_file_dialog<'a, W>(
    filters: &[DialogFileFilter],
    default_location: Option<impl AsRef<Path>>,
    window: W,
    callback: DialogCallback,
) -> Result<(), DialogError>
where
    W: Into<Option<&'a Window>>,
{
    let window = window.into();

    filters!(filters, filter_strings, c_filters);

    unsafe {
        window_ptr!(window, window_ptr);
        default_location_ptr!(default_location, default_location_ptr);
        callback_data_ptr!(callback, Some(filter_strings), callback_data_ptr);

        sys::dialog::SDL_ShowSaveFileDialog(
            Some(c_dialog_callback),
            callback_data_ptr as *mut c_void,
            window_ptr,
            c_filters.as_ptr(),
            c_filters.len() as i32,
            default_location_ptr,
        );
        Ok(())
    }
}
