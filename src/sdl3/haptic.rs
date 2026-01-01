//! Haptic Functions

use crate::sys;
use sys::joystick::SDL_OpenJoystick;

use crate::common::IntegerOrSdlError;
use crate::get_error;
use crate::HapticSubsystem;

impl HapticSubsystem {
    /// Attempt to open the joystick at index `joystick_index` and return its haptic device.
    #[doc(alias = "SDL_OpenJoystick")]
    pub fn open_from_joystick_id(&self, joystick_index: u32) -> Result<Haptic, IntegerOrSdlError> {
        use crate::common::IntegerOrSdlError::*;

        let haptic = unsafe {
            let joystick = SDL_OpenJoystick(sys::joystick::SDL_JoystickID(joystick_index));
            sys::haptic::SDL_OpenHapticFromJoystick(joystick)
        };

        if haptic.is_null() {
            Err(SdlError(get_error()))
        } else {
            unsafe { sys::haptic::SDL_InitHapticRumble(haptic) };
            Ok(Haptic {
                subsystem: self.clone(),
                raw: haptic,
            })
        }
    }
}

/// Wrapper around the `SDL_Haptic` object
pub struct Haptic {
    subsystem: HapticSubsystem,
    raw: *mut sys::haptic::SDL_Haptic,
}

impl Haptic {
    #[inline]
    #[doc(alias = "SDL_HapticRumblePlay")]
    pub fn subsystem(&self) -> &HapticSubsystem {
        &self.subsystem
    }

    /// Run a simple rumble effect on the haptic device.
    pub fn rumble_play(&mut self, strength: f32, duration: u32) {
        unsafe { sys::haptic::SDL_PlayHapticRumble(self.raw, strength, duration) };
    }

    /// Stop the simple rumble on the haptic device.
    #[doc(alias = "SDL_HapticRumbleStop")]
    pub fn rumble_stop(&mut self) {
        unsafe { sys::haptic::SDL_StopHapticRumble(self.raw) };
    }
}

impl Drop for Haptic {
    #[doc(alias = "SDL_HapticClose")]
    fn drop(&mut self) {
        unsafe { sys::haptic::SDL_CloseHaptic(self.raw) }
    }
}
