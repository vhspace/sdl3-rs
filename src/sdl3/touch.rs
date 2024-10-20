use crate::sys;
use std::ffi::c_void;
use sys::stdinc::SDL_free;
use sys::touch::SDL_TouchID;

pub type Finger = sys::touch::SDL_Finger;
pub struct TouchId(SDL_TouchID);

impl TouchId {
    pub fn new(id: SDL_TouchID) -> Self {
        Self(id)
    }
}

impl From<TouchId> for SDL_TouchID {
    fn from(touch_id: TouchId) -> Self {
        touch_id.0
    }
}

/// Get a list of registered touch devices.
#[doc(alias = "SDL_GetTouchDevices")]
pub fn num_touch_devices() -> Vec<TouchId> {
    let mut count = 0;
    let ids = unsafe { sys::touch::SDL_GetTouchDevices(&mut count) };

    if ids.is_null() {
        // Return empty Vec if ids is null
        return Vec::new();
    }

    let count = count as usize;

    if count == 0 {
        // If count is zero, return an empty vector
        unsafe {
            SDL_free(ids as *mut c_void);
        }
        return Vec::new();
    }

    let slice = unsafe { std::slice::from_raw_parts(ids, count) };

    let touch_ids = slice.iter().cloned().map(TouchId).collect::<Vec<_>>();

    unsafe {
        SDL_free(ids as *mut c_void);
    }

    touch_ids
}

#[doc(alias = "SDL_GetTouchFingers")]
pub fn num_touch_fingers(touch: TouchId) -> i32 {
    let mut count = 0;
    unsafe {
        sys::touch::SDL_GetTouchFingers(touch.into(), &mut count);
    }
    count
}
