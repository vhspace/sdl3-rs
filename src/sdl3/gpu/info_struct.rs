//! Types which hold data but don't own any GPU-resources.
//! There are no calls to SDL here.
//! 

use std::marker::PhantomData;

use sys::gpu::{
    SDL_GPUBlendFactor, SDL_GPUBlendOp, SDL_GPUBuffer, SDL_GPUBufferBinding,
    SDL_GPUBufferRegion, SDL_GPUColorTargetBlendState, SDL_GPUColorTargetDescription,
    SDL_GPUColorTargetInfo, SDL_GPUCompareOp, SDL_GPUCullMode, SDL_GPUDepthStencilState,
    SDL_GPUDepthStencilTargetInfo, SDL_GPUFillMode, SDL_GPUFilter, SDL_GPUFrontFace,
    SDL_GPUGraphicsPipelineTargetInfo, SDL_GPURasterizerState, SDL_GPUSampleCount,
    SDL_GPUSampler, SDL_GPUSamplerAddressMode, SDL_GPUSamplerCreateInfo,
    SDL_GPUSamplerMipmapMode, SDL_GPUStencilOp, SDL_GPUStencilOpState,
    SDL_GPUStorageBufferReadWriteBinding, SDL_GPUStorageTextureReadWriteBinding,
    SDL_GPUTexture, SDL_GPUTextureCreateInfo, SDL_GPUTextureRegion,
    SDL_GPUTextureSamplerBinding, SDL_GPUTextureTransferInfo, SDL_GPUTextureType,
    SDL_GPUTransferBuffer, SDL_GPUTransferBufferLocation, SDL_GPUVertexAttribute,
    SDL_GPUVertexBufferDescription, SDL_GPUVertexInputRate, SDL_GPUVertexInputState,
};

use crate::pixels::Color;

use super::{
    BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullMode, Extern,
    FillMode, Filter, FrontFace, LoadOp, SampleCount, Sampler, SamplerAddressMode,
    SamplerMipmapMode, StencilOp, StoreOp, Texture, TextureFormat, TextureType,
    TextureUsage, VertexElementFormat, VertexInputRate
};

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilTargetInfo<'a> {
    inner: SDL_GPUDepthStencilTargetInfo,
    _marker: PhantomData<&'a Extern<SDL_GPUTexture>>,
}
impl<'a> DepthStencilTargetInfo<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &'a Extern<SDL_GPUTexture>) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    pub fn with_clear_depth(mut self, clear_depth: f32) -> Self {
        self.inner.clear_depth = clear_depth;
        self
    }

    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op = value;
        self
    }

    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op = value;
        self
    }

    pub fn with_stencil_load_op(mut self, value: LoadOp) -> Self {
        self.inner.stencil_load_op = value;
        self
    }

    pub fn with_stencil_store_op(mut self, value: StoreOp) -> Self {
        self.inner.stencil_store_op = value;
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }

    pub fn with_clear_stencil(mut self, clear_stencil: u8) -> Self {
        self.inner.clear_stencil = clear_stencil;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ColorTargetInfo<'a> {
    inner: SDL_GPUColorTargetInfo,
    _marker: PhantomData<&'a Extern<SDL_GPUTexture>>,
}
impl<'a> ColorTargetInfo<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &'a Extern<SDL_GPUTexture>) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op = value;
        self
    }

    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op = value;
        self
    }

    pub fn with_clear_color(mut self, value: Color) -> Self {
        self.inner.clear_color.r = (value.r as f32) / 255.0;
        self.inner.clear_color.g = (value.g as f32) / 255.0;
        self.inner.clear_color.b = (value.b as f32) / 255.0;
        self.inner.clear_color.a = (value.a as f32) / 255.0;
        self
    }
}


#[derive(Default)]
pub struct TextureCreateInfo {
    pub(super) inner: SDL_GPUTextureCreateInfo,
}
impl TextureCreateInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The base dimensionality of the texture.
    pub fn with_type(mut self, value: TextureType) -> Self {
        self.inner.r#type = SDL_GPUTextureType(value as i32);
        self
    }

    /// The pixel format of the texture.
    pub fn with_format(mut self, format: TextureFormat) -> Self {
        self.inner.format = format;
        self
    }

    /// How the texture is intended to be used by the client.
    pub fn with_usage(mut self, value: TextureUsage) -> Self {
        self.inner.usage = value.0;
        self
    }

    /// The width of the texture.
    pub fn with_width(mut self, value: u32) -> Self {
        self.inner.width = value;
        self
    }

    /// The height of the texture.
    pub fn with_height(mut self, value: u32) -> Self {
        self.inner.height = value;
        self
    }

    /// The layer count or depth of the texture. This value is treated as a layer count on 2D array textures, and as a depth value on 3D textures.
    pub fn with_layer_count_or_depth(mut self, value: u32) -> Self {
        self.inner.layer_count_or_depth = value;
        self
    }

    /// The number of mip levels in the texture.
    pub fn with_num_levels(mut self, value: u32) -> Self {
        self.inner.num_levels = value;
        self
    }

    /// The number of samples per texel. Only applies if the texture is used as a render target.
    pub fn with_sample_count(mut self, value: SampleCount) -> Self {
        self.inner.sample_count = SDL_GPUSampleCount(value as i32);
        self
    }
}


#[derive(Default)]
pub struct SamplerCreateInfo {
    pub(super) inner: SDL_GPUSamplerCreateInfo,
}
impl SamplerCreateInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The minification filter to apply to lookups.
    pub fn with_min_filter(mut self, filter: Filter) -> Self {
        self.inner.min_filter = SDL_GPUFilter(filter as i32);
        self
    }

    /// The magnification filter to apply to lookups.
    pub fn with_mag_filter(mut self, filter: Filter) -> Self {
        self.inner.mag_filter = SDL_GPUFilter(filter as i32);
        self
    }

    /// The mipmap filter to apply to lookups.
    pub fn with_mipmap_mode(mut self, mode: SamplerMipmapMode) -> Self {
        self.inner.mipmap_mode = SDL_GPUSamplerMipmapMode(mode as i32);
        self
    }

    /// The addressing mode for U coordinates outside [0, 1).
    pub fn with_address_mode_u(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_u = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The addressing mode for V coordinates outside [0, 1).
    pub fn with_address_mode_v(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_v = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The addressing mode for W coordinates outside [0, 1).
    pub fn with_address_mode_w(mut self, mode: SamplerAddressMode) -> Self {
        self.inner.address_mode_w = SDL_GPUSamplerAddressMode(mode as i32);
        self
    }

    /// The bias to be added to mipmap LOD calculation.
    pub fn with_mip_lod_bias(mut self, value: f32) -> Self {
        self.inner.mip_lod_bias = value;
        self
    }

    /// The anisotropy value clamp used by the sampler. If enable_anisotropy is false, this is ignored.
    pub fn with_max_anisotropy(mut self, value: f32) -> Self {
        self.inner.max_anisotropy = value;
        self
    }

    /// The comparison operator to apply to fetched data before filtering.
    pub fn with_compare_op(mut self, value: CompareOp) -> Self {
        self.inner.compare_op = SDL_GPUCompareOp(value as i32);
        self
    }

    /// Clamps the minimum of the computed LOD value.
    pub fn with_min_lod(mut self, value: f32) -> Self {
        self.inner.min_lod = value;
        self
    }

    /// Clamps the maximum of the computed LOD value.
    pub fn with_max_lod(mut self, value: f32) -> Self {
        self.inner.max_lod = value;
        self
    }

    /// True to enable anisotropic filtering.
    pub fn with_enable_anisotropy(mut self, enable: bool) -> Self {
        self.inner.enable_anisotropy = enable;
        self
    }

    /// True to enable comparison against a reference value during lookups.
    pub fn with_enable_compare(mut self, enable: bool) -> Self {
        self.inner.enable_compare = enable;
        self
    }
}

#[derive(Default)]
pub struct TextureRegion<'a> {
    pub(super) inner: SDL_GPUTextureRegion,
    _marker: PhantomData<&'a Extern<SDL_GPUTexture>>,
}
impl<'a> TextureRegion<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture used in the copy operation.
    pub fn with_texture(mut self, texture: &'a Extern<SDL_GPUTexture>) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    /// The mip level index to transfer.
    pub fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.inner.mip_level = mip_level;
        self
    }

    /// The layer index to transfer.
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.inner.layer = layer;
        self
    }

    /// The left offset of the region.
    pub fn with_x(mut self, x: u32) -> Self {
        self.inner.x = x;
        self
    }

    /// The top offset of the region.
    pub fn with_y(mut self, y: u32) -> Self {
        self.inner.y = y;
        self
    }

    /// The front offset of the region.
    pub fn with_z(mut self, z: u32) -> Self {
        self.inner.z = z;
        self
    }

    /// The width of the region.
    pub fn with_width(mut self, width: u32) -> Self {
        self.inner.w = width;
        self
    }

    /// The height of the region.
    pub fn with_height(mut self, height: u32) -> Self {
        self.inner.h = height;
        self
    }

    /// The depth of the region.
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.inner.d = depth;
        self
    }
}

#[derive(Default)]
pub struct TextureTransferInfo<'a> {
    pub(super) inner: SDL_GPUTextureTransferInfo,
    _marker: PhantomData<&'a Extern<SDL_GPUTransferBuffer>>,
}
impl<'a> TextureTransferInfo<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// The transfer buffer used in the transfer operation.
    pub fn with_transfer_buffer(mut self, buffer: &'a Extern<SDL_GPUTransferBuffer>) -> Self {
        self.inner.transfer_buffer = buffer.raw();
        self
    }

    /// The starting byte of the image data in the transfer buffer.
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }

    /// The number of pixels from one row to the next.
    pub fn with_pixels_per_row(mut self, value: u32) -> Self {
        self.inner.pixels_per_row = value;
        self
    }

    /// The number of rows from one layer/depth-slice to the next.
    pub fn with_rows_per_layer(mut self, value: u32) -> Self {
        self.inner.rows_per_layer = value;
        self
    }
}


#[repr(C)]
#[derive(Default)]
pub struct BufferBinding<'a> {
    pub(super) inner: SDL_GPUBufferBinding,
    _marker: PhantomData<&'a Extern<SDL_GPUBuffer>>,
}
impl<'a> BufferBinding<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &'a Extern<SDL_GPUBuffer>) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default)]
pub struct TransferBufferLocation<'a> {
    pub(super) inner: SDL_GPUTransferBufferLocation,
    _marker: PhantomData<&'a Extern<SDL_GPUTransferBuffer>>,
}
impl<'a> TransferBufferLocation<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_transfer_buffer(mut self, transfer_buffer: &'a Extern<SDL_GPUTransferBuffer>) -> Self {
        self.inner.transfer_buffer = transfer_buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default)]
pub struct BufferRegion<'a> {
    pub(super) inner: SDL_GPUBufferRegion,
    _marker: PhantomData<&'a Extern<SDL_GPUBuffer>>,
}
impl<'a> BufferRegion<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &'a Extern<SDL_GPUBuffer>) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }

    pub fn with_size(mut self, size: u32) -> Self {
        self.inner.size = size;
        self
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct VertexBufferDescription {
    inner: SDL_GPUVertexBufferDescription,
}
impl VertexBufferDescription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_slot(mut self, value: u32) -> Self {
        self.inner.slot = value;
        self
    }

    pub fn with_pitch(mut self, value: u32) -> Self {
        self.inner.pitch = value;
        self
    }

    pub fn with_input_rate(mut self, value: VertexInputRate) -> Self {
        self.inner.input_rate = SDL_GPUVertexInputRate(value as i32);
        self
    }

    pub fn with_instance_step_rate(mut self, value: u32) -> Self {
        self.inner.instance_step_rate = value;
        self
    }
}



#[repr(C)]
#[derive(Default)]
pub struct VertexInputState<'a> {
    pub(super) inner: SDL_GPUVertexInputState,
    _marker: PhantomData<&'a [VertexBufferDescription]>,
}
impl<'a> VertexInputState<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_vertex_buffer_descriptions(mut self, value: &'a [VertexBufferDescription]) -> Self {
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


#[repr(C)]
#[derive(Default)]
pub struct RasterizerState {
    pub(super) inner: SDL_GPURasterizerState,
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
    pub(super) inner: SDL_GPUStencilOpState,
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
    pub(super) inner: SDL_GPUDepthStencilState,
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


#[derive(Default)]
pub struct GraphicsPipelineTargetInfo<'a> {
    pub(super) inner: SDL_GPUGraphicsPipelineTargetInfo,
    _marker: PhantomData<&'a [ColorTargetDescription]>,
}
impl<'a> GraphicsPipelineTargetInfo<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// A pointer to an array of color target descriptions.
    pub fn with_color_target_descriptions(mut self, value: &'a [ColorTargetDescription]) -> Self {
        self.inner.color_target_descriptions =
            value.as_ptr() as *const SDL_GPUColorTargetDescription;
        self.inner.num_color_targets = value.len() as u32;
        self
    }

    /// The pixel format of the depth-stencil target. Ignored if has_depth_stencil_target is false.
    pub fn with_depth_stencil_format(mut self, value: TextureFormat) -> Self {
        self.inner.depth_stencil_format = value;
        self
    }

    /// true specifies that the pipeline uses a depth-stencil target.
    pub fn with_has_depth_stencil_target(mut self, value: bool) -> Self {
        self.inner.has_depth_stencil_target = value;
        self
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
        self.inner.format = value;
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
pub struct TextureSamplerBinding<'a> {
    inner: SDL_GPUTextureSamplerBinding,
    _marker: PhantomData<&'a Extern<(SDL_GPUTexture, SDL_GPUSampler)>>,
}
impl<'a> TextureSamplerBinding<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture to bind. Must have been created with [`SDL_GPU_TEXTUREUSAGE_SAMPLER`].
    pub fn with_texture(mut self, texture: &'a Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    /// The sampler to bind.
    pub fn with_sampler(mut self, sampler: &'a Sampler) -> Self {
        self.inner.sampler = sampler.raw();
        self
    }
}


#[repr(C)]
#[derive(Default)]
pub struct StorageTextureReadWriteBinding<'a> {
    inner: SDL_GPUStorageTextureReadWriteBinding,
    _marker: PhantomData<&'a Extern<SDL_GPUTexture>>,
}
impl<'a> StorageTextureReadWriteBinding<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &'a Extern<SDL_GPUTexture>) -> Self {
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
pub struct StorageBufferReadWriteBinding<'a> {
    inner: SDL_GPUStorageBufferReadWriteBinding,
    _marker: PhantomData<&'a Extern<SDL_GPUBuffer>>,
}
impl<'a> StorageBufferReadWriteBinding<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &'a Extern<SDL_GPUBuffer>) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }
}
