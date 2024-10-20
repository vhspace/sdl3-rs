use crate::sys;
use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr::null_mut;
use sys::log::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Application,
    Error,
    Assert,
    System,
    Audio,
    Video,
    Render,
    Input,
    Test,
    Gpu,
    Custom,
    Unknown,
}

impl Category {
    #[allow(dead_code)]
    fn from_ll(value: u32) -> Category {
        match unsafe { transmute::<u32, SDL_LogCategory>(value) } {
            SDL_LOG_CATEGORY_APPLICATION => Self::Application,
            SDL_LOG_CATEGORY_ERROR => Self::Error,
            SDL_LOG_CATEGORY_ASSERT => Self::Assert,
            SDL_LOG_CATEGORY_SYSTEM => Self::System,
            SDL_LOG_CATEGORY_AUDIO => Self::Audio,
            SDL_LOG_CATEGORY_VIDEO => Self::Video,
            SDL_LOG_CATEGORY_RENDER => Self::Render,
            SDL_LOG_CATEGORY_INPUT => Self::Input,
            SDL_LOG_CATEGORY_TEST => Self::Test,
            SDL_LOG_CATEGORY_GPU => Self::Gpu,
            SDL_LOG_CATEGORY_CUSTOM => Self::Custom,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Priority {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl Priority {
    fn from_ll(value: SDL_LogPriority) -> Priority {
        match value {
            SDL_LOG_PRIORITY_VERBOSE => Priority::Verbose,
            SDL_LOG_PRIORITY_DEBUG => Priority::Debug,
            SDL_LOG_PRIORITY_INFO => Priority::Info,
            SDL_LOG_PRIORITY_WARN => Priority::Warn,
            SDL_LOG_PRIORITY_ERROR => Priority::Error,
            SDL_LOG_PRIORITY_CRITICAL | _ => Priority::Critical,
        }
    }
}

fn dummy(_priority: Priority, _category: Category, _message: &str) {}

#[allow(non_upper_case_globals)]
// NEVER make this public
static mut custom_log_fn: fn(Priority, Category, &str) = dummy;

unsafe extern "C" fn rust_sdl2_log_fn(
    _userdata: *mut libc::c_void,
    category: libc::c_int,
    priority: SDL_LogPriority,
    message: *const libc::c_char,
) {
    let category = Category::from_ll(category as u32);
    let priority = Priority::from_ll(priority);
    let message = CStr::from_ptr(message).to_string_lossy();
    custom_log_fn(priority, category, &*message);
}

#[doc(alias = "SDL_SetLogOutputFunction")]
pub fn set_output_function(callback: fn(Priority, Category, &str)) {
    unsafe {
        custom_log_fn = callback;
        sys::log::SDL_SetLogOutputFunction(Some(rust_sdl2_log_fn), null_mut());
    };
}

/// Standard log function which takes as priority INFO and
/// as category APPLICATION
#[doc(alias = "SDL_Log")]
pub fn log(message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    unsafe {
        crate::sys::log::SDL_Log(message.into_raw());
    }
}
