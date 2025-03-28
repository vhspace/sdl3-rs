use crate::{
    get_error,
    gpu::{
        device::WeakDevice, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullMode, Device,
        FillMode, FrontFace, PrimitiveType, Shader, StencilOp, TextureFormat,
        VertexBufferDescription, VertexElementFormat,
    },
    sys, Error,
};
use std::{ffi::CStr, sync::Arc};
use sys::gpu::{
    SDL_GPUBlendFactor, SDL_GPUBlendOp, SDL_GPUColorTargetBlendState,
    SDL_GPUColorTargetDescription, SDL_GPUCompareOp, SDL_GPUComputePipeline,
    SDL_GPUComputePipelineCreateInfo, SDL_GPUCullMode, SDL_GPUDepthStencilState, SDL_GPUFillMode,
    SDL_GPUFrontFace, SDL_GPUGraphicsPipeline, SDL_GPUGraphicsPipelineCreateInfo,
    SDL_GPUGraphicsPipelineTargetInfo, SDL_GPUPrimitiveType, SDL_GPURasterizerState,
    SDL_GPUStencilOp, SDL_GPUStencilOpState, SDL_GPUStorageBufferReadWriteBinding,
    SDL_GPUStorageTextureReadWriteBinding, SDL_GPUTextureFormat, SDL_GPUVertexAttribute,
    SDL_GPUVertexBufferDescription, SDL_GPUVertexInputState, SDL_ReleaseGPUComputePipeline,
    SDL_ReleaseGPUGraphicsPipeline,
};

use super::{Buffer, ShaderFormat, Texture};

#[derive(Default)]
pub struct GraphicsPipelineTargetInfo {
    inner: SDL_GPUGraphicsPipelineTargetInfo,
}
impl GraphicsPipelineTargetInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// A pointer to an array of color target descriptions.
    pub fn with_color_target_descriptions(mut self, value: &[ColorTargetDescription]) -> Self {
        self.inner.color_target_descriptions =
            value.as_ptr() as *const SDL_GPUColorTargetDescription;
        self.inner.num_color_targets = value.len() as u32;
        self
    }

    /// The pixel format of the depth-stencil target. Ignored if has_depth_stencil_target is false.
    pub fn with_depth_stencil_format(mut self, value: TextureFormat) -> Self {
        self.inner.depth_stencil_format = SDL_GPUTextureFormat(value as i32);
        self
    }

    /// true specifies that the pipeline uses a depth-stencil target.
    pub fn with_has_depth_stencil_target(mut self, value: bool) -> Self {
        self.inner.has_depth_stencil_target = value;
        self
    }
}

#[repr(C)]
pub struct GraphicsPipelineBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUGraphicsPipelineCreateInfo,
}
impl<'a> GraphicsPipelineBuilder<'a> {
    pub(super) fn new(device: &'a Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    pub fn with_fragment_shader(mut self, value: &'a Shader) -> Self {
        self.inner.fragment_shader = value.raw();
        self
    }
    pub fn with_vertex_shader(mut self, value: &'a Shader) -> Self {
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

    pub fn build(self) -> Result<GraphicsPipeline, Error> {
        let raw_pipeline =
            unsafe { sys::gpu::SDL_CreateGPUGraphicsPipeline(self.device.raw(), &self.inner) };
        if raw_pipeline.is_null() {
            Err(get_error())
        } else {
            Ok(GraphicsPipeline {
                inner: Arc::new(GraphicsPipelineContainer {
                    raw: raw_pipeline,
                    device: self.device.weak(),
                }),
            })
        }
    }
}

/// Manages the raw `SDL_GPUGraphicsPipeline` pointer and releases it on drop
struct GraphicsPipelineContainer {
    raw: *mut SDL_GPUGraphicsPipeline,
    device: WeakDevice,
}
impl Drop for GraphicsPipelineContainer {
    #[doc(alias = "SDL_ReleaseGPUGraphicsPipeline")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { SDL_ReleaseGPUGraphicsPipeline(device.raw(), self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct GraphicsPipeline {
    inner: Arc<GraphicsPipelineContainer>,
}
impl GraphicsPipeline {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUGraphicsPipeline {
        self.inner.raw
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct VertexAttribute {
    inner: SDL_GPUVertexAttribute,
}
impl VertexAttribute {
    pub fn new() -> Self {
        Default::default()
    }

    /// The shader input location index.
    pub fn with_location(mut self, value: u32) -> Self {
        self.inner.location = value;
        self
    }

    /// The binding slot of the associated vertex buffer.
    pub fn with_buffer_slot(mut self, value: u32) -> Self {
        self.inner.buffer_slot = value;
        self
    }

    /// The size and type of the attribute data.
    pub fn with_format(mut self, value: VertexElementFormat) -> Self {
        self.inner.format = unsafe { std::mem::transmute(value as u32) };
        self
    }

    /// The byte offset of this attribute relative to the start of the vertex element.
    pub fn with_offset(mut self, value: u32) -> Self {
        self.inner.offset = value;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct VertexInputState {
    inner: SDL_GPUVertexInputState,
}
impl VertexInputState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_vertex_buffer_descriptions(mut self, value: &[VertexBufferDescription]) -> Self {
        self.inner.vertex_buffer_descriptions =
            value.as_ptr() as *const SDL_GPUVertexBufferDescription;
        self.inner.num_vertex_buffers = value.len() as u32;
        self
    }

    pub fn with_vertex_attributes(mut self, value: &[VertexAttribute]) -> Self {
        self.inner.vertex_attributes = value.as_ptr() as *const SDL_GPUVertexAttribute;
        self.inner.num_vertex_attributes = value.len() as u32;
        self
    }
}

#[derive(Default)]
pub struct ColorTargetBlendState {
    inner: SDL_GPUColorTargetBlendState,
}
impl ColorTargetBlendState {
    pub fn new() -> Self {
        Self::default()
    }

    /// The value to be multiplied by the source RGB value.
    pub fn with_src_color_blendfactor(mut self, blend_factor: BlendFactor) -> Self {
        self.inner.src_color_blendfactor = SDL_GPUBlendFactor(blend_factor as i32);
        self
    }

    /// The value to be multiplied by the destination RGB value.
    pub fn with_dst_color_blendfactor(mut self, blend_factor: BlendFactor) -> Self {
        self.inner.dst_color_blendfactor = SDL_GPUBlendFactor(blend_factor as i32);
        self
    }

    /// The blend operation for the RGB components.
    pub fn with_color_blend_op(mut self, blend_op: BlendOp) -> Self {
        self.inner.color_blend_op = SDL_GPUBlendOp(blend_op as i32);
        self
    }

    /// The value to be multiplied by the source alpha.
    pub fn with_src_alpha_blendfactor(mut self, blend_factor: BlendFactor) -> Self {
        self.inner.src_alpha_blendfactor = SDL_GPUBlendFactor(blend_factor as i32);
        self
    }

    /// The value to be multiplied by the destination alpha.
    pub fn with_dst_alpha_blendfactor(mut self, blend_factor: BlendFactor) -> Self {
        self.inner.dst_alpha_blendfactor = SDL_GPUBlendFactor(blend_factor as i32);
        self
    }

    /// The blend operation for the alpha component.
    pub fn with_alpha_blend_op(mut self, blend_op: BlendOp) -> Self {
        self.inner.alpha_blend_op = SDL_GPUBlendOp(blend_op as i32);
        self
    }

    /// A bitmask specifying which of the RGBA components are enabled for writing. Writes to all channels if enable_color_write_mask is false.
    pub fn with_color_write_mask(mut self, flags: ColorComponentFlags) -> Self {
        self.inner.color_write_mask = flags.0;
        self
    }

    /// Whether blending is enabled for the color target.
    pub fn with_enable_blend(mut self, enable: bool) -> Self {
        self.inner.enable_blend = enable;
        self
    }

    /// Whether the color write mask is enabled.
    pub fn with_enable_color_write_mask(mut self, enable: bool) -> Self {
        self.inner.enable_color_write_mask = enable;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ColorTargetDescription {
    inner: SDL_GPUColorTargetDescription,
}
impl ColorTargetDescription {
    pub fn new() -> Self {
        Self::default()
    }

    /// The pixel format of the texture to be used as a color target.
    pub fn with_format(mut self, value: TextureFormat) -> Self {
        self.inner.format = SDL_GPUTextureFormat(value as i32);
        self
    }

    /// The blend state to be used for the color target.
    pub fn with_blend_state(mut self, value: ColorTargetBlendState) -> Self {
        self.inner.blend_state = value.inner;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct RasterizerState {
    inner: SDL_GPURasterizerState,
}
impl RasterizerState {
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether polygons will be filled in or drawn as lines.
    pub fn with_fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.inner.fill_mode = SDL_GPUFillMode(fill_mode as i32);
        self
    }

    /// The facing direction in which triangles will be culled.
    pub fn with_cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.inner.cull_mode = SDL_GPUCullMode(cull_mode as i32);
        self
    }

    /// The vertex winding that will cause a triangle to be determined as front-facing.
    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.inner.front_face = SDL_GPUFrontFace(front_face as i32);
        self
    }

    /// A scalar factor controlling the depth value added to each fragment.
    pub fn with_depth_bias_constant_factor(mut self, value: f32) -> Self {
        self.inner.depth_bias_constant_factor = value;
        self
    }

    /// The maximum depth bias of a fragment.
    pub fn with_depth_bias_clamp(mut self, value: f32) -> Self {
        self.inner.depth_bias_clamp = value;
        self
    }

    /// A scalar factor applied to a fragment's slope in depth calculations.
    pub fn with_depth_slope_factor(mut self, value: f32) -> Self {
        self.inner.depth_bias_slope_factor = value;
        self
    }

    /// True to bias fragment depth values.
    pub fn with_enable_depth_bias(mut self, value: bool) -> Self {
        self.inner.enable_depth_bias = value;
        self
    }

    /// True to enable depth clip, false to enable depth clamp.
    pub fn with_enable_depth_clip(mut self, value: bool) -> Self {
        self.inner.enable_depth_clip = value;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StencilOpState {
    inner: SDL_GPUStencilOpState,
}
impl StencilOpState {
    pub fn new() -> Self {
        Default::default()
    }

    /// The action performed on samples that fail the stencil test.
    pub fn with_fail_op(mut self, value: StencilOp) -> Self {
        self.inner.fail_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The action performed on samples that pass the depth and stencil tests.
    pub fn with_pass_op(mut self, value: StencilOp) -> Self {
        self.inner.pass_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The action performed on samples that pass the stencil test and fail the depth test.
    pub fn with_depth_fail_op(mut self, value: StencilOp) -> Self {
        self.inner.depth_fail_op = SDL_GPUStencilOp(value as i32);
        self
    }

    /// The comparison operator used in the stencil test.
    pub fn compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilState {
    inner: SDL_GPUDepthStencilState,
}
impl DepthStencilState {
    pub fn new() -> Self {
        Default::default()
    }

    /// The comparison operator used for depth testing.
    pub fn with_compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }

    /// The stencil op state for back-facing triangles.
    pub fn with_back_stencil_state(mut self, value: StencilOpState) -> Self {
        self.inner.back_stencil_state = value.inner;
        self
    }

    /// The stencil op state for front-facing triangles.
    pub fn with_front_stencil_state(mut self, value: StencilOpState) -> Self {
        self.inner.front_stencil_state = value.inner;
        self
    }

    /// Selects the bits of the stencil values participating in the stencil test.
    pub fn with_compare_mask(mut self, value: u8) -> Self {
        self.inner.compare_mask = value;
        self
    }

    /// Selects the bits of the stencil values updated by the stencil test.
    pub fn with_write_mask(mut self, value: u8) -> Self {
        self.inner.write_mask = value;
        self
    }

    /// True enables the depth test.
    pub fn with_enable_depth_test(mut self, value: bool) -> Self {
        self.inner.enable_depth_test = value;
        self
    }

    /// True enables depth writes.
    pub fn with_enable_depth_write(mut self, value: bool) -> Self {
        self.inner.enable_depth_write = value;
        self
    }

    /// True enables the stencil test.
    pub fn with_enable_stencil_test(mut self, value: bool) -> Self {
        self.inner.enable_stencil_test = value;
        self
    }
}

#[repr(C)]
pub struct ComputePipelineBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUComputePipelineCreateInfo,
}
impl<'a> ComputePipelineBuilder<'a> {
    pub(super) fn new(device: &'a Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    pub fn with_code(mut self, fmt: ShaderFormat, code: &'a [u8]) -> Self {
        self.inner.format = fmt.0;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len();
        self
    }

    pub fn with_entrypoint(mut self, entry_point: &'a CStr) -> Self {
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

    pub fn build(self) -> Result<ComputePipeline, Error> {
        let raw_pipeline =
            unsafe { sys::gpu::SDL_CreateGPUComputePipeline(self.device.raw(), &self.inner) };
        if raw_pipeline.is_null() {
            Err(get_error())
        } else {
            Ok(ComputePipeline {
                inner: Arc::new(ComputePipelineContainer {
                    raw: raw_pipeline,
                    device: self.device.weak(),
                }),
            })
        }
    }
}

/// Manages the raw `SDL_GPUComputePipeline` pointer and releases it on drop
struct ComputePipelineContainer {
    raw: *mut SDL_GPUComputePipeline,
    device: WeakDevice,
}
impl Drop for ComputePipelineContainer {
    #[doc(alias = "SDL_ReleaseGPUComputePipeline")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { SDL_ReleaseGPUComputePipeline(device.raw(), self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct ComputePipeline {
    inner: Arc<ComputePipelineContainer>,
}
impl ComputePipeline {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUComputePipeline {
        self.inner.raw
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StorageTextureReadWriteBinding {
    inner: SDL_GPUStorageTextureReadWriteBinding,
}
impl StorageTextureReadWriteBinding {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    pub fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.inner.mip_level = mip_level;
        self
    }

    pub fn with_layer(mut self, layer: u32) -> Self {
        self.inner.layer = layer;
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StorageBufferReadWriteBinding {
    inner: SDL_GPUStorageBufferReadWriteBinding,
}
impl StorageBufferReadWriteBinding {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &Buffer) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }
}
