use super::util::*;

use sys::gpu::{
    SDL_GPUGraphicsPipeline,
    SDL_GPUGraphicsPipelineCreateInfo,
    SDL_CreateGPUGraphicsPipeline,
    SDL_ReleaseGPUGraphicsPipeline,
};

#[doc(alias = "SDL_GPUGraphicsPipeline")]
pub struct GraphicsPipeline<'gpu> {
    raw: NonNull<Extern<SDL_GPUGraphicsPipeline>>,
    device: &'gpu ExternDevice,
}

impl<'gpu> Drop for GraphicsPipeline<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUGraphicsPipeline")]
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUGraphicsPipeline(self.device.raw(), self.raw()) }
    }
}

impl<'gpu> Deref for GraphicsPipeline<'gpu> {
    type Target = Extern<SDL_GPUGraphicsPipeline>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> GraphicsPipeline<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUGraphicsPipelineCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUGraphicsPipeline(device.raw(), create_info)
        })?;
        Ok(GraphicsPipeline {
            raw,
            device,
        })
    }
}
