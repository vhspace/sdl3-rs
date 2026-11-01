//! Native file and folder dialogs.
//!
//! This module wraps `SDL_dialog.h`. It lets an application pop up the
//! platform's own "open file", "save file" and "open folder" dialogs and be
//! told which paths the user picked.
//!
//! All of the dialog functions are asynchronous: they return immediately and
//! the result is delivered later to a [`DialogCallback`]. Note that the
//! callback may run on a different thread than the one the dialog was
//! requested from, depending on the operating system. The callback is invoked
//! exactly once per dialog, whether the user accepted, canceled, or an error
//! occurred.
//!
//! On Linux, dialogs may require XDG Portals, which requires DBus, which
//! requires an event-handling loop. Applications that do not use SDL to handle
//! events should call [`EventPump::pump_events`](crate::EventPump::pump_events)
//! in their main loop so the dialog can make progress.
//!
//! # Example
//!
//! ```no_run
//! use sdl3::dialog::{show_open_file_dialog, DialogFileFilter};
//! use std::path::PathBuf;
//!
//! let sdl_context = sdl3::init().unwrap();
//! let video_subsystem = sdl_context.video().unwrap();
//! let window = video_subsystem.window("dialog", 800, 600).build().unwrap();
//!
//! let filters = [
//!     DialogFileFilter { name: "Text", pattern: "txt" },
//!     DialogFileFilter { name: "Videos", pattern: "mp4;mkv" },
//!     DialogFileFilter { name: "All", pattern: "*" },
//! ];
//!
//! show_open_file_dialog(
//!     &filters,
//!     None::<PathBuf>,
//!     true,
//!     &window,
//!     Box::new(|result, filter| match result {
//!         Ok(paths) => println!("picked {paths:?} with filter {filter:?}"),
//!         Err(error) => eprintln!("dialog failed: {error}"),
//!     }),
//! )
//! .unwrap();
//!
//! // Keep pumping events so the dialog can run and the callback gets called.
//! let mut event_pump = sdl_context.event_pump().unwrap();
//! loop {
//!     for _event in event_pump.poll_iter() {}
//! }
//! ```
//!
//! See `examples/dialog.rs` for a complete program that drives all three
//! dialog types.

use crate::get_error;
use crate::sys;
use crate::Error;
use core::fmt;
use libc::{c_char, c_int, c_void};
use std::ffi::NulError;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use std::str::{FromStr, Utf8Error};
use sys::dialog::SDL_DialogFileFilter;

use crate::video::Window;

/// An entry in the list of file type filters shown by a file dialog.
///
/// Filters are passed to [`show_open_file_dialog`] and
/// [`show_save_file_dialog`]. Not all platforms support filters, and platforms
/// that do may let the user ignore them.
///
/// # Example
///
/// ```
/// use sdl3::dialog::DialogFileFilter;
///
/// let filters = [
///     DialogFileFilter { name: "Office document", pattern: "doc;docx" },
///     DialogFileFilter { name: "All files", pattern: "*" },
/// ];
/// ```
#[doc(alias = "SDL_DialogFileFilter")]
#[derive(Debug)]
pub struct DialogFileFilter<'a> {
    /// A user-readable label for the filter, for example `"Office document"`.
    pub name: &'a str,
    /// A semicolon-separated list of file extensions, for example
    /// `"doc;docx"`. Extensions may only contain alphanumeric characters,
    /// hyphens, underscores and periods. Alternatively the whole string can be
    /// a single asterisk (`"*"`), which acts as an "All files" filter.
    pub pattern: &'a str,
}

/// Errors reported by the dialog functions, either when requesting a dialog or
/// when its result is delivered to the [`DialogCallback`].
#[derive(Debug, Clone)]
pub enum DialogError {
    /// A filter's [`name`](DialogFileFilter::name) or
    /// [`pattern`](DialogFileFilter::pattern) contained an interior NUL byte
    /// and could not be passed to SDL. Returned directly by
    /// [`show_open_file_dialog`] and [`show_save_file_dialog`].
    FilterError(NulError),
    /// One of the paths chosen by the user was not valid UTF-8. Passed to the
    /// callback.
    InvalidFilename(Utf8Error),
    /// The user closed the dialog without choosing anything. Passed to the
    /// callback.
    Canceled,
    /// SDL failed to show the dialog or to collect its result. Passed to the
    /// callback.
    SdlError(Error),
}

impl fmt::Display for DialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::DialogError::*;

        match *self {
            FilterError(ref e) => write!(f, "Could not create filter: {e}"),
            InvalidFilename(ref e) => write!(f, "Invalid filename: {e}"),
            Canceled => write!(f, "Canceled"),
            SdlError(ref e) => write!(f, "SDL error: {e}"),
        }
    }
}

/// Callback invoked with the outcome of a dialog.
///
/// The first argument is the list of paths the user chose, or a
/// [`DialogError`] if the dialog was canceled ([`DialogError::Canceled`]), a
/// path was not valid UTF-8 ([`DialogError::InvalidFilename`]), or SDL
/// reported an error ([`DialogError::SdlError`]). Unless the dialog was shown
/// with `allow_many` the list contains a single path.
///
/// The second argument is the filter the user selected, if any. It is `None`
/// when no filters were supplied, when the dialog was canceled or failed, or
/// when the platform does not report the selected filter (Linux portals, for
/// instance, do not).
///
/// The callback may be invoked from a thread other than the one that requested
/// the dialog, depending on the operating system.
///
/// On Android the paths are `content://` URIs rather than filesystem paths.
#[doc(alias = "SDL_DialogFileCallback")]
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
            ((*callback_info_ptr).callback)(Ok(files), None)
        } else {
            // Seemingly not implemented in linux portals, untested
            if let Some(filter_strings) = &(*callback_info_ptr).filter_strings {
                if let Some(filter) = filter_strings.get(filter as usize) {
                    let filter = DialogFileFilter {
                        // We created these from strs, they cannot fail
                        name: filter.0.to_str().unwrap(),
                        pattern: filter.1.to_str().unwrap(),
                    };
                    ((*callback_info_ptr).callback)(Ok(files), Some(filter))
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

/// Displays a dialog that lets the user select one or more files on their
/// filesystem.
///
/// This function returns immediately; the result is delivered later to
/// `callback`, possibly from a different thread. Depending on the platform, the
/// user may be allowed to enter paths that don't exist yet.
///
/// # Arguments
///
/// * `filters` - File type filters offered to the user. May be empty. Not all
///   platforms support filters, and those that do may let the user ignore them.
/// * `default_location` - The folder or file the dialog should start at. Not
///   all platforms support this option.
/// * `allow_many` - If `true`, the user may select several files at once. Not
///   all platforms support this option.
/// * `window` - The window the dialog should be modal for, or `None`. Not all
///   platforms support this option.
/// * `callback` - Invoked once with the chosen paths, or with a
///   [`DialogError`] if the user canceled or an error occurred.
///
/// # Errors
///
/// Returns [`DialogError::FilterError`] if a filter's name or pattern contains
/// an interior NUL byte. Errors that happen after the dialog has been requested
/// are reported to `callback` instead.
///
/// # Panics
///
/// Panics if `default_location` is not valid UTF-8 or contains an interior
/// NUL byte.
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

/// Displays a dialog that lets the user select one or more folders on their
/// filesystem.
///
/// This function returns immediately; the result is delivered later to
/// `callback`, possibly from a different thread. Depending on the platform, the
/// user may be allowed to enter paths that don't exist yet.
///
/// Folder dialogs have no filters, so the callback's filter argument is always
/// `None`.
///
/// # Arguments
///
/// * `default_location` - The folder the dialog should start at. Not all
///   platforms support this option.
/// * `allow_many` - If `true`, the user may select several folders at once.
///   Not all platforms support this option.
/// * `window` - The window the dialog should be modal for, or `None`. Not all
///   platforms support this option.
/// * `callback` - Invoked once with the chosen paths, or with a
///   [`DialogError`] if the user canceled or an error occurred.
///
/// # Panics
///
/// Panics if `default_location` is not valid UTF-8 or contains an interior
/// NUL byte.
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

/// Displays a dialog that lets the user choose a new or existing file on their
/// filesystem to save to.
///
/// This function returns immediately; the result is delivered later to
/// `callback`, possibly from a different thread. The chosen file may or may not
/// already exist; nothing is written to it by SDL.
///
/// # Arguments
///
/// * `filters` - File type filters offered to the user. May be empty. Not all
///   platforms support filters, and those that do may let the user ignore them.
/// * `default_location` - The folder or file the dialog should start at. Not
///   all platforms support this option.
/// * `window` - The window the dialog should be modal for, or `None`. Not all
///   platforms support this option.
/// * `callback` - Invoked once with the chosen path, or with a
///   [`DialogError`] if the user canceled or an error occurred.
///
/// # Errors
///
/// Returns [`DialogError::FilterError`] if a filter's name or pattern contains
/// an interior NUL byte. Errors that happen after the dialog has been requested
/// are reported to `callback` instead.
///
/// # Panics
///
/// Panics if `default_location` is not valid UTF-8 or contains an interior
/// NUL byte.
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
