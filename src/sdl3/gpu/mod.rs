use crate::gpu::device::WeakDevice;

mod buffer;
pub use buffer::{
    Buffer, BufferBinding, BufferBuilder, BufferMemMap, BufferRegion, TransferBuffer,
    TransferBufferBuilder, TransferBufferLocation, VertexBufferDescription,
};

mod device;
pub use device::Device;

mod enums;
pub use enums::{
    BlendFactor, BlendOp, BufferUsageFlags, ColorComponentFlags, CompareOp, CullMode, FillMode,
    Filter, FrontFace, IndexElementSize, LoadOp, PrimitiveType, SampleCount, SamplerAddressMode,
    SamplerMipmapMode, ShaderFormat, ShaderStage, StencilOp, StoreOp, TextureFormat, TextureType,
    TextureUsage, TransferBufferUsage, VertexElementFormat, VertexInputRate,
};

mod pass;
pub use pass::{
    ColorTargetInfo, CommandBuffer, ComputePass, CopyPass, DepthStencilTargetInfo, RenderPass,
};

mod pipeline;
pub use pipeline::{
    ColorTargetBlendState, ColorTargetDescription, ComputePipeline, ComputePipelineBuilder,
    DepthStencilState, GraphicsPipeline, GraphicsPipelineBuilder, GraphicsPipelineTargetInfo,
    RasterizerState, StencilOpState, VertexAttribute, VertexInputState,
};

mod texture;
pub use texture::{
    Sampler, SamplerCreateInfo, Texture, TextureCreateInfo, TextureRegion, TextureSamplerBinding,
    TextureTransferInfo,
};

mod shader;
pub use shader::{Shader, ShaderBuilder};
