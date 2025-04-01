
use std::{ops::Deref, ptr::NonNull};

use crate::gpu::{
    BufferRegion, TextureTransferInfo, TransferBufferLocation,
};
use sys::gpu::{
    SDL_GPUCopyPass,
    SDL_UploadToGPUBuffer,
    SDL_UploadToGPUTexture,
};

use super::{Extern, TextureRegion};

pub struct CopyPass {
    pub(super) raw: NonNull<Extern<SDL_GPUCopyPass>>,
}

impl<'gpu> Deref for CopyPass {
    type Target = Extern<SDL_GPUCopyPass>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl CopyPass {
    #[doc(alias = "SDL_UploadToGPUBuffer")]
    pub fn upload_to_gpu_buffer(
        &self,
        transfer_buf_location: TransferBufferLocation,
        buffer_region: BufferRegion,
        cycle: bool,
    ) {
        unsafe {
            SDL_UploadToGPUBuffer(
                self.raw(),
                &transfer_buf_location.inner,
                &buffer_region.inner,
                cycle,
            )
        }
    }

    #[doc(alias = "SDL_UploadToGPUTexture")]
    pub fn upload_to_gpu_texture(
        &self,
        source: TextureTransferInfo,
        destination: TextureRegion,
        cycle: bool,
    ) {
        unsafe { SDL_UploadToGPUTexture(self.raw(), &source.inner, &destination.inner, cycle) }
    }
}
