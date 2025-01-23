use crate::{pixels::Color, sys};
use std::{
    ffi::{CStr, CString},
    ops::{BitAnd, BitOr},
    process::Command,
    str::FromStr,
};
use sys::gpu::{
    SDL_GPUColorTargetDescription, SDL_GPUColorTargetInfo, SDL_GPUCommandBuffer, SDL_GPUCompareOp,
    SDL_GPUComputePass, SDL_GPUCopyPass, SDL_GPUDepthStencilTargetInfo, SDL_GPUDevice,
    SDL_GPUGraphicsPipeline, SDL_GPUGraphicsPipelineCreateInfo, SDL_GPUGraphicsPipelineTargetInfo,
    SDL_GPUPrimitiveType, SDL_GPURenderPass, SDL_GPUShader, SDL_GPUTexture, SDL_GPUViewport,
};

macro_rules! impl_with {
    ($z:ident $x:ident $y:ident) => {
        #[inline]
        pub fn $z(mut self, value: $y) -> Self {
            self.inner.$x = value;
            self
        }
    };
    (usize $z:ident $x:ident $y:ident) => {
        #[inline]
        pub fn $z(mut self, value: usize) -> Self {
            self.inner.$x = value as $y;
            self
        }
    };
    (raw $x:ident) => {
        #[inline]
        pub fn raw(&self) -> *mut $x {
            self.inner
        }
    };
    (dont_drop $x:ident $msg:expr) => {
        impl Drop for $x {
            fn drop(&mut self) {
                println!($msg);
            }
        }
    };
    (enum_ops $x:ident) => {
        impl BitOr<$x> for $x {
            type Output = $x;
            fn bitor(self, rhs: $x) -> Self::Output {
                unsafe { std::mem::transmute((self as u32) | (rhs as u32)) }
            }
        }
        impl BitAnd<$x> for $x {
            type Output = $x;
            fn bitand(self, rhs: $x) -> Self::Output {
                unsafe { std::mem::transmute((self as u32) & (rhs as u32)) }
            }
        }
    };
}

//
// ENUMS
//
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LoadOp {
    #[default]
    Load = sys::gpu::SDL_GPU_LOADOP_LOAD.0 as u32,
    DontCare = sys::gpu::SDL_GPU_LOADOP_DONT_CARE.0 as u32,
    Clear = sys::gpu::SDL_GPU_LOADOP_CLEAR.0 as u32,
}
impl_with!(enum_ops LoadOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StoreOp {
    #[default]
    Store = sys::gpu::SDL_GPU_STOREOP_STORE.0 as u32,
    DontCare = sys::gpu::SDL_GPU_STOREOP_DONT_CARE.0 as u32,
    Resolve = sys::gpu::SDL_GPU_STOREOP_RESOLVE.0 as u32,
    ResolveAndStore = sys::gpu::SDL_GPU_STOREOP_RESOLVE_AND_STORE.0 as u32,
}
impl_with!(enum_ops StoreOp);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureFormat {
    // TODO: I should regex this somehow -- there must be a way
    #[default]
    Invalid = sys::gpu::SDL_GPU_TEXTUREFORMAT_INVALID.0 as u32,
    A8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_A8_UNORM.0 as u32,
    R8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_UNORM.0 as u32,
    R8g8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_UNORM.0 as u32,
    R8g8b8a8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM.0 as u32,
    R16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_UNORM.0 as u32,
    R16g16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_UNORM.0 as u32,
    R16g16b16a16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_UNORM.0 as u32,
    R10g10b10a2Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R10G10B10A2_UNORM.0 as u32,
    B5g6r5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B5G6R5_UNORM.0 as u32,
    B5g5r5a1Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B5G5R5A1_UNORM.0 as u32,
    B4g4r4a4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B4G4R4A4_UNORM.0 as u32,
    B8g8r8a8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM.0 as u32,
    Bc1RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC1_RGBA_UNORM.0 as u32,
    Bc2RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC2_RGBA_UNORM.0 as u32,
    Bc3RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC3_RGBA_UNORM.0 as u32,
    Bc4RUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC4_R_UNORM.0 as u32,
    Bc5RgUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC5_RG_UNORM.0 as u32,
    Bc7RgbaUnorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC7_RGBA_UNORM.0 as u32,
    Bc6hRgbFloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC6H_RGB_FLOAT.0 as u32,
    Bc6hRgbUfloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC6H_RGB_UFLOAT.0 as u32,
    R8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_SNORM.0 as u32,
    R8g8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_SNORM.0 as u32,
    R8g8b8a8Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_SNORM.0 as u32,
    R16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_SNORM.0 as u32,
    R16g16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_SNORM.0 as u32,
    R16g16b16a16Snorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_SNORM.0 as u32,
    R16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_FLOAT.0 as u32,
    R16g16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_FLOAT.0 as u32,
    R16g16b16a16Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_FLOAT.0 as u32,
    R32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_FLOAT.0 as u32,
    R32g32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_FLOAT.0 as u32,
    R32g32b32a32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_FLOAT.0 as u32,
    R11g11b10Ufloat = sys::gpu::SDL_GPU_TEXTUREFORMAT_R11G11B10_UFLOAT.0 as u32,
    R8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_UINT.0 as u32,
    R8g8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_UINT.0 as u32,
    R8g8b8a8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UINT.0 as u32,
    R16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_UINT.0 as u32,
    R16g16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_UINT.0 as u32,
    R16g16b16a16Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_UINT.0 as u32,
    R32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_UINT.0 as u32,
    R32g32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_UINT.0 as u32,
    R32g32b32a32Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_UINT.0 as u32,
    R8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8_INT.0 as u32,
    R8g8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8_INT.0 as u32,
    R8g8b8a8Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_INT.0 as u32,
    R16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16_INT.0 as u32,
    R16g16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16_INT.0 as u32,
    R16g16b16a16Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R16G16B16A16_INT.0 as u32,
    R32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32_INT.0 as u32,
    R32g32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32_INT.0 as u32,
    R32g32b32a32Int = sys::gpu::SDL_GPU_TEXTUREFORMAT_R32G32B32A32_INT.0 as u32,
    R8g8b8a8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM_SRGB.0 as u32,
    B8g8r8a8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM_SRGB.0 as u32,
    Bc1RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC1_RGBA_UNORM_SRGB.0 as u32,
    Bc2RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC2_RGBA_UNORM_SRGB.0 as u32,
    Bc3RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC3_RGBA_UNORM_SRGB.0 as u32,
    Bc7RgbaUnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_BC7_RGBA_UNORM_SRGB.0 as u32,
    D16Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_D16_UNORM.0 as u32,
    D24Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_D24_UNORM.0 as u32,
    D32Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_D32_FLOAT.0 as u32,
    D24UnormS8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_D24_UNORM_S8_UINT.0 as u32,
    D32FloatS8Uint = sys::gpu::SDL_GPU_TEXTUREFORMAT_D32_FLOAT_S8_UINT.0 as u32,
    Astc4x4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_UNORM.0 as u32,
    Astc5x4Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_UNORM.0 as u32,
    Astc5x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_UNORM.0 as u32,
    Astc6x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_UNORM.0 as u32,
    Astc6x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_UNORM.0 as u32,
    Astc8x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_UNORM.0 as u32,
    Astc8x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_UNORM.0 as u32,
    Astc8x8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_UNORM.0 as u32,
    Astc10x5Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_UNORM.0 as u32,
    Astc10x6Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_UNORM.0 as u32,
    Astc10x8Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_UNORM.0 as u32,
    Astc10x10Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_UNORM.0 as u32,
    Astc12x10Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_UNORM.0 as u32,
    Astc12x12Unorm = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_UNORM.0 as u32,
    Astc4x4UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_UNORM_SRGB.0 as u32,
    Astc5x4UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_UNORM_SRGB.0 as u32,
    Astc5x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_UNORM_SRGB.0 as u32,
    Astc6x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_UNORM_SRGB.0 as u32,
    Astc6x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_UNORM_SRGB.0 as u32,
    Astc8x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_UNORM_SRGB.0 as u32,
    Astc8x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_UNORM_SRGB.0 as u32,
    Astc8x8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_UNORM_SRGB.0 as u32,
    Astc10x5UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_UNORM_SRGB.0 as u32,
    Astc10x6UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_UNORM_SRGB.0 as u32,
    Astc10x8UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_UNORM_SRGB.0 as u32,
    Astc10x10UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_UNORM_SRGB.0 as u32,
    Astc12x10UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_UNORM_SRGB.0 as u32,
    Astc12x12UnormSrgb = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_UNORM_SRGB.0 as u32,
    Astc4x4Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_4x4_FLOAT.0 as u32,
    Astc5x4Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x4_FLOAT.0 as u32,
    Astc5x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_5x5_FLOAT.0 as u32,
    Astc6x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x5_FLOAT.0 as u32,
    Astc6x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_6x6_FLOAT.0 as u32,
    Astc8x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x5_FLOAT.0 as u32,
    Astc8x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x6_FLOAT.0 as u32,
    Astc8x8Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_8x8_FLOAT.0 as u32,
    Astc10x5Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x5_FLOAT.0 as u32,
    Astc10x6Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x6_FLOAT.0 as u32,
    Astc10x8Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x8_FLOAT.0 as u32,
    Astc10x10Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_10x10_FLOAT.0 as u32,
    Astc12x10Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x10_FLOAT.0 as u32,
    Astc12x12Float = sys::gpu::SDL_GPU_TEXTUREFORMAT_ASTC_12x12_FLOAT.0 as u32,
}
impl_with!(enum_ops TextureFormat);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ShaderFormat {
    #[default]
    Invalid = sys::gpu::SDL_GPU_SHADERFORMAT_INVALID as u32,
    Dxbc = sys::gpu::SDL_GPU_SHADERFORMAT_DXBC as u32,
    Dxil = sys::gpu::SDL_GPU_SHADERFORMAT_DXIL as u32,
    MetalLib = sys::gpu::SDL_GPU_SHADERFORMAT_METALLIB as u32,
    Msl = sys::gpu::SDL_GPU_SHADERFORMAT_MSL as u32,
    Private = sys::gpu::SDL_GPU_SHADERFORMAT_PRIVATE as u32,
    SpirV = sys::gpu::SDL_GPU_SHADERFORMAT_SPIRV as u32,
}
impl_with!(enum_ops ShaderFormat);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureUsage {
    #[default]
    Invalid = 0,
    ComputeStorageWrite = sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_WRITE,
    ComputeStorageRead = sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_READ,
    ComputeSimultaneousReadWrite =
        sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_SIMULTANEOUS_READ_WRITE,
    DepthStencilTarget = sys::gpu::SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
    GraphicsStorageRead = sys::gpu::SDL_GPU_TEXTUREUSAGE_GRAPHICS_STORAGE_READ,
    Sampler = sys::gpu::SDL_GPU_TEXTUREUSAGE_SAMPLER,
    ColorTarget = sys::gpu::SDL_GPU_TEXTUREUSAGE_COLOR_TARGET,
}
impl_with!(enum_ops TextureUsage);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ShaderStage {
    #[default]
    Vertex = sys::gpu::SDL_GPU_SHADERSTAGE_VERTEX.0 as u32,
    Fragment = sys::gpu::SDL_GPU_SHADERSTAGE_FRAGMENT.0 as u32,
}
impl_with!(enum_ops ShaderStage);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PrimitiveType {
    #[default]
    TriangleList = sys::gpu::SDL_GPU_PRIMITIVETYPE_TRIANGLELIST.0 as u32,
    TriangleStrip = sys::gpu::SDL_GPU_PRIMITIVETYPE_TRIANGLESTRIP.0 as u32,
    LineList = sys::gpu::SDL_GPU_PRIMITIVETYPE_LINELIST.0 as u32,
    LineStrip = sys::gpu::SDL_GPU_PRIMITIVETYPE_LINESTRIP.0 as u32,
    PointList = sys::gpu::SDL_GPU_PRIMITIVETYPE_POINTLIST.0 as u32,
}
impl_with!(enum_ops PrimitiveType);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FillMode {
    #[default]
    Fill = sys::gpu::SDL_GPU_FILLMODE_FILL.0 as u32,
    Line = sys::gpu::SDL_GPU_FILLMODE_LINE.0 as u32,
}
impl_with!(enum_ops FillMode);

//
// STRUCTS
//
pub struct CommandBuffer {
    inner: *mut SDL_GPUCommandBuffer,
    swapchain: Texture,
}
impl Default for CommandBuffer {
    fn default() -> Self {
        Self {
            inner: std::ptr::null_mut(),
            swapchain: Texture::default(),
        }
    }
}
impl CommandBuffer {
    impl_with!(raw SDL_GPUCommandBuffer);
    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<(), crate::sdl::Error> {
        if unsafe { sys::gpu::SDL_SubmitGPUCommandBuffer(self.inner) } {
            Ok(())
        } else {
            Err(crate::sdl::get_error())
        }
    }
    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(&mut self) {
        unsafe {
            sys::gpu::SDL_CancelGPUCommandBuffer(self.inner);
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilTargetInfo {
    inner: SDL_GPUDepthStencilTargetInfo,
}
impl DepthStencilTargetInfo {
    impl_with!(with_clear_depth clear_depth f32);
    impl_with!(with_clear_stencil clear_stencil u8);
    impl_with!(with_cycle cycle bool);
    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }
    pub fn with_stencil_load_op(mut self, value: LoadOp) -> Self {
        self.inner.stencil_load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }
    pub fn with_stencil_store_op(mut self, value: StoreOp) -> Self {
        self.inner.stencil_store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ColorTargetInfo {
    inner: SDL_GPUColorTargetInfo,
}
impl ColorTargetInfo {
    pub fn with_texture(mut self, texture: &Texture) -> Self {
        self.inner.texture = texture.inner;
        self
    }
    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }
    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
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
type Viewport = SDL_GPUViewport;
pub struct RenderPass {
    inner: *mut SDL_GPURenderPass,
}
impl RenderPass {
    impl_with!(raw SDL_GPURenderPass);
    pub fn bind_graphics_pipeline(&self, gp: &GraphicsPipeline) {
        unsafe {
            sys::gpu::SDL_BindGPUGraphicsPipeline(self.inner, gp.inner);
        }
    }
    pub fn draw_primitives(
        &self,
        num_vertices: usize,
        num_instances: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        unsafe {
            sys::gpu::SDL_DrawGPUPrimitives(
                self.inner,
                num_vertices as u32,
                num_instances as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }
}

pub struct CopyPass {
    inner: *mut SDL_GPUCopyPass,
}
impl CopyPass {
    impl_with!(raw SDL_GPUCopyPass);
}

pub struct ComputePass {
    inner: *mut SDL_GPUComputePass,
}
impl ComputePass {
    impl_with!(raw SDL_GPUComputePass);
}

pub struct Shader {
    inner: *mut SDL_GPUShader,
}
impl Shader {
    impl_with!(raw SDL_GPUShader);
    pub fn release(self, device: &Device) {
        unsafe {
            sys::gpu::SDL_ReleaseGPUShader(device.raw(), self.inner);
            std::mem::forget(self);
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    inner: *mut SDL_GPUTexture,
    size: Option<(u32, u32)>,
}
impl Default for Texture {
    fn default() -> Self {
        Self {
            inner: std::ptr::null_mut(),
            size: Some((0, 0)),
        }
    }
}
impl Texture {
    impl_with!(raw SDL_GPUTexture);
    pub fn width(&self) -> Option<usize> {
        self.size.map(|(a, _)| a as usize)
    }
    pub fn height(&self) -> Option<usize> {
        self.size.map(|(_, b)| b as usize)
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a Device,
    entrypoint: std::ffi::CString,
    inner: sdl3_sys::everything::SDL_GPUShaderCreateInfo,
}
impl<'a> ShaderBuilder<'a> {
    impl_with!(usize with_samplers num_samplers u32);
    impl_with!(usize with_storage_buffers num_storage_buffers u32);
    impl_with!(usize with_storage_textures num_storage_textures u32);
    impl_with!(usize with_uniform_buffers num_uniform_buffers u32);
    pub fn with_code(mut self, fmt: ShaderFormat, code: &'a [u8], stage: ShaderStage) -> Self {
        self.inner.format = fmt as u32;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len() as usize;
        self.inner.stage = unsafe { std::mem::transmute(stage as u32) };
        self
    }
    pub fn with_entrypoint(mut self, entry_point: &'a str) -> Self {
        self.entrypoint = CString::new(entry_point).unwrap(); //need to save
        self.inner.entrypoint = self.entrypoint.as_c_str().as_ptr();
        self
    }
    pub fn build(self) -> Result<Shader, crate::sdl::Error> {
        let p = unsafe { sys::gpu::SDL_CreateGPUShader(self.device.inner, &self.inner) };
        if p != std::ptr::null_mut() {
            Ok(Shader { inner: p })
        } else {
            Err(crate::sdl::get_error())
        }
    }
}

#[derive(Default)]
pub struct ColorTargetDescriptionBuilder {
    inner: SDL_GPUColorTargetDescription,
}
#[repr(C)]
#[derive(Default)]
pub struct ColorTargetDescription {
    inner: SDL_GPUColorTargetDescription,
}
impl ColorTargetDescriptionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_format(mut self, value: TextureFormat) -> Self {
        self.inner.format = unsafe { std::mem::transmute(value as u32) };
        self
    }
    pub fn build(self) -> ColorTargetDescription {
        ColorTargetDescription { inner: self.inner }
    }
}

#[repr(C)]
pub struct GraphicsPipelineBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUGraphicsPipelineCreateInfo,
}
impl<'a> GraphicsPipelineBuilder<'a> {
    pub fn with_fragment_shader(mut self, value: &'a Shader) -> Self {
        self.inner.fragment_shader = value.raw();
        self
    }
    pub fn with_vertex_shader(mut self, value: &'a Shader) -> Self {
        self.inner.vertex_shader = value.raw();
        self
    }
    pub fn with_primitive_type(mut self, value: PrimitiveType) -> Self {
        self.inner.primitive_type = unsafe { std::mem::transmute(value as u32) };
        self
    }
    pub fn with_target_info(mut self, value: &'a [ColorTargetDescription]) -> Self {
        self.inner.target_info.color_target_descriptions =
            value.as_ptr() as *const SDL_GPUColorTargetDescription; //heavy promise
        self.inner.target_info.num_color_targets = value.len() as u32;
        self
    }
    pub fn with_fill_mode(mut self, value: FillMode) -> Self {
        self.inner.rasterizer_state.fill_mode = unsafe { std::mem::transmute(value as u32) };
        self
    }
    pub fn build(self) -> Result<GraphicsPipeline, crate::sdl::Error> {
        //self.inner.props = sys::gpu::SDL_PROP_GPU_DEVICE_CREATE_NAME_STRING as u32;
        let p = unsafe {
            //sys::gpu::SDL_CreateGPUGraphicsPipeline(self.device.raw(), &self.inner)
            sys::gpu::SDL_CreateGPUGraphicsPipeline(
                self.device.raw(),
                &SDL_GPUGraphicsPipelineCreateInfo {
                    vertex_shader: self.inner.vertex_shader,
                    fragment_shader: self.inner.fragment_shader,
                    primitive_type: self.inner.primitive_type,
                    target_info: SDL_GPUGraphicsPipelineTargetInfo {
                        color_target_descriptions: [SDL_GPUColorTargetDescription {
                            format: sys::gpu::SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM,
                            ..Default::default()
                        }]
                        .as_ptr(),
                        num_color_targets: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        };
        if p != std::ptr::null_mut() {
            Ok(GraphicsPipeline { inner: p })
        } else {
            Err(crate::sdl::get_error())
        }
    }
}
pub struct GraphicsPipeline {
    inner: *mut SDL_GPUGraphicsPipeline,
}

#[derive(Debug)]
pub struct Device {
    inner: *mut SDL_GPUDevice,
}
impl Device {
    impl_with!(raw SDL_GPUDevice);
    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(flags: ShaderFormat, debug_mode: bool) -> Self {
        Self {
            inner: unsafe {
                sys::gpu::SDL_CreateGPUDevice(flags as u32, debug_mode, std::ptr::null())
            },
        }
    }
    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn with_window(self, w: &crate::video::Window) -> Result<Self, crate::sdl::Error> {
        let p = unsafe { sys::gpu::SDL_ClaimWindowForGPUDevice(self.inner, w.raw()) };
        if p {
            Ok(self)
        } else {
            Err(crate::sdl::get_error())
        }
    }
    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn acquire_command_buffer(&self) -> CommandBuffer {
        CommandBuffer {
            inner: unsafe { sys::gpu::SDL_AcquireGPUCommandBuffer(self.inner) },
            ..Default::default()
        }
    }
    pub fn create_shader(&self) -> ShaderBuilder {
        ShaderBuilder {
            device: self,
            entrypoint: std::ffi::CString::new("main").unwrap(),
            inner: sys::gpu::SDL_GPUShaderCreateInfo::default(),
        }
    }
    #[doc(alias = "SDL_SetGPUViewport")]
    pub fn set_viewport(&self, render_pass: &RenderPass, viewport: Viewport) {
        unsafe {
            sys::gpu::SDL_SetGPUViewport(render_pass.inner, &viewport);
        }
    }
    pub fn get_swapchain_texture_format(&self, w: &crate::video::Window) -> TextureFormat {
        unsafe {
            std::mem::transmute(sys::gpu::SDL_GetGPUSwapchainTextureFormat(self.inner, w.raw()).0)
        }
    }
    // May not:
    // - Live longer than command buffer
    // - Or than the GPU object
    #[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
    pub fn wait_and_acquire_swapchain_texture<'a>(
        &self,
        w: &crate::video::Window,
        command_buffer: &'a mut CommandBuffer,
    ) -> Result<&'a Texture, crate::sdl::Error> {
        unsafe {
            let mut swapchain = std::ptr::null_mut();
            let mut width = 0;
            let mut height = 0;
            if sys::gpu::SDL_WaitAndAcquireGPUSwapchainTexture(
                command_buffer.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            ) {
                command_buffer.swapchain.inner = swapchain;
                command_buffer.swapchain.size = Some((width, height));
                Ok(&command_buffer.swapchain)
            } else {
                Err(crate::sdl::get_error())
            }
        }
    }
    // Only use if you are aware of the timings: Will not wait for freed memory, potentially
    // causing an out of memory error
    // May not:
    // - Live longer than command buffer
    // - Or than the GPU object
    #[doc(alias = "SDL_AcquireGPUSwapchainTexture")]
    pub fn acquire_swapchain_texture<'a>(
        &self,
        w: &crate::video::Window,
        command_buffer: &'a mut CommandBuffer,
    ) -> Result<&'a Texture, crate::sdl::Error> {
        unsafe {
            let mut swapchain = std::ptr::null_mut();
            let mut width = 0;
            let mut height = 0;
            if sys::gpu::SDL_AcquireGPUSwapchainTexture(
                command_buffer.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            ) {
                command_buffer.swapchain.inner = swapchain;
                command_buffer.swapchain.size = Some((width, height));
                Ok(&command_buffer.swapchain)
            } else {
                Err(crate::sdl::get_error())
            }
        }
    }
    // You cannot begin another render pass, or begin a compute pass or copy pass until you have ended the render pass.
    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn begin_render_pass(
        &self,
        command_buffer: &CommandBuffer,
        color_info: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
    ) -> Result<RenderPass, crate::sdl::Error> {
        let p = unsafe {
            sys::gpu::SDL_BeginGPURenderPass(
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
        if p != std::ptr::null_mut() {
            Ok(RenderPass { inner: p })
        } else {
            Err(crate::sdl::get_error())
        }
    }
    #[doc(alias = "SDL_EndGPURenderPass")]
    pub fn end_render_pass(&self, pass: RenderPass) {
        unsafe {
            sys::gpu::SDL_EndGPURenderPass(pass.inner);
            std::mem::forget(pass);
        }
    }
    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn begin_copy_pass(
        &self,
        command_buffer: &CommandBuffer,
    ) -> Result<CopyPass, crate::sdl::Error> {
        let p = unsafe { sys::gpu::SDL_BeginGPUCopyPass(command_buffer.inner) };
        if p != std::ptr::null_mut() {
            Ok(CopyPass { inner: p })
        } else {
            Err(crate::sdl::get_error())
        }
    }
    #[doc(alias = "SDL_EndGPURenderPass")]
    pub fn end_copy_pass(&self, pass: CopyPass) {
        unsafe {
            sys::gpu::SDL_EndGPUCopyPass(pass.inner);
            std::mem::forget(pass);
        }
    }
    pub fn create_graphics_pipeline<'a>(&'a self) -> GraphicsPipelineBuilder<'a> {
        GraphicsPipelineBuilder {
            device: self,
            inner: SDL_GPUGraphicsPipelineCreateInfo::default(),
        }
    }
    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn get_shader_formats(&self) -> ShaderFormat {
        unsafe { std::mem::transmute(sys::gpu::SDL_GetGPUShaderFormats(self.inner)) }
    }
    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKSuspendGPU")]
    pub fn gdk_suspend(&self) {
        unsafe {
            sys::gpu::SDL_GDKSuspendGPU(self.inner);
        }
    }
    #[cfg(target_os = "xbox")]
    #[doc(alias = "SDL_GDKResumeGPU")]
    pub fn gdk_resume(&self) {
        unsafe {
            sys::gpu::SDL_GDKResumeGPU(self.inner);
        }
    }
}

impl Drop for Device {
    #[doc(alias = "SDL_DestroyGPUDevice")]
    fn drop(&mut self) {
        unsafe {
            sys::gpu::SDL_DestroyGPUDevice(self.inner);
        }
    }
}
