use super::util::*;

use sys::gpu::{
    SDL_GPUShader,
    SDL_GPUShaderCreateInfo,
    SDL_CreateGPUShader,
    SDL_ReleaseGPUShader,
};

#[doc(alias = "SDL_GPUShader")]
pub struct Shader<'gpu> {
    raw: NonNull<Extern<SDL_GPUShader>>,
    device: &'gpu ExternDevice,
}

impl<'gpu> Drop for Shader<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUShader")]
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUShader(self.device.raw(), self.raw()) }
    }
}

impl<'gpu> Deref for Shader<'gpu> {
    type Target = Extern<SDL_GPUShader>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> Shader<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUShaderCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUShader(device.raw(), create_info)
        })?;
        Ok(Shader {
            raw,
            device,
        })
    }
}
