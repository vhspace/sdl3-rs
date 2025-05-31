use sys::gpu::{
    SDL_BindGPUComputePipeline, SDL_BindGPUComputeSamplers, SDL_BindGPUComputeStorageBuffers,
    SDL_BindGPUComputeStorageTextures, SDL_DispatchGPUCompute, SDL_DispatchGPUComputeIndirect,
    SDL_GPUComputePass,
};

use crate::gpu::{
    info_struct::IndirectDispatchCommand, Buffer, ComputePipeline, Extern, Texture,
    TextureSamplerBinding,
};

pub type ComputePass = Extern<SDL_GPUComputePass>;

impl ComputePass {
    #[doc(alias = "SDL_BindGPUComputePipeline")]
    pub fn bind_compute_pipeline(&self, pipeline: &ComputePipeline) {
        unsafe { SDL_BindGPUComputePipeline(self.ll(), pipeline.ll()) }
    }

    #[doc(alias = "SDL_BindGPUComputeSamplers")]
    pub fn bind_compute_samplers(&self, first_slot: u32, samplers: &[&TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUComputeSamplers(
                self.ll(),
                first_slot,
                samplers.as_ptr().cast(),
                samplers.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageBuffers")]
    pub fn bind_compute_storage_buffers(&self, first_slot: u32, storage_buffers: &[&Buffer]) {
        unsafe {
            SDL_BindGPUComputeStorageBuffers(
                self.ll(),
                first_slot,
                storage_buffers.as_ptr().cast(),
                storage_buffers.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageTextures")]
    pub fn bind_compute_storage_textures(&self, first_slot: u32, storage_textures: &[&Texture]) {
        unsafe {
            SDL_BindGPUComputeStorageTextures(
                self.ll(),
                first_slot,
                storage_textures.as_ptr().cast(),
                storage_textures.len() as u32,
            )
        }
    }

    /// Dispatch compute work
    #[doc(alias = "SDL_DispatchGPUCompute")]
    pub fn dispatch(&self, groupcount_x: u32, groupcount_y: u32, groupcount_z: u32) {
        unsafe { SDL_DispatchGPUCompute(self.ll(), groupcount_x, groupcount_y, groupcount_z) }
    }

    /// Dispatch compute work. Same as `dispatch`, except the dispatch parameters are read from GPU memory.
    #[doc(alias = "SDL_DispatchGPUComputeIndirect")]
    pub fn dispatch_indirect(&self, dispatch: crate::gpu::Ref<'_, IndirectDispatchCommand>) {
        unsafe { SDL_DispatchGPUComputeIndirect(self.ll(), dispatch.buf.ll(), dispatch.offset) }
    }
}
