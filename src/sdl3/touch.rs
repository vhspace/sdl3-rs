use crate::sys;

pub type Finger = sys::touch::SDL_Finger;
pub type TouchId = sys::touch::SDL_TouchID;

/// Get a list of registered touch devices.
#[doc(alias = "SDL_GetTouchDevices")]
pub fn num_touch_devices() -> Vec<TouchId> {
    let mut count = 0;
    let ids = unsafe { sys::touch::SDL_GetTouchDevices(&mut count) };

    (0..count).map(|i| ids[i]).collect()
}

#[doc(alias = "SDL_GetTouchFingers")]
pub fn num_touch_fingers(touch: TouchId) -> i32 {
    let mut count = 0;
    unsafe {
        sys::touch::SDL_GetTouchFingers(touch, &mut count);
    }
    count
}
