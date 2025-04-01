use super::util::*;

use sys::gpu::{
    SDL_GPUSampler,
    SDL_GPUSamplerCreateInfo,
    SDL_CreateGPUSampler,
    SDL_ReleaseGPUSampler
};

#[doc(alias = "SDL_GPUSampler")]
pub struct Sampler<'gpu> {
    raw: NonNull<Extern<SDL_GPUSampler>>,
    device: &'gpu ExternDevice,
}

impl<'gpu> Drop for Sampler<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUSampler")]
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUSampler(self.device.raw(), self.raw()) }
    }
}

impl<'gpu> Deref for Sampler<'gpu> {
    type Target = Extern<SDL_GPUSampler>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> Sampler<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUSamplerCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUSampler(device.raw(), create_info)
        })?;
        Ok(Sampler {
            raw,
            device,
        })
    }
}
