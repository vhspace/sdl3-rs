//! GPU-resources
//! 
mod util {
    pub(super) use std::ptr::NonNull;
    pub(super) use std::ops::Deref;
    pub(super) use crate::Error;

    pub(super) use super::super::util::*;

    pub(super) use super::super::Extern;
    pub(super) use super::super::ExternDevice;

}

mod builders;
pub use builders::{
    ComputePipelineBuilder, GraphicsPipelineBuilder, ShaderBuilder, BufferBuilder, 
    TransferBufferBuilder,
};

mod buffer;
pub use buffer::Buffer;

mod shader;
pub use shader::Shader;

mod compute_pipeline;
pub use compute_pipeline::ComputePipeline;

mod graphics_pipeline;
pub use graphics_pipeline::GraphicsPipeline;

mod sampler;
pub use sampler::Sampler;

mod texture;
pub use texture::Texture;

mod transfer_buffer;
pub use transfer_buffer::TransferBuffer;

mod device;
pub use device::Device;
