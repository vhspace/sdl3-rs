//! Defines builders for various GPU-resources.
//! 
//! 

use std::{ffi::CStr, marker::PhantomData};

use sys::gpu::{
    SDL_GPUBufferCreateInfo, SDL_GPUComputePipelineCreateInfo,
    SDL_GPUFillMode, SDL_GPUGraphicsPipelineCreateInfo,
    SDL_GPUPrimitiveType, SDL_GPUShaderCreateInfo, SDL_GPUTransferBufferCreateInfo
};

use crate::gpu::{SamplerCreateInfo, TextureCreateInfo};
use crate::Error;

use super::super::{
    BufferUsageFlags, ComputePipeline, DepthStencilState, FillMode, GraphicsPipeline, GraphicsPipelineTargetInfo, PrimitiveType, RasterizerState, Shader, ShaderFormat, ShaderStage, TransferBuffer, TransferBufferUsage, VertexInputState
};

impl Device {
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
    pub fn create_sampler<'gpu>(&'gpu self, create_info: SamplerCreateInfo) -> Result<Owned<'gpu, Sampler>, Error> {
        Owned::new(self, &create_info.inner, ())
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
    pub fn create_texture<'gpu>(
        &'gpu self,
        create_info: &TextureCreateInfo,
    ) -> Result<Owned<'gpu, Texture>, Error> {
        Owned::new(self, &create_info.inner, (create_info.inner.width, create_info.inner.height))
    }

}


use super::{Buffer, Device, Owned, Sampler, Texture};
#[repr(C)]
pub struct ComputePipelineBuilder<'gpu, 'builder> {
    device: &'gpu Device,
    inner: SDL_GPUComputePipelineCreateInfo,
    _marker: PhantomData<&'builder Shader>,
}
impl<'gpu, 'builder> ComputePipelineBuilder<'gpu, 'builder> {
    pub(in crate::gpu) fn new(device: &'gpu Device) -> Self {
        Self {
            device,
            inner: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_code(mut self, fmt: ShaderFormat, code: &'builder [u8]) -> Self {
        self.inner.format = fmt.0;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len();
        self
    }

    pub fn with_entrypoint(mut self, entry_point: &'builder CStr) -> Self {
        self.inner.entrypoint = entry_point.as_ptr();
        self
    }

    pub fn with_readonly_storage_textures(mut self, value: u32) -> Self {
        self.inner.num_readonly_storage_textures = value;
        self
    }

    pub fn with_readonly_storage_buffers(mut self, value: u32) -> Self {
        self.inner.num_readonly_storage_buffers = value;
        self
    }

    pub fn with_readwrite_storage_textures(mut self, value: u32) -> Self {
        self.inner.num_readwrite_storage_textures = value;
        self
    }

    pub fn with_readwrite_storage_buffers(mut self, value: u32) -> Self {
        self.inner.num_readwrite_storage_buffers = value;
        self
    }

    pub fn with_uniform_buffers(mut self, value: u32) -> Self {
        self.inner.num_uniform_buffers = value;
        self
    }

    pub fn with_thread_count(mut self, x: u32, y: u32, z: u32) -> Self {
        self.inner.threadcount_x = x;
        self.inner.threadcount_y = y;
        self.inner.threadcount_z = z;
        self
    }

    pub fn build(self) -> Result<Owned<'gpu, ComputePipeline>, Error> {
        Owned::new(self.device, &self.inner, ())
    }
}

pub struct ShaderBuilder<'builder, 'gpu> {
    device: &'gpu Device,
    inner: SDL_GPUShaderCreateInfo,
    _marker: PhantomData<&'builder [u8]>,
}

impl<'gpu, 'builder> ShaderBuilder<'builder, 'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu Device) -> Self {
        Self {
            device,
            inner: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_samplers(mut self, value: u32) -> Self {
        self.inner.num_samplers = value;
        self
    }

    pub fn with_storage_buffers(mut self, value: u32) -> Self {
        self.inner.num_storage_buffers = value;
        self
    }

    pub fn with_storage_textures(mut self, value: u32) -> Self {
        self.inner.num_storage_textures = value;
        self
    }

    pub fn with_uniform_buffers(mut self, value: u32) -> Self {
        self.inner.num_uniform_buffers = value;
        self
    }

    pub fn with_code(mut self, fmt: ShaderFormat, code: &'builder [u8], stage: ShaderStage) -> Self {
        self.inner.format = fmt.0;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len() as usize;
        self.inner.stage = unsafe { std::mem::transmute(stage as u32) };
        self
    }
    pub fn with_entrypoint(mut self, entry_point: &'gpu CStr) -> Self {
        self.inner.entrypoint = entry_point.as_ptr();
        self
    }
    pub fn build(self) -> Result<Owned<'gpu, Shader>, Error> {
        Owned::new(self.device, &self.inner, ())
    }
}


pub struct TransferBufferBuilder<'gpu> {
    device: &'gpu Device,
    inner: SDL_GPUTransferBufferCreateInfo,
}
impl<'gpu> TransferBufferBuilder<'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    /// How the buffer will be used.
    pub fn with_usage(mut self, value: TransferBufferUsage) -> Self {
        self.inner.usage = value;
        self
    }

    /// Desired size of the buffer in bytes.
    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<Owned<'gpu, TransferBuffer>, Error> {
        Owned::new(self.device, &self.inner, self.inner.size)
    }
}


pub struct BufferBuilder<'gpu> {
    device: &'gpu Device,
    inner: SDL_GPUBufferCreateInfo,
}
impl<'gpu> BufferBuilder<'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    pub fn with_usage(mut self, value: BufferUsageFlags) -> Self {
        self.inner.usage = value.0;
        self
    }

    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<Owned<'gpu, Buffer>, Error> {
        Owned::new(self.device, &self.inner, self.inner.size)
    }
}

#[derive(Copy,Clone)]
#[repr(C)]
pub struct GraphicsPipelineBuilder<'gpu, 'builder> {
    device: &'gpu Device,
    inner: SDL_GPUGraphicsPipelineCreateInfo,
    _marker: PhantomData<(
        &'builder Shader,
        GraphicsPipelineTargetInfo<'builder>,
    )>,
}

impl<'gpu, 'builder> GraphicsPipelineBuilder<'gpu, 'builder> {
    pub(in crate::gpu) fn new(device: &'gpu Device) -> Self {
        Self {
            device,
            inner: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_fragment_shader(mut self, value: &'builder Shader) -> Self {
        self.inner.fragment_shader = value.ll();
        self
    }
    pub fn with_vertex_shader(mut self, value: &'builder Shader) -> Self {
        self.inner.vertex_shader = value.ll();
        self
    }
    pub fn with_primitive_type(mut self, value: PrimitiveType) -> Self {
        self.inner.primitive_type = SDL_GPUPrimitiveType(value as i32);
        self
    }

    /// Whether polygons will be filled in or drawn as lines.
    ///
    /// Note: this will override the value set in `with_rasterizer_state` if called after.
    pub fn with_fill_mode(mut self, value: FillMode) -> Self {
        self.inner.rasterizer_state.fill_mode = SDL_GPUFillMode(value as i32);
        self
    }

    /// Sets the parameters of the graphics pipeline rasterizer state.
    ///
    /// Note: this will override the value set in `with_fill_mode` if called after.
    pub fn with_rasterizer_state(mut self, value: RasterizerState) -> Self {
        self.inner.rasterizer_state = value.inner;
        self
    }

    pub fn with_depth_stencil_state(mut self, value: DepthStencilState) -> Self {
        self.inner.depth_stencil_state = value.inner;
        self
    }

    pub fn with_vertex_input_state(mut self, value: VertexInputState<'builder>) -> Self {
        self.inner.vertex_input_state = value.inner;
        self
    }

    pub fn with_target_info(mut self, value: GraphicsPipelineTargetInfo<'builder>) -> Self {
        self.inner.target_info = value.inner;
        self
    }

    pub fn build(&self) -> Result<Owned<'gpu, GraphicsPipeline>, Error> {
        Owned::new(self.device, &self.inner, ())
    }
}