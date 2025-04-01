
use std::{marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull};

use crate::{get_error, gpu::{util::nonnull_ext_or_get_error, ComputePass}, Error};

use super::{
    swapchain::SwapchainTexture, util::Defer, ColorTargetInfo, CopyPass,
    DepthStencilTargetInfo, Extern, ExternDevice, RenderPass,
    StorageBufferReadWriteBinding, StorageTextureReadWriteBinding
};

use sys::gpu::{
    SDL_AcquireGPUCommandBuffer, SDL_AcquireGPUSwapchainTexture,
    SDL_BeginGPUComputePass, SDL_BeginGPUCopyPass,
    SDL_BeginGPURenderPass, SDL_GPUColorTargetInfo,
    SDL_GPUCommandBuffer, SDL_GPUDepthStencilTargetInfo,
    SDL_GPUDevice, SDL_PushGPUComputeUniformData, SDL_PushGPUFragmentUniformData,
    SDL_PushGPUVertexUniformData, SDL_WaitAndAcquireGPUSwapchainTexture
};

#[repr(transparent)]
pub struct CommandBuffer<'gpu> {
    pub(super) raw: NonNull<Extern<SDL_GPUCommandBuffer>>,
    pub(super) _marker: std::marker::PhantomData<&'gpu SDL_GPUDevice>,
}

impl<'gpu> Deref for CommandBuffer<'gpu> {
    type Target = Extern<SDL_GPUCommandBuffer>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'gpu> DerefMut for CommandBuffer<'gpu> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut() }
    }
}

impl<'gpu> CommandBuffer<'gpu> {
    pub(crate) fn new(device: &'gpu ExternDevice) -> Result<Self, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_AcquireGPUCommandBuffer(device.raw())
        })?;
        Ok(CommandBuffer { raw, _marker: PhantomData } )
    }

    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<(), Error> {
        if unsafe { sys::gpu::SDL_SubmitGPUCommandBuffer(self.raw()) } {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(self) {
        unsafe {
            sys::gpu::SDL_CancelGPUCommandBuffer(self.raw());
        }
    }
}

impl Extern<SDL_GPUCommandBuffer> {
    #[doc(alias = "SDL_BeginGPUComputePass")]
    pub fn compute_pass<R>(
        &mut self,
        storage_texture_bindings: &[StorageTextureReadWriteBinding],
        storage_buffer_bindings: &[StorageBufferReadWriteBinding],
        pass: impl for<'a> FnOnce(&'a Extern<SDL_GPUCommandBuffer>, &'a mut ComputePass) -> R,
    ) -> Result<R, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_BeginGPUComputePass(
                self.raw(),
                storage_texture_bindings.as_ptr().cast(),
                storage_buffer_bindings.len() as u32,
                storage_buffer_bindings.as_ptr().cast(),
                storage_buffer_bindings.len() as u32,
            )
        })?;

        let _defer = Defer::new(|| unsafe {
            sys::gpu::SDL_EndGPUComputePass(raw.as_ptr().cast());
        });

        Ok(pass(&*self, &mut ComputePass { raw }))
    }

    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn render_pass<R>(
        &mut self,
        color_info: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
        pass: impl for<'a> FnOnce(&'a Extern<SDL_GPUCommandBuffer>, &'a mut RenderPass) -> R,
    ) -> Result<R, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_BeginGPURenderPass(
                self.raw(),
                color_info.as_ptr() as *const SDL_GPUColorTargetInfo, //heavy promise
                color_info.len() as u32,
                if let Some(p) = depth_stencil_target {
                    p as *const _ as *const SDL_GPUDepthStencilTargetInfo //heavy promise
                } else {
                    std::ptr::null()
                },
            )
        })?;

        let _defer = Defer::new(|| unsafe {
            sys::gpu::SDL_EndGPURenderPass(raw.as_ptr().cast());
        });

        Ok(pass(&*self, &mut RenderPass { raw }))
    }

    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn copy_pass<R>(
        &mut self,
        pass: impl for<'a> FnOnce(&'a Extern<SDL_GPUCommandBuffer>, &'a mut CopyPass) -> R,
    ) -> Result<R, Error> {
        let raw = nonnull_ext_or_get_error(unsafe {
            SDL_BeginGPUCopyPass(self.raw())
        })?;

        let _defer = Defer::new(|| unsafe {
            sys::gpu::SDL_EndGPUCopyPass(raw.as_ptr().cast());
        });

        Ok(pass(&*self, &mut CopyPass { raw }))
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
                self.raw(),
                w.raw(),
                &mut raw,
                &mut width,
                &mut height,
            )
        };
        let raw = raw.cast();
        if success {
            if let Some(raw) = NonNull::new(raw) {
                Ok(
                    SwapchainTexture {
                        raw,
                        width,
                        height,
                        _marker: PhantomData,
                    }
                )
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
            SDL_AcquireGPUSwapchainTexture(
                self.raw(),
                w.raw(),
                &mut raw,
                &mut width,
                &mut height,
            )
        };
        let raw = raw.cast();
        if success {
            if let Some(raw) = NonNull::new(raw) {
                Ok(
                    SwapchainTexture {
                        raw,
                        width,
                        height,
                        _marker: PhantomData,
                    }
                )
            } else {
                Err(None)
            }
        } else {
            Err(Some(get_error()))
        }
    }
}

impl Extern<SDL_GPUCommandBuffer> {
    #[doc(alias = "SDL_PushGPUVertexUniformData")]
    pub fn push_vertex_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.raw(),
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
                self.raw(),
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
                self.raw(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }
}
