use libc::c_char;
use libc::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::ptr;
use std::str::Utf8Error;
use sys::properties::SDL_PropertiesID;

use crate::Error;
use crate::get_error;
use crate::sys;

#[derive(Debug)]
pub enum PropertiesError {
    ArgumentError(NulError),
    StringError(Utf8Error),
    NullPointer,
    SdlError(Error),
}

#[derive(Debug, Clone)]
pub struct Properties {
    internal: sys::properties::SDL_PropertiesID,
    global: bool,
}

macro_rules! cstring {
    ($name:ident) => {
        let $name = match CString::new($name) {
            Ok(name) => name,
            Err(error) => return Err(PropertiesError::ArgumentError(error)),
        };
    };
}

pub type EnumerateCallback = Box<dyn Fn(&Properties, Result<&str, PropertiesError>)>;
unsafe extern "C" fn enumerate(
    userdata: *mut c_void,
    props: SDL_PropertiesID,
    name: *const c_char,
) {
    let properties: &Properties = std::mem::transmute(&props);
    let callback_ptr = userdata as *mut EnumerateCallback;
    let name = CStr::from_ptr(name);
    match name.to_str() {
        Ok(name) => (*callback_ptr)(properties, Ok(name)),
        Err(error) => (*callback_ptr)(properties, Err(PropertiesError::StringError(error))),
    }
}

pub type CleanupBox = fn(*mut c_void);
unsafe extern "C" fn cleanup_box(userdata: *mut c_void, value: *mut c_void) {
    let callback_ptr = userdata as *mut CleanupBox;
    (*callback_ptr)(value);
}

pub type CleanupCallback = Box<dyn FnOnce(*mut c_void)>;
unsafe extern "C" fn cleanup_custom(userdata: *mut c_void, value: *mut c_void) {
    let callback_ptr = userdata as *mut CleanupCallback;
    let callback = Box::from_raw(callback_ptr);
    (*callback)(value);
}

pub use sys::properties::SDL_PropertyType as PropertyType;

impl Properties {
    #[doc(alias = "SDL_CreateProperties")]
    pub fn new() -> Result<Self, PropertiesError> {
        let internal = unsafe { sys::properties::SDL_CreateProperties() };
        if internal == 0 {
            Err(PropertiesError::SdlError(get_error()))
        } else {
            Ok(Self {
                internal,
                global: false,
            })
        }
    }

    #[doc(alias = "SDL_GetGlobalProperties")]
    pub fn global() -> Result<Self, PropertiesError> {
        let internal = unsafe { sys::properties::SDL_GetGlobalProperties() };
        if internal == 0 {
            Err(PropertiesError::SdlError(get_error()))
        } else {
            Ok(Self {
                internal,
                global: true,
            })
        }
    }

    #[doc(alias = "SDL_LockProperties")]
    pub fn lock(&mut self) -> Result<(), PropertiesError> {
        unsafe {
            if !sys::properties::SDL_LockProperties(self.internal) {
                return Err(PropertiesError::SdlError(get_error()));
            }
        }
        Ok(())
    }

    #[doc(alias = "SDL_UnlockProperties")]
    pub fn unlock(&mut self) {
        unsafe {
            sys::properties::SDL_UnlockProperties(self.internal);
        }
    }

    #[doc(alias = "SDL_HasProperty")]
    pub fn contains(&self, name: &str) -> Result<bool, PropertiesError> {
        cstring!(name);
        unsafe {
            Ok(sys::properties::SDL_HasProperty(
                self.internal,
                name.as_ptr(),
            ))
        }
    }

    #[doc(alias = "SDL_SetPointerPropertyWithCleanup")]
    pub fn set_with_cleanup<T>(
        &self,
        name: &str,
        value: *mut T,
        cleanup: Box<dyn FnOnce(*mut T)>,
    ) -> Result<(), PropertiesError> {
        cstring!(name);
        let value_ptr = value as *mut c_void;
        let cleanup_ptr = Box::into_raw(Box::new(cleanup)) as *mut c_void;
        if unsafe {
            sys::properties::SDL_SetPointerPropertyWithCleanup(
                self.internal,
                name.as_ptr(),
                value_ptr,
                Some(cleanup_custom),
                cleanup_ptr,
            )
        } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }

    #[doc(alias = "SDL_GetPropertyType")]
    pub fn get_type(&self, name: &str) -> Result<PropertyType, PropertiesError> {
        cstring!(name);
        unsafe {
            Ok(sys::properties::SDL_GetPropertyType(
                self.internal,
                name.as_ptr(),
            ))
        }
    }

    #[doc(alias = "SDL_GetStringProperty")]
    pub fn get_string(&self, name: &str, default: &str) -> Result<String, PropertiesError> {
        cstring!(name);
        cstring!(default);
        let value = unsafe {
            let value = sys::properties::SDL_GetStringProperty(
                self.internal,
                name.as_ptr(),
                default.as_ptr(),
            );
            CStr::from_ptr(value)
        };

        match value.to_str() {
            Ok(value) => Ok(String::from(value)),
            Err(error) => Err(PropertiesError::StringError(error)),
        }
    }

    #[doc(alias = "SDL_CopyProperties")]
    pub fn copy(&self, destination: &mut Self) -> Result<(), PropertiesError> {
        if unsafe { sys::properties::SDL_CopyProperties(self.internal, destination.internal) } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }

    #[doc(alias = "SDL_EnumerateProperties")]
    pub fn enumerate(&self, callback: EnumerateCallback) -> Result<(), PropertiesError> {
        let callback_ptr = Box::into_raw(Box::new(callback)) as *mut c_void;
        if unsafe {
            sys::properties::SDL_EnumerateProperties(self.internal, Some(enumerate), callback_ptr)
        } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }

    #[doc(alias = "SDL_ClearProperty")]
    pub fn clear(&mut self, name: &str) -> Result<(), PropertiesError> {
        cstring!(name);
        if unsafe { sys::properties::SDL_ClearProperty(self.internal, name.as_ptr()) } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }

    #[doc(alias = "SDL_GetPointerProperty")]
    pub fn with<T>(&mut self, name: &str, with: fn(&T)) -> Result<(), PropertiesError> {
        self.lock()?;
        let pointer: *mut T = self.get(name, ptr::null_mut())?;
        if pointer.is_null() {
            return Err(PropertiesError::NullPointer);
        }
        let reference = unsafe { &mut *pointer };
        with(reference);
        self.unlock();
        Ok(())
    }
}

pub trait Setter<T> {
    fn set(&self, name: &str, value: T) -> Result<(), PropertiesError>;
}

impl Setter<bool> for Properties {
    #[doc(alias = "SDL_SetBooleanProperty")]
    fn set(&self, name: &str, value: bool) -> Result<(), PropertiesError> {
        cstring!(name);
        if unsafe { sys::properties::SDL_SetBooleanProperty(self.internal, name.as_ptr(), value) } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

impl Setter<f32> for Properties {
    #[doc(alias = "SDL_SetFloatProperty")]
    fn set(&self, name: &str, value: f32) -> Result<(), PropertiesError> {
        cstring!(name);
        if unsafe { sys::properties::SDL_SetFloatProperty(self.internal, name.as_ptr(), value) } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

impl Setter<i64> for Properties {
    #[doc(alias = "SDL_SetNumberProperty")]
    fn set(&self, name: &str, value: i64) -> Result<(), PropertiesError> {
        cstring!(name);
        if unsafe { sys::properties::SDL_SetNumberProperty(self.internal, name.as_ptr(), value) } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

impl Setter<&str> for Properties {
    #[doc(alias = "SDL_SetStringProperty")]
    fn set(&self, name: &str, value: &str) -> Result<(), PropertiesError> {
        cstring!(name);
        // Have to transform the value into a cstring, SDL makes an internal copy
        cstring!(value);
        if unsafe {
            sys::properties::SDL_SetStringProperty(self.internal, name.as_ptr(), value.as_ptr())
        } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

impl<T> Setter<*mut T> for Properties {
    #[doc(alias = "SDL_SetPointerProperty")]
    fn set(&self, name: &str, value: *mut T) -> Result<(), PropertiesError> {
        cstring!(name);
        if unsafe {
            sys::properties::SDL_SetPointerProperty(
                self.internal,
                name.as_ptr(),
                value as *mut c_void,
            )
        } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

impl<T> Setter<Box<T>> for Properties {
    #[doc(alias = "SDL_SetPointerPropertyWithCleanup")]
    fn set(&self, name: &str, value: Box<T>) -> Result<(), PropertiesError> {
        cstring!(name);
        let value_ptr: *mut c_void = Box::into_raw(value) as *mut c_void;
        let cleanup: CleanupBox = |value: *mut c_void| {
            let value = value as *mut T;
            unsafe {
                drop(Box::from_raw(value));
            }
        };
        let cleanup_ptr = Box::into_raw(Box::new(cleanup)) as *mut c_void;
        if unsafe {
            sys::properties::SDL_SetPointerPropertyWithCleanup(
                self.internal,
                name.as_ptr(),
                value_ptr,
                Some(cleanup_box),
                cleanup_ptr,
            )
        } {
            Ok(())
        } else {
            Err(PropertiesError::SdlError(get_error()))
        }
    }
}

pub trait Getter<T> {
    fn get(&self, name: &str, default: T) -> Result<T, PropertiesError>;
}

impl Getter<bool> for Properties {
    #[doc(alias = "SDL_GetBooleanProperty")]
    fn get(&self, name: &str, default: bool) -> Result<bool, PropertiesError> {
        cstring!(name);
        unsafe {
            Ok(sys::properties::SDL_GetBooleanProperty(
                self.internal,
                name.as_ptr(),
                default,
            ))
        }
    }
}

impl Getter<f32> for Properties {
    #[doc(alias = "SDL_GetFloatProperty")]
    fn get(&self, name: &str, default: f32) -> Result<f32, PropertiesError> {
        cstring!(name);
        unsafe {
            Ok(sys::properties::SDL_GetFloatProperty(
                self.internal,
                name.as_ptr(),
                default,
            ))
        }
    }
}

impl Getter<i64> for Properties {
    #[doc(alias = "SDL_GetNumberProperty")]
    fn get(&self, name: &str, default: i64) -> Result<i64, PropertiesError> {
        cstring!(name);
        unsafe {
            Ok(sys::properties::SDL_GetNumberProperty(
                self.internal,
                name.as_ptr(),
                default,
            ))
        }
    }
}

impl<T> Getter<*mut T> for Properties {
    #[doc(alias = "SDL_GetPointerProperty")]
    fn get(&self, name: &str, default: *mut T) -> Result<*mut T, PropertiesError> {
        cstring!(name);
        let pointer = unsafe {
            sys::properties::SDL_GetPointerProperty(
                self.internal,
                name.as_ptr(),
                default as *mut c_void,
            )
        };
        if pointer.is_null() {
            return Err(PropertiesError::NullPointer);
        }
        Ok(pointer as *mut T)
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        if !self.global {
            unsafe {
                sys::properties::SDL_DestroyProperties(self.internal);
            }
        }
    }
}
