use super::util::*;

use sys::gpu::{
    SDL_GPUBuffer,
    SDL_GPUBufferCreateInfo,
    SDL_CreateGPUBuffer,
    SDL_ReleaseGPUBuffer,
};

#[doc(alias = "SDL_GPUBuffer")]
pub struct Buffer<'gpu> {
    raw: NonNull<Extern<SDL_GPUBuffer>>,
    device: &'gpu ExternDevice,
    len: u32,
}

impl<'gpu> Drop for Buffer<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUBuffer")]
    fn drop(&mut self) {
        unsafe {
            SDL_ReleaseGPUBuffer(self.device.raw(), self.raw());
        }
    }
}

impl<'gpu> Deref for Buffer<'gpu> {
    type Target = Extern<SDL_GPUBuffer>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> Buffer<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUBufferCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUBuffer(device.raw(), create_info)
        })?;
        Ok(Buffer {
            raw,
            device,
            len: create_info.size,
        })
    }
    
    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}
