use crate::get_error;

use super::util::*;

use sys::gpu::{
    SDL_CreateGPUTransferBuffer, SDL_GPUTransferBuffer, SDL_GPUTransferBufferCreateInfo, SDL_MapGPUTransferBuffer, SDL_ReleaseGPUTransferBuffer, SDL_UnmapGPUTransferBuffer
};

#[doc(alias = "SDL_GPUTransferBuffer")]
pub struct TransferBuffer<'gpu> {
    raw: NonNull<Extern<SDL_GPUTransferBuffer>>,
    device: &'gpu ExternDevice,
    len: u32,
}

impl<'gpu> Drop for TransferBuffer<'gpu> {
    #[doc(alias = "SDL_ReleaseGPUTransferBuffer")]
    fn drop(&mut self) {
        unsafe {
            SDL_ReleaseGPUTransferBuffer(self.device.raw(), self.raw());
        }
    }
}

impl<'gpu> Deref for TransferBuffer<'gpu> {
    type Target = Extern<SDL_GPUTransferBuffer>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}


impl<'gpu> TransferBuffer<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice, create_info: &'_ SDL_GPUTransferBufferCreateInfo) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_CreateGPUTransferBuffer(device.raw(), create_info)
        })?;
        Ok(TransferBuffer {
            raw,
            device,
            len: create_info.size,
        })
    }

    #[doc(alias = "SDL_MapGPUTransferBuffer")]
    pub fn mapped_mut<R>(
        &mut self,
        cycle: bool,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> R,
    ) -> Result<R, Error> {
        unsafe {
            let raw = SDL_MapGPUTransferBuffer(self.device.raw(), self.raw(), cycle);
            if raw.is_null() {
                return Err(get_error());
            }

            let bytes = std::slice::from_raw_parts_mut(raw as *mut u8, self.len as usize);

            let _defer = Defer::new(||
                SDL_UnmapGPUTransferBuffer(self.device.raw(), self.raw())
            );
    
            Ok(f(bytes))
        }
    }

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}
