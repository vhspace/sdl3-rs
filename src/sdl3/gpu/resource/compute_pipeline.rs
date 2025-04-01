use super::util::*;

use sys::gpu::{
    SDL_GPUComputePipeline,
    SDL_GPUComputePipelineCreateInfo,
    SDL_CreateGPUComputePipeline,
    SDL_ReleaseGPUComputePipeline,
};

#[doc(alias = "SDL_GPUComputePipeline")]
pub struct ComputePipeline<'gpu> {
    raw: NonNull<Extern<SDL_GPUComputePipeline>>,
    device: &'gpu ExternDevice,
}

impl<'gpu> Drop for ComputePipeline<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUComputePipeline")]
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUComputePipeline(self.device.raw(), self.raw()) }
    }
}

impl<'gpu> Deref for ComputePipeline<'gpu> {
    type Target = Extern<SDL_GPUComputePipeline>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> ComputePipeline<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUComputePipelineCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUComputePipeline(device.raw(), create_info)
        })?;
        Ok(ComputePipeline {
            raw,
            device,
        })
    }
}
