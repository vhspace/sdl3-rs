use crate::gpu::{
    BufferRegion, Extern, TextureRegion, TextureTransferInfo, TransferBufferLocation,
};

use sys::gpu::{SDL_GPUCopyPass, SDL_UploadToGPUBuffer, SDL_UploadToGPUTexture};

pub type CopyPass = Extern<SDL_GPUCopyPass>;

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
                self.ll(),
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
        unsafe { SDL_UploadToGPUTexture(self.ll(), &source.inner, &destination.inner, cycle) }
    }
}
