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

    fn to_ll(&self) -> u32 {
        match *self {
            Category::Application => SDL_LOG_CATEGORY_APPLICATION.0 as u32,
            Category::Error => SDL_LOG_CATEGORY_ERROR.0 as u32,
            Category::Assert => SDL_LOG_CATEGORY_ASSERT.0 as u32,
            Category::System => SDL_LOG_CATEGORY_SYSTEM.0 as u32,
            Category::Audio => SDL_LOG_CATEGORY_AUDIO.0 as u32,
            Category::Video => SDL_LOG_CATEGORY_VIDEO.0 as u32,
            Category::Render => SDL_LOG_CATEGORY_RENDER.0 as u32,
            Category::Input => SDL_LOG_CATEGORY_INPUT.0 as u32,
            Category::Test => SDL_LOG_CATEGORY_TEST.0 as u32,
            Category::Gpu => SDL_LOG_CATEGORY_GPU.0 as u32,
            Category::Custom => SDL_LOG_CATEGORY_CUSTOM.0 as u32,
            // Only the application uses this category
            Category::Unknown => SDL_LOG_CATEGORY_APPLICATION.0 as u32,
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
            SDL_LOG_PRIORITY_CRITICAL => Priority::Critical,
            _ => Priority::Critical,
        }
    }

    fn to_ll(&self) -> u32 {
        match *self {
            Priority::Verbose => SDL_LOG_PRIORITY_VERBOSE.0 as u32,
            Priority::Debug => SDL_LOG_PRIORITY_DEBUG.0 as u32,
            Priority::Info => SDL_LOG_PRIORITY_INFO.0 as u32,
            Priority::Warn => SDL_LOG_PRIORITY_WARN.0 as u32,
            Priority::Error => SDL_LOG_PRIORITY_ERROR.0 as u32,
            Priority::Critical => SDL_LOG_PRIORITY_CRITICAL.0 as u32,
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
    custom_log_fn(priority, category, &message);
}

#[doc(alias = "SDL_SetLogOutputFunction")]
pub fn set_output_function(callback: fn(Priority, Category, &str)) {
    unsafe {
        custom_log_fn = callback;
        sys::log::SDL_SetLogOutputFunction(Some(rust_sdl2_log_fn), null_mut());
    };
}

#[doc(alias = "SDL_SetLogPriorities")]
pub fn set_log_priorities(priority: Priority) {
    let priority = priority.to_ll();
    unsafe {
        crate::sys::log::SDL_SetLogPriorities(SDL_LogPriority(priority as i32));
    }
}

#[doc(alias = "SDL_SetLogPriority")]
pub fn set_log_priority(category: Category, priority: Priority) {
    let category = category.to_ll();
    let priority = priority.to_ll();
    unsafe {
        crate::sys::log::SDL_SetLogPriority(category as i32, SDL_LogPriority(priority as i32));
    }
}

#[doc(alias = "SDL_GetLogPriority")]
pub fn get_log_priority(category: Category) -> Priority {
    let category = category.to_ll();
    unsafe { Priority::from_ll(crate::sys::log::SDL_GetLogPriority(category as i32)) }
}

#[doc(alias = "SDL_ResetLogPriorities")]
pub fn reset_log_priorities() {
    unsafe {
        crate::sys::log::SDL_ResetLogPriorities();
    }
}

#[doc(alias = "SDL_SetLogPriorityPrefix")]
pub fn set_log_priority_prefix(priority: Priority, prefix: &str) {
    let prefix = CString::new(prefix).unwrap();
    let priority = priority.to_ll();
    unsafe {
        crate::sys::log::SDL_SetLogPriorityPrefix(
            SDL_LogPriority(priority as i32),
            prefix.into_raw(),
        );
    }
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

#[doc(alias = "SDL_LogTrace")]
pub fn log_trace(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogTrace(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogVerbose")]
pub fn log_verbose(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogVerbose(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogDebug")]
pub fn log_debug(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogDebug(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogInfo")]
pub fn log_info(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogInfo(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogWarn")]
pub fn log_warn(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogWarn(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogError")]
pub fn log_error(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogError(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogCritical")]
pub fn log_critical(category: Category, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    unsafe {
        crate::sys::log::SDL_LogCritical(category as i32, message.into_raw());
    }
}

#[doc(alias = "SDL_LogMessage")]
pub fn log_message(category: Category, priority: Priority, message: &str) {
    let message = message.replace('%', "%%");
    let message = CString::new(message).unwrap();
    let category = category.to_ll();
    let priority = priority.to_ll();
    unsafe {
        crate::sys::log::SDL_LogMessage(
            category as i32,
            SDL_LogPriority(priority as i32),
            message.into_raw(),
        );
    }
}
