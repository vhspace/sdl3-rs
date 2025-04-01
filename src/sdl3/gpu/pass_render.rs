
use std::{ops::Deref, ptr::NonNull};

use crate::gpu::{
    BufferBinding, GraphicsPipeline, IndexElementSize,
    TextureSamplerBinding,
};
use sys::gpu::{
    SDL_BindGPUFragmentSamplers, SDL_BindGPUIndexBuffer, SDL_BindGPUVertexBuffers, SDL_DrawGPUIndexedPrimitives, SDL_GPUBuffer, SDL_GPUBufferBinding, SDL_GPURenderPass, SDL_GPUTexture, SDL_GPUTextureSamplerBinding, SDL_GPUViewport, SDL_SetGPUViewport
};

use super::Extern;


pub struct RenderPass {
    pub(super) raw: NonNull<Extern<SDL_GPURenderPass>>,
}
impl<'gpu> Deref for RenderPass {
    type Target = Extern<SDL_GPURenderPass>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}
impl RenderPass {
    #[doc(alias = "SDL_SetGPUViewport")]
    pub fn set_viewport(&mut self, viewport: &SDL_GPUViewport) {
        unsafe { SDL_SetGPUViewport(self.raw(), viewport) }
    }

    #[doc(alias = "SDL_BindGPUGraphicsPipeline")]
    pub fn bind_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
        unsafe { sys::gpu::SDL_BindGPUGraphicsPipeline(self.raw(), pipeline.raw()) }
    }

    #[doc(alias = "SDL_BindGPUVertexBuffers")]
    pub fn bind_vertex_buffers(&self, first_slot: u32, bindings: &[BufferBinding]) {
        unsafe {
            SDL_BindGPUVertexBuffers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *mut SDL_GPUBufferBinding,
                bindings.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUVertexStorageBuffers")]
    pub fn bind_vertex_storage_buffers(&self, first_slot: u32, storage_buffers: &[&Extern<SDL_GPUBuffer>]) {
        unsafe {
            sys::gpu::SDL_BindGPUVertexStorageBuffers(
                self.raw(),
                first_slot,
                storage_buffers.as_ptr().cast(),
                storage_buffers.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUVertexStorageTextures")]
    pub fn bind_vertex_storage_textures(&self, first_slot: u32, storage_textures: &[&Extern<SDL_GPUTexture>]) {
        unsafe {
            sys::gpu::SDL_BindGPUVertexStorageTextures(
                self.raw(),
                first_slot,
                storage_textures.as_ptr().cast(),
                storage_textures.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUIndexBuffer")]
    pub fn bind_index_buffer(&self, binding: &BufferBinding, index_element_size: IndexElementSize) {
        unsafe {
            SDL_BindGPUIndexBuffer(
                self.raw(),
                &binding.inner,
                index_element_size,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUFragmentSamplers")]
    pub fn bind_fragment_samplers(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUFragmentSamplers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *const SDL_GPUTextureSamplerBinding,
                bindings.len() as u32,
            );
        }
    }

    #[doc(alias = "SDL_BindGPUFragmentStorageBuffers")]
    pub fn bind_fragment_storage_buffers(&self, first_slot: u32, storage_buffers: &[&Extern<SDL_GPUBuffer>]) {
        unsafe {
            sys::gpu::SDL_BindGPUFragmentStorageBuffers(
                self.raw(),
                first_slot,
                storage_buffers.as_ptr().cast(),
                storage_buffers.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUFragmentStorageTextures")]
    pub fn bind_fragment_storage_textures(&self, first_slot: u32, storage_textures: &[&Extern<SDL_GPUTexture>]) {
        unsafe {
            sys::gpu::SDL_BindGPUFragmentStorageTextures(
                self.raw(),
                first_slot,
                storage_textures.as_ptr().cast(),
                storage_textures.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_DrawGPUIndexedPrimitives")]
    pub fn draw_indexed_primitives(
        &self,
        num_indices: u32,
        num_instances: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitives(
                self.raw(),
                num_indices,
                num_instances,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    #[doc(alias = "SDL_DrawGPUPrimitives")]
    pub fn draw_primitives(
        &self,
        num_vertices: usize,
        num_instances: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        unsafe {
            sys::gpu::SDL_DrawGPUPrimitives(
                self.raw(),
                num_vertices as u32,
                num_instances as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }
}
