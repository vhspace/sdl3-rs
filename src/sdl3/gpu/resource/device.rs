
use super::util::*;

use super::super::ShaderFormat;

use sys::gpu::{
    SDL_GPUDevice,
    SDL_CreateGPUDevice,
    SDL_DestroyGPUDevice
};

pub struct Device {
    raw: NonNull<Extern<SDL_GPUDevice>>,
}

impl Device {
    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(flags: ShaderFormat, debug_mode: bool) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUDevice(flags.0, debug_mode, std::ptr::null())
        })?.cast();
        Ok(Self { raw })
    }
}

impl Drop for Device {
    #[doc(alias = "SDL_DestroyGPUDevice")]
    fn drop(&mut self) {
        unsafe { SDL_DestroyGPUDevice(self.raw()) }
    }
}

impl Deref for Device {
    type Target = Extern<SDL_GPUDevice>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

