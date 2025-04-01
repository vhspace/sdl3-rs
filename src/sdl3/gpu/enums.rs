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
pub type TextureFormat = sys::gpu::SDL_GPUTextureFormat;

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
