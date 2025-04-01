mod resource;
mod swapchain;
use std::cell::UnsafeCell;

pub use resource::{
    Buffer,
    TransferBuffer,
     /* FIXME: BufferMemMap, */
    GraphicsPipeline,
    ComputePipeline,
    Texture,
    Sampler,
    Shader,
    Device,
};
pub use resource::{
    ComputePipelineBuilder, GraphicsPipelineBuilder, ShaderBuilder, BufferBuilder, 
    TransferBufferBuilder,
};

mod command_buffer;
pub use command_buffer::CommandBuffer;

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

mod pass_render;
pub use pass_render::RenderPass;

mod pass_copy;
pub use pass_copy::CopyPass;

mod pass_compute;
pub use pass_compute::ComputePass;
use sys::gpu::{SDL_ClaimWindowForGPUDevice, SDL_GetGPUSwapchainTextureFormat, SDL_ReleaseWindowFromGPUDevice};

use crate::{get_error, Error};

mod util;

pub type ExternDevice = Extern<sys::gpu::SDL_GPUDevice>;

// We need some wrapper to be able to implement (inherent) methods for the type.
// The UnsafeCell doesn't actually do anything for &mut Extern, but the wrapped types
// are also zero-sized, so safe code still can't access any bytes with that.
#[repr(transparent)]
pub struct Extern<T>(UnsafeCell<T>);

impl<T> Extern<T> {
    pub fn raw(&self) -> *mut T {
        self.0.get()
    }
}

impl ExternDevice {
    #[doc(alias = "SDL_CreateGPUShader")]
    pub fn create_shader(&self) -> ShaderBuilder {
        ShaderBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUBuffer")]
    pub fn create_buffer(&self) -> BufferBuilder {
        BufferBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUTransferBuffer")]
    pub fn create_transfer_buffer(&self) -> TransferBufferBuilder {
        TransferBufferBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUSampler")]
    pub fn create_sampler(&self, create_info: SamplerCreateInfo) -> Result<Sampler, Error> {
        Sampler::new(self, &create_info.inner)
    }

    #[doc(alias = "SDL_CreateGPUGraphicsPipeline")]
    pub fn create_graphics_pipeline<'gpu,'a>(&'gpu self) -> GraphicsPipelineBuilder<'gpu, 'a> {
        GraphicsPipelineBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUComputePipeline")]
    pub fn create_compute_pipeline<'gpu,'a>(&'gpu self) -> ComputePipelineBuilder<'gpu, 'a> {
        ComputePipelineBuilder::new(self)
    }

    #[doc(alias = "SDL_CreateGPUTexture")]
    pub fn create_texture(
        &self,
        create_info: &TextureCreateInfo,
    ) -> Result<Texture, Error> {
        Texture::new(self, &create_info.inner)
    }

    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn acquire_command_buffer<'gpu>(&'gpu self) -> Result<CommandBuffer<'gpu>, Error> {
        CommandBuffer::new(self)
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn claim_window(&self, w: &crate::video::Window) -> Result<(), Error> {
        let p = unsafe { SDL_ClaimWindowForGPUDevice(self.raw(), w.raw()) };
        if p {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn release_window(&self, w: &crate::video::Window) {
        unsafe { SDL_ReleaseWindowFromGPUDevice(self.raw(), w.raw()) };
    }


    #[doc(alias = "SDL_GetGPUSwapchainTextureFormat")]
    pub fn get_swapchain_texture_format(&self, w: &crate::video::Window) -> TextureFormat {
        unsafe { SDL_GetGPUSwapchainTextureFormat(self.raw(), w.raw()) }
    }

    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn get_shader_formats(&self) -> ShaderFormat {
        unsafe { ShaderFormat(sys::gpu::SDL_GetGPUShaderFormats(self.raw())) }
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
