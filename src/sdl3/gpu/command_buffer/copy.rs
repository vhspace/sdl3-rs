use crate::gpu::{
    info_struct::{BufferLocation, TextureLocation},
    BufferRegion, Extern, TextureRegion, TextureTransferInfo, TransferBufferLocation,
};

use sys::gpu::{
    SDL_CopyGPUBufferToBuffer, SDL_CopyGPUTextureToTexture, SDL_DownloadFromGPUBuffer,
    SDL_DownloadFromGPUTexture, SDL_GPUCopyPass, SDL_UploadToGPUBuffer, SDL_UploadToGPUTexture,
};

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
    pub fn upload_to_gpu_texture(&self, src: TextureTransferInfo, dst: TextureRegion, cycle: bool) {
        unsafe { SDL_UploadToGPUTexture(self.ll(), &src.inner, &dst.inner, cycle) }
    }

    #[doc(alias = "SDL_CopyGPUTextureToTexture")]
    pub fn copy_texture_to_texture(
        &self,
        src: TextureLocation<'_>,
        dst: TextureLocation<'_>,
        w: u32,
        h: u32,
        d: u32,
        cycle: bool,
    ) {
        unsafe { SDL_CopyGPUTextureToTexture(self.ll(), &src.inner, &dst.inner, w, h, d, cycle) }
    }

    #[doc(alias = "SDL_CopyGPUBufferToBuffer")]
    pub fn copy_buffer_to_buffer(
        &self,
        src: BufferLocation<'_>,
        dst: BufferLocation<'_>,
        size: u32,
        cycle: bool,
    ) {
        unsafe { SDL_CopyGPUBufferToBuffer(self.ll(), &src.inner, &dst.inner, size, cycle) }
    }

    /// Note: The data is not guaranteed to be copied until the command buffer fence is signaled.
    #[doc(alias = "SDL_DownloadFromGPUBuffer")]
    pub fn download_from_gpu_buffer(&self, src: BufferRegion<'_>, dst: TransferBufferLocation<'_>) {
        unsafe { SDL_DownloadFromGPUBuffer(self.ll(), &src.inner, &dst.inner) }
    }

    /// Note: The data is not guaranteed to be copied until the command buffer fence is signaled.
    #[doc(alias = "SDL_DownloadFromGPUTexture")]
    pub fn download_from_gpu_texture(&self, src: TextureRegion<'_>, dst: TextureTransferInfo<'_>) {
        unsafe { SDL_DownloadFromGPUTexture(self.ll(), &src.inner, &dst.inner) }
    }
}
