mod resource;
use std::cell::UnsafeCell;

pub use resource::{
    Owned,
    Buffer,
    TransferBuffer,
    GraphicsPipeline,
    ComputePipeline,
    Texture,
    Sampler,
    Shader,
    OwnedDevice, Device,
};
pub use resource::{
    ComputePipelineBuilder, GraphicsPipelineBuilder, ShaderBuilder, BufferBuilder, 
    TransferBufferBuilder,
};

mod command_buffer;
pub use command_buffer::{
    OwnedCommandBuffer,
    CommandBuffer,
    ComputePass,
    RenderPass,
    CopyPass,
    Fence,
};

mod enums;
pub use enums::{
    BlendFactor, BlendOp, BufferUsageFlags, ColorComponentFlags, CompareOp, CullMode, FillMode,
    Filter, FrontFace, IndexElementSize, LoadOp, PrimitiveType, SampleCount, SamplerAddressMode,
    SamplerMipmapMode, ShaderFormat, ShaderStage, StencilOp, StoreOp, TextureFormat, TextureType,
    TextureUsage, TransferBufferUsage, VertexElementFormat, VertexInputRate,
};

mod info_struct;
pub use info_struct::{
    BufferBinding, BufferRegion, ColorTargetInfo, DepthStencilTargetInfo, SamplerCreateInfo,
    TextureCreateInfo, TextureRegion, TextureTransferInfo, TransferBufferLocation,
    VertexBufferDescription, RasterizerState,
    StencilOpState, VertexAttribute, VertexInputState,
    ColorTargetBlendState, ColorTargetDescription, GraphicsPipelineTargetInfo,
    DepthStencilState, TextureSamplerBinding,
    StorageBufferReadWriteBinding, StorageTextureReadWriteBinding,
};


use sys::gpu::{SDL_ClaimWindowForGPUDevice, SDL_GetGPUSwapchainTextureFormat, SDL_ReleaseWindowFromGPUDevice};

use crate::{get_error, Error};

mod util;



unsafe impl Sync for Device {}
unsafe impl Sync for Buffer {}

// We need some wrapper to be able to implement (inherent) methods for the type.
// The UnsafeCell doesn't actually do anything for &mut Extern, but the wrapped types
// are also zero-sized, so safe code still can't access any bytes with that.
#[repr(transparent)]
pub struct Extern<T>(UnsafeCell<T>);

impl<T> Extern<T> {
    pub fn ll(&self) -> *mut T {
        self.0.get()
    }
}

impl Device {
  
    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn acquire_command_buffer<'gpu>(&'gpu self) -> Result<OwnedCommandBuffer<'gpu>, Error> {
        OwnedCommandBuffer::new(self)
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn claim_window(&self, w: &crate::video::Window) -> Result<(), Error> {
        let p = unsafe { SDL_ClaimWindowForGPUDevice(self.ll(), w.raw()) };
        if p {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn release_window(&self, w: &crate::video::Window) {
        unsafe { SDL_ReleaseWindowFromGPUDevice(self.ll(), w.raw()) };
    }


    #[doc(alias = "SDL_GetGPUSwapchainTextureFormat")]
    pub fn get_swapchain_texture_format(&self, w: &crate::video::Window) -> TextureFormat {
        unsafe { SDL_GetGPUSwapchainTextureFormat(self.ll(), w.raw()) }
    }

    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn get_shader_formats(&self) -> ShaderFormat {
        unsafe { ShaderFormat(sys::gpu::SDL_GetGPUShaderFormats(self.ll())) }
    }

    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKSuspendGPU")]
    pub fn gdk_suspend(&self) {
        unsafe {
            sys::gpu::SDL_GDKSuspendGPU(self.raw());
        }
    }

    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKResumeGPU")]
    pub fn gdk_resume(&self) {
        unsafe {
            sys::gpu::SDL_GDKResumeGPU(self.raw());
        }
    }

}
