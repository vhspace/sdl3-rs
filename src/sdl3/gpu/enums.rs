use crate::sys;
use std::ops::{BitAnd, BitOr};
use sys::gpu::{SDL_GPUBlendFactor, SDL_GPUBlendOp};

macro_rules! impl_with {
    (bitwise_and_or $x:ident $prim:ident) => {
        impl BitOr<$x> for $x {
            type Output = $x;
            fn bitor(self, rhs: $x) -> Self::Output {
                $x (self.0 | rhs.0)
            }
        }
        impl BitAnd<$x> for $x {
            type Output = $x;
            fn bitand(self, rhs: $x) -> Self::Output {
                $x (self.0 & rhs.0)
            }
        }
    };
}

pub type LoadOp = sys::gpu::SDL_GPULoadOp;
pub type StoreOp = sys::gpu::SDL_GPUStoreOp;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureFormat {
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderFormat(pub sys::gpu::SDL_GPUShaderFormat);
impl ShaderFormat {
    pub const INVALID: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_INVALID);
    pub const DXBC: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_DXBC);
    pub const DXIL: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_DXIL);
    pub const METALLIB: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_METALLIB);
    pub const MSL: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_MSL);
    pub const PRIVATE: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_PRIVATE);
    pub const SPIRV: Self = Self(sys::gpu::SDL_GPU_SHADERFORMAT_SPIRV);
}
impl_with!(bitwise_and_or ShaderFormat u32);

pub struct TextureUsage(pub sys::gpu::SDL_GPUTextureUsageFlags);
impl TextureUsage {
    pub const INVALID: Self =
        Self(0);
    pub const COMPUTE_STORAGE_WRITE: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_WRITE);
    pub const COMPUTE_STORAGE_READ: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_READ);
    pub const COMPUTE_STORAGE_SIMULTANEOUS_READ_WRITE: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_SIMULTANEOUS_READ_WRITE);
    pub const DEPTH_STENCIL_TARGET: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET);
    pub const GRAPHICS_STORAGE_READ: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_GRAPHICS_STORAGE_READ);
    pub const SAMPLER: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_SAMPLER);
    pub const COLOR_TARGET: Self =
        Self(sys::gpu::SDL_GPU_TEXTUREUSAGE_COLOR_TARGET);
}
impl_with!(bitwise_and_or TextureUsage u32);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ShaderStage {
    #[default]
    Vertex = sys::gpu::SDL_GPU_SHADERSTAGE_VERTEX.0 as u32,
    Fragment = sys::gpu::SDL_GPU_SHADERSTAGE_FRAGMENT.0 as u32,
}

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FillMode {
    #[default]
    Fill = sys::gpu::SDL_GPU_FILLMODE_FILL.0 as u32,
    Line = sys::gpu::SDL_GPU_FILLMODE_LINE.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CullMode {
    #[default]
    None = sys::gpu::SDL_GPUCullMode::NONE.0 as u32,
    Front = sys::gpu::SDL_GPUCullMode::FRONT.0 as u32,
    Back = sys::gpu::SDL_GPUCullMode::BACK.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FrontFace {
    #[default]
    CounterClockwise = sys::gpu::SDL_GPUFrontFace::COUNTER_CLOCKWISE.0 as u32,
    Clockwise = sys::gpu::SDL_GPUFrontFace::CLOCKWISE.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CompareOp {
    #[default]
    Invalid = sys::gpu::SDL_GPUCompareOp::INVALID.0 as u32,
    Never = sys::gpu::SDL_GPUCompareOp::NEVER.0 as u32,
    Less = sys::gpu::SDL_GPUCompareOp::LESS.0 as u32,
    Equal = sys::gpu::SDL_GPUCompareOp::EQUAL.0 as u32,
    LessOrEqual = sys::gpu::SDL_GPUCompareOp::LESS_OR_EQUAL.0 as u32,
    Greater = sys::gpu::SDL_GPUCompareOp::GREATER.0 as u32,
    NotEqual = sys::gpu::SDL_GPUCompareOp::NOT_EQUAL.0 as u32,
    GreaterOrEqual = sys::gpu::SDL_GPUCompareOp::GREATER_OR_EQUAL.0 as u32,
    Always = sys::gpu::SDL_GPUCompareOp::ALWAYS.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StencilOp {
    #[default]
    Invalid = sys::gpu::SDL_GPUStencilOp::INVALID.0 as u32,
    Keep = sys::gpu::SDL_GPUStencilOp::KEEP.0 as u32,
    Zero = sys::gpu::SDL_GPUStencilOp::ZERO.0 as u32,
    Replace = sys::gpu::SDL_GPUStencilOp::REPLACE.0 as u32,
    IncrementAndClamp = sys::gpu::SDL_GPUStencilOp::INCREMENT_AND_CLAMP.0 as u32,
    DecrementAndClamp = sys::gpu::SDL_GPUStencilOp::DECREMENT_AND_CLAMP.0 as u32,
    Invert = sys::gpu::SDL_GPUStencilOp::INVERT.0 as u32,
    IncrementAndWrap = sys::gpu::SDL_GPUStencilOp::INCREMENT_AND_WRAP.0 as u32,
    DecrementAndWrap = sys::gpu::SDL_GPUStencilOp::DECREMENT_AND_WRAP.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TextureType {
    #[default]
    _2D = sys::gpu::SDL_GPUTextureType::_2D.0 as u32,
    _2DArray = sys::gpu::SDL_GPUTextureType::_2D_ARRAY.0 as u32,
    _3D = sys::gpu::SDL_GPUTextureType::_3D.0 as u32,
    Cube = sys::gpu::SDL_GPUTextureType::CUBE.0 as u32,
    CubeArray = sys::gpu::SDL_GPUTextureType::CUBE_ARRAY.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SampleCount {
    #[default]
    NoMultiSampling = sys::gpu::SDL_GPUSampleCount::_1.0 as u32,
    MSAA2x = sys::gpu::SDL_GPUSampleCount::_2.0 as u32,
    MSAA4x = sys::gpu::SDL_GPUSampleCount::_4.0 as u32,
    MSAA8x = sys::gpu::SDL_GPUSampleCount::_8.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VertexElementFormat {
    #[default]
    Invalid = sys::gpu::SDL_GPUVertexElementFormat::INVALID.0 as u32,
    Int = sys::gpu::SDL_GPUVertexElementFormat::INT.0 as u32,
    Int2 = sys::gpu::SDL_GPUVertexElementFormat::INT2.0 as u32,
    Int3 = sys::gpu::SDL_GPUVertexElementFormat::INT3.0 as u32,
    Int4 = sys::gpu::SDL_GPUVertexElementFormat::INT4.0 as u32,
    Uint = sys::gpu::SDL_GPUVertexElementFormat::UINT.0 as u32,
    Uint2 = sys::gpu::SDL_GPUVertexElementFormat::UINT2.0 as u32,
    Uint3 = sys::gpu::SDL_GPUVertexElementFormat::UINT3.0 as u32,
    Uint4 = sys::gpu::SDL_GPUVertexElementFormat::UINT4.0 as u32,
    Float = sys::gpu::SDL_GPUVertexElementFormat::FLOAT.0 as u32,
    Float2 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT2.0 as u32,
    Float3 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT3.0 as u32,
    Float4 = sys::gpu::SDL_GPUVertexElementFormat::FLOAT4.0 as u32,
    Byte2 = sys::gpu::SDL_GPUVertexElementFormat::BYTE2.0 as u32,
    Byte4 = sys::gpu::SDL_GPUVertexElementFormat::BYTE4.0 as u32,
    Ubyte2 = sys::gpu::SDL_GPUVertexElementFormat::UBYTE2.0 as u32,
    Ubyte4 = sys::gpu::SDL_GPUVertexElementFormat::UBYTE4.0 as u32,
    Byte2Norm = sys::gpu::SDL_GPUVertexElementFormat::BYTE2_NORM.0 as u32,
    Byte4Norm = sys::gpu::SDL_GPUVertexElementFormat::BYTE4_NORM.0 as u32,
    Ubyte2Norm = sys::gpu::SDL_GPUVertexElementFormat::UBYTE2_NORM.0 as u32,
    Ubyte4Norm = sys::gpu::SDL_GPUVertexElementFormat::UBYTE4_NORM.0 as u32,
    Short2 = sys::gpu::SDL_GPUVertexElementFormat::SHORT2.0 as u32,
    Short4 = sys::gpu::SDL_GPUVertexElementFormat::SHORT4.0 as u32,
    Ushort2 = sys::gpu::SDL_GPUVertexElementFormat::USHORT2.0 as u32,
    Ushort4 = sys::gpu::SDL_GPUVertexElementFormat::USHORT4.0 as u32,
    Short2Norm = sys::gpu::SDL_GPUVertexElementFormat::SHORT2_NORM.0 as u32,
    Short4Norm = sys::gpu::SDL_GPUVertexElementFormat::SHORT4_NORM.0 as u32,
    Ushort2Norm = sys::gpu::SDL_GPUVertexElementFormat::USHORT2_NORM.0 as u32,
    Ushort4Norm = sys::gpu::SDL_GPUVertexElementFormat::USHORT4_NORM.0 as u32,
    Half2 = sys::gpu::SDL_GPUVertexElementFormat::HALF2.0 as u32,
    Half4 = sys::gpu::SDL_GPUVertexElementFormat::HALF4.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Filter {
    #[default]
    Nearest = sys::gpu::SDL_GPUFilter::NEAREST.0 as u32,
    Linear = sys::gpu::SDL_GPUFilter::LINEAR.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SamplerMipmapMode {
    #[default]
    Nearest = sys::gpu::SDL_GPUSamplerMipmapMode::NEAREST.0 as u32,
    Linear = sys::gpu::SDL_GPUSamplerMipmapMode::LINEAR.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SamplerAddressMode {
    #[default]
    Repeat = sys::gpu::SDL_GPUSamplerAddressMode::REPEAT.0 as u32,
    MirroredRepeat = sys::gpu::SDL_GPUSamplerAddressMode::MIRRORED_REPEAT.0 as u32,
    ClampToEdge = sys::gpu::SDL_GPUSamplerAddressMode::CLAMP_TO_EDGE.0 as u32,
}

pub type IndexElementSize = sys::gpu::SDL_GPUIndexElementSize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VertexInputRate {
    #[default]
    Vertex = sys::gpu::SDL_GPUVertexInputRate::VERTEX.0 as u32,
    Instance = sys::gpu::SDL_GPUVertexInputRate::INSTANCE.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferUsageFlags(pub sys::gpu::SDL_GPUBufferUsageFlags);
impl BufferUsageFlags {
    pub const VERTEX                : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_VERTEX);
    pub const INDEX                 : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_INDEX);
    pub const INDIRECT              : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_INDIRECT);
    pub const GRAPHICS_STORAGE_READ : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_GRAPHICS_STORAGE_READ);
    pub const COMPUTE_STORAGE_READ  : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ);
    pub const COMPUTE_STORAGE_WRITE : Self = Self(sys::gpu::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE);
}

pub type TransferBufferUsage = sys::gpu::SDL_GPUTransferBufferUsage;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BlendFactor {
    #[default]
    Invalid = SDL_GPUBlendFactor::INVALID.0 as u32,
    Zero = SDL_GPUBlendFactor::ZERO.0 as u32,
    One = SDL_GPUBlendFactor::ONE.0 as u32,
    SrcColor = SDL_GPUBlendFactor::SRC_COLOR.0 as u32,
    OneMinusSrcColor = SDL_GPUBlendFactor::ONE_MINUS_SRC_COLOR.0 as u32,
    DstColor = SDL_GPUBlendFactor::DST_COLOR.0 as u32,
    OneMinusDstColor = SDL_GPUBlendFactor::ONE_MINUS_DST_COLOR.0 as u32,
    SrcAlpha = SDL_GPUBlendFactor::SRC_ALPHA.0 as u32,
    OneMinusSrcAlpha = SDL_GPUBlendFactor::ONE_MINUS_SRC_ALPHA.0 as u32,
    DstAlpha = SDL_GPUBlendFactor::DST_ALPHA.0 as u32,
    OneMinusDstAlpha = SDL_GPUBlendFactor::ONE_MINUS_DST_ALPHA.0 as u32,
    ConstantColor = SDL_GPUBlendFactor::CONSTANT_COLOR.0 as u32,
    OneMinusConstantColor = SDL_GPUBlendFactor::ONE_MINUS_CONSTANT_COLOR.0 as u32,
    SrcAlphaSaturate = SDL_GPUBlendFactor::SRC_ALPHA_SATURATE.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BlendOp {
    #[default]
    Invalid = SDL_GPUBlendOp::INVALID.0 as u32,
    Add = SDL_GPUBlendOp::ADD.0 as u32,
    Subtract = SDL_GPUBlendOp::SUBTRACT.0 as u32,
    ReverseSubtract = SDL_GPUBlendOp::REVERSE_SUBTRACT.0 as u32,
    Min = SDL_GPUBlendOp::MIN.0 as u32,
    Max = SDL_GPUBlendOp::MAX.0 as u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorComponentFlags(pub sys::gpu::SDL_GPUColorComponentFlags);

impl ColorComponentFlags {
    pub const R: Self = Self(sys::gpu::SDL_GPU_COLORCOMPONENT_R);
    pub const G: Self = Self(sys::gpu::SDL_GPU_COLORCOMPONENT_G);
    pub const B: Self = Self(sys::gpu::SDL_GPU_COLORCOMPONENT_B);
    pub const A: Self = Self(sys::gpu::SDL_GPU_COLORCOMPONENT_A);
}
impl_with!(bitwise_and_or ColorComponentFlags u8);
