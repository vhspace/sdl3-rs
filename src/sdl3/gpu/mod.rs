use crate::gpu::device::WeakDevice;

mod buffer;
pub use buffer::{
    Buffer, BufferBinding, BufferBuilder, BufferMemMap, BufferRegion, TransferBuffer,
    TransferBufferBuilder, TransferBufferLocation, VertexBufferDescription,
};

mod device;
pub use device::{Device, Viewport};

mod enums;
pub use enums::{
    BlendFactor, BlendOp, BufferUsageFlags, ColorComponentFlags, CompareOp, CullMode, FillMode,
    Filter, FrontFace, IndexElementSize, LoadOp, PresentMode, PrimitiveType, SampleCount,
    SamplerAddressMode, SamplerMipmapMode, ShaderFormat, ShaderStage, StencilOp, StoreOp,
    SwapchainComposition, TextureFormat, TextureType, TextureUsage, TransferBufferUsage,
    VertexElementFormat, VertexInputRate,
};

mod pass;
pub use pass::{
    BlitInfo, ColorTargetInfo, CommandBuffer, ComputePass, CopyPass, DepthStencilTargetInfo, Fence,
    RenderPass,
};

mod pipeline;
pub use pipeline::{
    ColorTargetBlendState, ColorTargetDescription, ComputePipeline, ComputePipelineBuilder,
    DepthStencilState, GraphicsPipeline, GraphicsPipelineBuilder, GraphicsPipelineTargetInfo,
    RasterizerState, StencilOpState, StorageBufferReadWriteBinding, StorageTextureReadWriteBinding,
    VertexAttribute, VertexInputState,
};

mod texture;
pub use texture::{
    Sampler, SamplerCreateInfo, Texture, TextureCreateInfo, TextureRegion, TextureSamplerBinding,
    TextureTransferInfo,
};

mod shader;
pub use shader::{Shader, ShaderBuilder};
