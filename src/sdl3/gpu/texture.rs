use crate::gpu::{
    CompareOp, Device, Filter, SampleCount, SamplerAddressMode, SamplerMipmapMode, TextureFormat,
    TextureType, TextureUsage, TransferBuffer, WeakDevice,
};
use std::{marker::PhantomData, sync::Arc};
use sys::gpu::{
    SDL_GPUCompareOp, SDL_GPUFilter, SDL_GPUSampleCount, SDL_GPUSampler, SDL_GPUSamplerAddressMode,
    SDL_GPUSamplerCreateInfo, SDL_GPUSamplerMipmapMode, SDL_GPUTexture, SDL_GPUTextureCreateInfo,
    SDL_GPUTextureFormat, SDL_GPUTextureRegion, SDL_GPUTextureSamplerBinding,
    SDL_GPUTextureTransferInfo, SDL_GPUTextureType, SDL_ReleaseGPUSampler, SDL_ReleaseGPUTexture,
};

#[derive(Default)]
pub struct TextureTransferInfo {
    pub(super) inner: SDL_GPUTextureTransferInfo,
}
impl TextureTransferInfo {
    pub fn new() -> Self {
        Default::default()
    }

    /// The transfer buffer used in the transfer operation.
    pub fn with_transfer_buffer(mut self, buffer: &TransferBuffer) -> Self {
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

#[derive(Default)]
pub struct TextureRegion {
    pub(super) inner: SDL_GPUTextureRegion,
}
impl TextureRegion {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture used in the copy operation.
    pub fn with_texture(mut self, texture: &Texture) -> Self {
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

/// Manages the raw `SDL_GPUSampler` pointer and releases it on drop
struct SamplerContainer {
    raw: *mut SDL_GPUSampler,
    device: WeakDevice,
}
impl Drop for SamplerContainer {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { SDL_ReleaseGPUSampler(device.raw(), self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct Sampler {
    inner: Arc<SamplerContainer>,
}
impl Sampler {
    pub(super) fn new(device: &Device, raw_sampler: *mut SDL_GPUSampler) -> Self {
        Self {
            inner: Arc::new(SamplerContainer {
                raw: raw_sampler,
                device: device.weak(),
            }),
        }
    }

    #[inline]
    fn raw(&self) -> *mut SDL_GPUSampler {
        self.inner.raw
    }
}

#[repr(C)]
#[derive(Default)]
pub struct TextureSamplerBinding {
    inner: SDL_GPUTextureSamplerBinding,
}
impl TextureSamplerBinding {
    pub fn new() -> Self {
        Default::default()
    }

    /// The texture to bind. Must have been created with [`SDL_GPU_TEXTUREUSAGE_SAMPLER`].
    pub fn with_texture(mut self, texture: &Texture<'static>) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    /// The sampler to bind.
    pub fn with_sampler(mut self, sampler: &Sampler) -> Self {
        self.inner.sampler = sampler.raw();
        self
    }
}

/// Manages the raw `SDL_GPUTexture` pointer and releases it on drop (if necessary)
enum TextureContainer {
    /// The user is responsible for releasing this texture
    UserManaged {
        raw: *mut SDL_GPUTexture,
        device: WeakDevice,
    },
    /// SDL owns this texture and is responsible for releasing it
    SdlManaged { raw: *mut SDL_GPUTexture },
}
impl TextureContainer {
    fn raw(&self) -> *mut SDL_GPUTexture {
        match self {
            Self::UserManaged { raw, .. } => *raw,
            Self::SdlManaged { raw } => *raw,
        }
    }
}
impl Drop for TextureContainer {
    #[doc(alias = "SDL_ReleaseGPUTexture")]
    fn drop(&mut self) {
        match self {
            Self::UserManaged { raw, device } => {
                if let Some(device) = device.upgrade() {
                    unsafe { SDL_ReleaseGPUTexture(device.raw(), *raw) };
                }
            }
            _ => {}
        }
    }
}

// Texture has a lifetime for the case of the special swapchain texture that must not
// live longer than the command buffer it is bound to. Otherwise, it is always 'static.
#[derive(Clone)]
pub struct Texture<'a> {
    inner: Arc<TextureContainer>,
    width: u32,
    height: u32,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> Texture<'a> {
    pub(super) fn new(
        device: &Device,
        raw: *mut SDL_GPUTexture,
        width: u32,
        height: u32,
    ) -> Texture<'a> {
        Texture {
            inner: Arc::new(TextureContainer::UserManaged {
                raw,
                device: device.weak(),
            }),
            width,
            height,
            _phantom: Default::default(),
        }
    }

    pub(super) fn new_sdl_managed(
        raw: *mut SDL_GPUTexture,
        width: u32,
        height: u32,
    ) -> Texture<'a> {
        Texture {
            inner: Arc::new(TextureContainer::SdlManaged { raw }),
            width,
            height,
            _phantom: Default::default(),
        }
    }

    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUTexture {
        self.inner.raw()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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
        self.inner.format = SDL_GPUTextureFormat(format as i32);
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
