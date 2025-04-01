
use std::{ops::Deref, ptr::NonNull};

use sys::gpu::{
    SDL_BindGPUComputePipeline,
    SDL_BindGPUComputeStorageBuffers,
    SDL_BindGPUComputeStorageTextures,
    SDL_DispatchGPUCompute,
    SDL_GPUBuffer, SDL_GPUComputePass, SDL_GPUTexture
};

use super::{ComputePipeline, Extern};

pub struct ComputePass {
    pub(super) raw: NonNull<Extern<SDL_GPUComputePass>>,
}

impl<'gpu> Deref for ComputePass {
    type Target = Extern<SDL_GPUComputePass>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl ComputePass {
    #[doc(alias = "SDL_BindGPUComputePipeline")]
    pub fn bind_compute_pipeline(&self, pipeline: &ComputePipeline) {
        unsafe { SDL_BindGPUComputePipeline(self.raw(), pipeline.raw()) }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageBuffers")]
    pub fn bind_compute_storage_buffers(&self, first_slot: u32, storage_buffers: &[&Extern<SDL_GPUBuffer>]) {
        unsafe {
            SDL_BindGPUComputeStorageBuffers(
                self.raw(),
                first_slot,
                storage_buffers.as_ptr().cast(),
                storage_buffers.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageTextures")]
    pub fn bind_compute_storage_textures(&self, first_slot: u32, storage_textures: &[&Extern<SDL_GPUTexture>]) {
        unsafe {
            SDL_BindGPUComputeStorageTextures(
                self.raw(),
                first_slot,
                storage_textures.as_ptr().cast(),
                storage_textures.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_DispatchGPUCompute")]
    pub fn dispatch(&self, groupcount_x: u32, groupcount_y: u32, groupcount_z: u32) {
        unsafe {
            SDL_DispatchGPUCompute(self.raw(), groupcount_x, groupcount_y, groupcount_z)
        }
    }
}
