use crate::{
    get_error,
    gpu::{
        BufferBuilder, ColorTargetInfo, CommandBuffer, CopyPass, DepthStencilTargetInfo,
        GraphicsPipelineBuilder, PresentMode, RenderPass, Sampler, SamplerCreateInfo,
        ShaderBuilder, ShaderFormat, SwapchainComposition, Texture, TextureCreateInfo,
        TextureFormat, TransferBufferBuilder,
    },
    properties::Properties,
    sys,
    video::Window,
    Error,
};
use std::sync::{Arc, Weak};
use sys::gpu::{
    SDL_BeginGPUComputePass, SDL_BeginGPUCopyPass, SDL_BeginGPURenderPass, SDL_CreateGPUDevice,
    SDL_CreateGPUDeviceWithProperties, SDL_CreateGPUSampler, SDL_CreateGPUTexture,
    SDL_DestroyGPUDevice, SDL_GPUColorTargetInfo, SDL_GPUDepthStencilTargetInfo, SDL_GPUDevice,
    SDL_GPUViewport, SDL_GetGPUSwapchainTextureFormat, SDL_SetGPUViewport,
};

use super::{
    pass::Fence,
    pipeline::{StorageBufferReadWriteBinding, StorageTextureReadWriteBinding},
    ComputePass, ComputePipelineBuilder,
};

pub struct Viewport {
    inner: SDL_GPUViewport,
}
impl Viewport {
    #[doc(alias = "SDL_GPUViewport")]
    pub fn new(x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) -> Self {
        Self {
            inner: SDL_GPUViewport {
                x,
                y,
                w,
                h,
                min_depth,
                max_depth,
            },
        }
    }

    #[inline]
    pub fn raw(&self) -> *const SDL_GPUViewport {
        &self.inner
    }
}

/// Manages the raw `SDL_GPUDevice` pointer and releases it on drop
pub(super) struct DeviceContainer(*mut SDL_GPUDevice);
impl DeviceContainer {
    pub(super) fn raw(&self) -> *mut SDL_GPUDevice {
        self.0
    }
}
impl Drop for DeviceContainer {
    #[doc(alias = "SDL_DestroyGPUDevice")]
    fn drop(&mut self) {
        unsafe { SDL_DestroyGPUDevice(self.0) }
    }
}

pub(super) type WeakDevice = Weak<DeviceContainer>;

#[derive(Clone)]
pub struct Device {
    inner: Arc<DeviceContainer>,
    // Keep a `VideoSubsystem` in each `Device`,
    // to properly drop data in the right order
    subsystem: Option<crate::VideoSubsystem>,
}
impl Device {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUDevice {
        self.inner.0
    }

    pub(super) fn weak(&self) -> WeakDevice {
        Arc::downgrade(&self.inner)
    }

    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(flags: ShaderFormat, debug_mode: bool) -> Result<Self, Error> {
        let raw_device = unsafe { SDL_CreateGPUDevice(flags.0, debug_mode, std::ptr::null()) };
        if raw_device.is_null() {
            Err(get_error())
        } else {
            Ok(Self {
                inner: Arc::new(DeviceContainer(raw_device)),
                subsystem: None,
            })
        }
    }

    #[doc(alias = "SDL_CreateGPUDeviceWithProperties")]
    pub fn new_with_properties(properties: Properties) -> Result<Self, Error> {
        let raw_device = unsafe { SDL_CreateGPUDeviceWithProperties(properties.raw()) };
        if raw_device.is_null() {
            Err(get_error())
        } else {
            Ok(Self {
                inner: Arc::new(DeviceContainer(raw_device)),
                subsystem: None,
            })
        }
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn with_window(mut self, window: &crate::video::Window) -> Result<Self, Error> {
        self.subsystem = Some(window.subsystem().clone());

        let p = unsafe { sys::gpu::SDL_ClaimWindowForGPUDevice(self.inner.0, window.raw()) };
        if p {
            Ok(self)
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn acquire_command_buffer(&self) -> Result<CommandBuffer, Error> {
        let raw_buffer = unsafe { sys::gpu::SDL_AcquireGPUCommandBuffer(self.inner.0) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(CommandBuffer::new(raw_buffer))
        }
    }

    pub fn create_shader(&self) -> ShaderBuilder<'_> {
        ShaderBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUBuffer")]
    pub fn create_buffer(&self) -> BufferBuilder<'_> {
        BufferBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUTransferBuffer")]
    pub fn create_transfer_buffer(&self) -> TransferBufferBuilder<'_> {
        TransferBufferBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUSampler")]
    pub fn create_sampler(&self, create_info: SamplerCreateInfo) -> Result<Sampler, Error> {
        let raw_sampler = unsafe { SDL_CreateGPUSampler(self.raw(), &create_info.inner) };
        if raw_sampler.is_null() {
            Err(get_error())
        } else {
            Ok(Sampler::new(self, raw_sampler))
        }
    }

    #[doc(alias = "SDL_CreateGPUTexture")]
    pub fn create_texture(
        &self,
        create_info: TextureCreateInfo,
    ) -> Result<Texture<'static>, Error> {
        let raw_texture = unsafe { SDL_CreateGPUTexture(self.raw(), &create_info.inner) };
        if raw_texture.is_null() {
            Err(get_error())
        } else {
            Ok(Texture::new(
                self,
                raw_texture,
                create_info.inner.width,
                create_info.inner.height,
            ))
        }
    }

    #[doc(alias = "SDL_SetGPUViewport")]
    pub fn set_viewport(&self, render_pass: &RenderPass, viewport: Viewport) {
        unsafe { SDL_SetGPUViewport(render_pass.inner, viewport.raw()) }
    }

    pub fn get_swapchain_texture_format(&self, w: &crate::video::Window) -> TextureFormat {
        unsafe { std::mem::transmute(SDL_GetGPUSwapchainTextureFormat(self.inner.0, w.raw()).0) }
    }

    // You cannot begin another render pass, or begin a compute pass or copy pass until you have ended the render pass.
    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn begin_render_pass(
        &self,
        command_buffer: &CommandBuffer,
        color_info: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
    ) -> Result<RenderPass, Error> {
        let p = unsafe {
            SDL_BeginGPURenderPass(
                command_buffer.inner,
                color_info.as_ptr() as *const SDL_GPUColorTargetInfo, //heavy promise
                color_info.len() as u32,
                if let Some(p) = depth_stencil_target {
                    p as *const _ as *const SDL_GPUDepthStencilTargetInfo //heavy promise
                } else {
                    std::ptr::null()
                },
            )
        };
        if !p.is_null() {
            Ok(RenderPass { inner: p })
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_EndGPURenderPass")]
    pub fn end_render_pass(&self, pass: RenderPass) {
        unsafe {
            sys::gpu::SDL_EndGPURenderPass(pass.inner);
        }
    }

    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn begin_copy_pass(&self, command_buffer: &CommandBuffer) -> Result<CopyPass, Error> {
        let p = unsafe { SDL_BeginGPUCopyPass(command_buffer.inner) };
        if !p.is_null() {
            Ok(CopyPass { inner: p })
        } else {
            Err(get_error())
        }
    }
    #[doc(alias = "SDL_EndGPUCopyPass")]
    pub fn end_copy_pass(&self, pass: CopyPass) {
        unsafe {
            sys::gpu::SDL_EndGPUCopyPass(pass.inner);
        }
    }

    #[doc(alias = "SDL_BeginGPUComputePass")]
    pub fn begin_compute_pass(
        &self,
        command_buffer: &CommandBuffer,
        storage_texture_bindings: &[StorageTextureReadWriteBinding],
        storage_buffer_bindings: &[StorageBufferReadWriteBinding],
    ) -> Result<ComputePass, Error> {
        let p = unsafe {
            SDL_BeginGPUComputePass(
                command_buffer.inner,
                storage_texture_bindings.as_ptr().cast(),
                storage_texture_bindings.len() as u32,
                storage_buffer_bindings.as_ptr().cast(),
                storage_buffer_bindings.len() as u32,
            )
        };
        if !p.is_null() {
            Ok(ComputePass { inner: p })
        } else {
            Err(get_error())
        }
    }
    #[doc(alias = "SDL_EndGPUComputePass")]
    pub fn end_compute_pass(&self, pass: ComputePass) {
        unsafe {
            sys::gpu::SDL_EndGPUComputePass(pass.inner);
        }
    }

    pub fn create_graphics_pipeline<'a>(&'a self) -> GraphicsPipelineBuilder<'a> {
        GraphicsPipelineBuilder::new(self)
    }

    pub fn create_compute_pipeline<'a>(&'a self) -> ComputePipelineBuilder<'a> {
        ComputePipelineBuilder::new(self)
    }

    #[doc(alias = "SDL_WaitForGPUFences")]
    pub fn wait_fences(&self, wait_all: bool, fences: &[Fence]) -> Result<(), Error> {
        let fences: Vec<_> = fences.iter().map(|x| x.raw()).collect();
        unsafe {
            if !sys::gpu::SDL_WaitForGPUFences(
                self.raw(),
                wait_all,
                fences.as_ptr(),
                fences.len() as u32,
            ) {
                Err(get_error())
            } else {
                Ok(())
            }
        }
    }

    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn get_shader_formats(&self) -> ShaderFormat {
        unsafe { std::mem::transmute(sys::gpu::SDL_GetGPUShaderFormats(self.raw())) }
    }

    #[doc(alias = "SDL_SetGPUSwapchainParameters")]
    pub fn set_swapchain_parameters(
        &self,
        window: &Window,
        present_mode: PresentMode,
        swapchain_composition: SwapchainComposition,
    ) -> Result<(), Error> {
        let raw_device_ptr = self.raw();
        let raw_window_ptr = window.raw();

        let c_present_mode = sys::gpu::SDL_GPUPresentMode(present_mode as i32);
        let c_swapchain_composition =
            sys::gpu::SDL_GPUSwapchainComposition(swapchain_composition as i32);

        let success = unsafe {
            sys::gpu::SDL_SetGPUSwapchainParameters(
                raw_device_ptr,
                raw_window_ptr,
                c_swapchain_composition,
                c_present_mode,
            )
        };

        if success {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    // NOTE: for Xbox builds, the target is a UWP, e.g.: x86_64-uwp-windows-msvc
    #[cfg(target_vendor = "uwp")]
    #[doc(alias = "SDL_GDKSuspendGPU")]
    pub fn gdk_suspend(&self) {
        unsafe {
            sys::gpu::SDL_GDKSuspendGPU(self.inner);
        }
    }

    #[cfg(target_vendor = "uwp")]
    #[doc(alias = "SDL_GDKResumeGPU")]
    pub fn gdk_resume(&self) {
        unsafe {
            sys::gpu::SDL_GDKResumeGPU(self.inner);
        }
    }
}
