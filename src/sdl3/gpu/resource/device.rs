use std::{ops::Deref, ptr::NonNull};

use crate::{
    gpu::{util::nonnull_ext_or_get_error, Extern},
    Error,
};

use super::super::ShaderFormat;

use sys::gpu::{SDL_CreateGPUDevice, SDL_DestroyGPUDevice, SDL_GPUDevice};

pub type Device = Extern<SDL_GPUDevice>;

pub struct OwnedDevice {
    raw: NonNull<Device>,
}

impl OwnedDevice {
    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(flags: ShaderFormat, debug_mode: bool) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUDevice(flags.0, debug_mode, std::ptr::null())
        })?
        .cast();
        Ok(Self { raw })
    }
}

impl Drop for OwnedDevice {
    #[doc(alias = "SDL_DestroyGPUDevice")]
    fn drop(&mut self) {
        unsafe { SDL_DestroyGPUDevice(self.ll()) }
    }
}

impl Deref for OwnedDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}
