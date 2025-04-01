use super::util::*;

use sys::gpu::{
    SDL_GPUTexture,
    SDL_GPUTextureCreateInfo,
    SDL_CreateGPUTexture,
    SDL_ReleaseGPUTexture,
};

#[doc(alias = "SDL_Texture")]
pub struct Texture<'gpu> {
    pub(super) raw: NonNull<Extern<SDL_GPUTexture>>,
    // the GPU to release this from
    pub(super) device: &'gpu ExternDevice,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl<'gpu> Drop for Texture<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUTexture")]
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUTexture(self.device.raw(), self.raw()) };
    }
}

impl<'gpu> Deref for Texture<'gpu> {
    type Target = Extern<SDL_GPUTexture>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> Texture<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUTextureCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUTexture(device.raw(), create_info)
        })?;
        Ok(Texture {
            raw,
            width: create_info.width,
            height: create_info.height,
            device,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
