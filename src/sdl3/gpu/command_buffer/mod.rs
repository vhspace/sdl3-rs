use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{get_error, gpu::util::nonnull_ext_or_get_error, Error};

use super::{
    util::Defer, ColorTargetInfo, DepthStencilTargetInfo, Device, Extern,
    StorageBufferReadWriteBinding, StorageTextureReadWriteBinding,
};

use sys::gpu::{
    SDL_AcquireGPUCommandBuffer, SDL_AcquireGPUSwapchainTexture, SDL_BeginGPUComputePass,
    SDL_BeginGPUCopyPass, SDL_BeginGPURenderPass, SDL_GPUColorTargetInfo, SDL_GPUCommandBuffer,
    SDL_GPUDevice, SDL_PushGPUComputeUniformData,
    SDL_PushGPUFragmentUniformData, SDL_PushGPUVertexUniformData,
    SDL_WaitAndAcquireGPUSwapchainTexture,
};

mod compute;
pub use compute::ComputePass;

mod render;
pub use render::RenderPass;

mod copy;
pub use copy::CopyPass;

mod swapchain;
pub use swapchain::SwapchainTexture;

mod fence;
pub use fence::Fence;

pub type CommandBuffer = Extern<sys::gpu::SDL_GPUCommandBuffer>;

#[repr(transparent)]
pub struct OwnedCommandBuffer<'gpu> {
    pub(super) raw: NonNull<Extern<SDL_GPUCommandBuffer>>,
    pub(super) _marker: std::marker::PhantomData<&'gpu SDL_GPUDevice>,
}

impl<'gpu> Deref for OwnedCommandBuffer<'gpu> {
    type Target = CommandBuffer;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> DerefMut for OwnedCommandBuffer<'gpu> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut() }
    }
}

impl<'gpu> OwnedCommandBuffer<'gpu> {
    pub(crate) fn new(device: &'gpu Device) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe { SDL_AcquireGPUCommandBuffer(device.ll()) })?;
        Ok(OwnedCommandBuffer {
            raw,
            _marker: PhantomData,
        })
    }

    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<(), Error> {
        if unsafe { sys::gpu::SDL_SubmitGPUCommandBuffer(self.ll()) } {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(self) {
        unsafe {
            sys::gpu::SDL_CancelGPUCommandBuffer(self.ll());
        }
    }
}

impl CommandBuffer {
    #[doc(alias = "SDL_BeginGPUComputePass")]
    pub fn compute_pass<R>(
        &mut self,
        storage_texture_bindings: &[StorageTextureReadWriteBinding],
        storage_buffer_bindings: &[StorageBufferReadWriteBinding],
        func: impl for<'a> FnOnce(&'a Extern<SDL_GPUCommandBuffer>, &'a mut ComputePass) -> R,
    ) -> Result<R, Error> {
        let mut raw = nonnull_ext_or_get_error(unsafe {
            SDL_BeginGPUComputePass(
                self.ll(),
                storage_texture_bindings.as_ptr().cast(),
                storage_texture_bindings.len() as u32,
                storage_buffer_bindings.as_ptr().cast(),
                storage_buffer_bindings.len() as u32,
            )
        })?;

        let _defer = Defer::new(move || unsafe {
            sys::gpu::SDL_EndGPUComputePass(raw.as_ptr().cast());
        });

        Ok(unsafe { func(self, raw.as_mut()) })
    }

    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn render_pass<R>(
        &mut self,
        color_info: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
        func: impl for<'a> FnOnce(&'a CommandBuffer, &'a mut RenderPass) -> R,
    ) -> Result<R, Error> {
        let mut raw = nonnull_ext_or_get_error(unsafe {
            SDL_BeginGPURenderPass(
                self.ll(),
                color_info.as_ptr() as *const SDL_GPUColorTargetInfo,
                color_info.len() as u32,
                depth_stencil_target
                    .map(std::ptr::from_ref)
                    .unwrap_or(std::ptr::null())
                    .cast(),
            )
        })?;

        let _defer = Defer::new(move || unsafe {
            sys::gpu::SDL_EndGPURenderPass(raw.as_ptr().cast());
        });

        Ok(unsafe { func(self, raw.as_mut()) })
    }

    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn copy_pass<R>(
        &mut self,
        func: impl for<'a> FnOnce(&'a Extern<SDL_GPUCommandBuffer>, &'a mut CopyPass) -> R,
    ) -> Result<R, Error> {
        let mut raw = nonnull_ext_or_get_error(unsafe { SDL_BeginGPUCopyPass(self.ll()) })?;

        let _defer = Defer::new(move || unsafe {
            sys::gpu::SDL_EndGPUCopyPass(raw.as_ptr().cast());
        });

        Ok(unsafe { func(self, raw.as_mut()) })
    }

    // FIXME:
    // The lifetime here isn't quite right.
    // The swapchain texture can only be used with the command buffer it
    // was obtained from, but we also can't borrow the command buffer here
    // without preventing you from running passes.
    #[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
    pub fn wait_and_acquire_swapchain_texture<'a>(
        &mut self,
        w: &'a crate::video::Window,
    ) -> Result<SwapchainTexture<'a>, Option<Error>> {
        let mut raw = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_WaitAndAcquireGPUSwapchainTexture(
                self.ll(),
                w.raw(),
                &mut raw,
                &mut width,
                &mut height,
            )
        };
        let raw = raw.cast();
        if success {
            if let Some(raw) = NonNull::new(raw) {
                Ok(SwapchainTexture {
                    raw,
                    width,
                    height,
                    _marker: PhantomData,
                })
            } else {
                Err(None)
            }
        } else {
            Err(Some(get_error()))
        }
    }

    #[doc(alias = "SDL_AcquireGPUSwapchainTexture")]
    pub fn acquire_swapchain_texture<'a>(
        &mut self,
        w: &'a crate::video::Window,
    ) -> Result<SwapchainTexture<'a>, Option<Error>> {
        let mut raw = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_AcquireGPUSwapchainTexture(self.ll(), w.raw(), &mut raw, &mut width, &mut height)
        };
        let raw = raw.cast();
        if success {
            if let Some(raw) = NonNull::new(raw) {
                Ok(SwapchainTexture {
                    raw,
                    width,
                    height,
                    _marker: PhantomData,
                })
            } else {
                Err(None)
            }
        } else {
            Err(Some(get_error()))
        }
    }

    #[doc(alias = "SDL_PushGPUVertexUniformData")]
    pub fn push_vertex_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.ll(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_PushGPUFragmentUniformData")]
    pub fn push_fragment_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUFragmentUniformData(
                self.ll(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_PushGPUComputeUniformData")]
    pub fn push_compute_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUComputeUniformData(
                self.ll(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }
}
