//! Defines builders for various GPU-resources.
//! 
//! 

use std::{ffi::CStr, marker::PhantomData};

use sys::gpu::{
    SDL_GPUBufferCreateInfo, SDL_GPUComputePipelineCreateInfo, SDL_GPUFillMode, SDL_GPUGraphicsPipelineCreateInfo, SDL_GPUPrimitiveType, SDL_GPUShaderCreateInfo, SDL_GPUTransferBufferCreateInfo
};

use crate::Error;

use super::super::{
    Buffer, BufferUsageFlags, ComputePipeline, DepthStencilState, FillMode, GraphicsPipeline, GraphicsPipelineTargetInfo, PrimitiveType, RasterizerState, Shader, ShaderFormat, ShaderStage, TransferBuffer, TransferBufferUsage, VertexInputState
};
use super::util::ExternDevice;
#[repr(C)]
pub struct ComputePipelineBuilder<'gpu, 'builder> {
    device: &'gpu ExternDevice,
    inner: SDL_GPUComputePipelineCreateInfo,
    _marker: PhantomData<&'builder Shader<'gpu>>,
}
impl<'gpu, 'builder> ComputePipelineBuilder<'gpu, 'builder> {
    pub(in crate::gpu) fn new(device: &'gpu ExternDevice) -> Self {
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

    pub fn build(self) -> Result<ComputePipeline<'gpu>, Error> {
        ComputePipeline::new(self.device, &self.inner)
    }
}

pub struct ShaderBuilder<'gpu> {
    device: &'gpu ExternDevice,
    inner: SDL_GPUShaderCreateInfo,
}

impl<'gpu> ShaderBuilder<'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu ExternDevice) -> Self {
        Self {
            device,
            inner: Default::default(),
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

    pub fn with_code(mut self, fmt: ShaderFormat, code: &'gpu [u8], stage: ShaderStage) -> Self {
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
    pub fn build(self) -> Result<Shader<'gpu>, Error> {
        Shader::new(self.device, &self.inner)
    }
}


pub struct TransferBufferBuilder<'gpu> {
    device: &'gpu ExternDevice,
    inner: SDL_GPUTransferBufferCreateInfo,
}
impl<'gpu> TransferBufferBuilder<'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu ExternDevice) -> Self {
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

    pub fn build(self) -> Result<TransferBuffer<'gpu>, Error> {
        TransferBuffer::new(self.device, &self.inner)
    }
}


pub struct BufferBuilder<'gpu> {
    device: &'gpu ExternDevice,
    inner: SDL_GPUBufferCreateInfo,
}
impl<'gpu> BufferBuilder<'gpu> {
    pub(in crate::gpu) fn new(device: &'gpu ExternDevice) -> Self {
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

    pub fn build(self) -> Result<Buffer<'gpu>, Error> {
        Buffer::new(self.device, &self.inner)
    }
}

#[repr(C)]
pub struct GraphicsPipelineBuilder<'gpu, 'builder> {
    device: &'gpu ExternDevice,
    inner: SDL_GPUGraphicsPipelineCreateInfo,
    _marker: PhantomData<&'builder Shader<'gpu>>,
}

impl<'gpu, 'builder> GraphicsPipelineBuilder<'gpu, 'builder> {
    pub(in crate::gpu) fn new(device: &'gpu ExternDevice) -> Self {
        Self {
            device,
            inner: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_fragment_shader(mut self, value: &'builder Shader<'gpu>) -> Self {
        self.inner.fragment_shader = value.raw();
        self
    }
    pub fn with_vertex_shader(mut self, value: &'builder Shader<'gpu>) -> Self {
        self.inner.vertex_shader = value.raw();
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

    pub fn with_vertex_input_state(mut self, value: VertexInputState) -> Self {
        self.inner.vertex_input_state = value.inner;
        self
    }

    pub fn with_target_info(mut self, value: GraphicsPipelineTargetInfo) -> Self {
        self.inner.target_info = value.inner;
        self
    }

    pub fn build(self) -> Result<GraphicsPipeline<'gpu>, Error> {
        GraphicsPipeline::new(self.device, &self.inner)
    }
}