use crate::{
    get_error,
    gpu::{Device, ShaderFormat, ShaderStage, WeakDevice},
    Error,
};
use std::{ffi::CStr, sync::Arc};
use sys::gpu::{SDL_GPUShader, SDL_GPUShaderCreateInfo};

/// Manages the raw `SDL_GPUShader` pointer and releases it on drop
struct ShaderContainer {
    raw: *mut SDL_GPUShader,
    device: WeakDevice,
}
impl Drop for ShaderContainer {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe { sys::gpu::SDL_ReleaseGPUShader(device.raw(), self.raw) }
        }
    }
}

#[derive(Clone)]
pub struct Shader {
    inner: Arc<ShaderContainer>,
}
impl Shader {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUShader {
        self.inner.raw
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUShaderCreateInfo,
}
impl<'a> ShaderBuilder<'a> {
    pub(super) fn new(device: &'a Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    pub fn with_samplers(mut self, value: u32) -> Self {
        self.inner.num_samplers = value;
        self
    }

    pub fn with_storage_buffers(mut self, value: u32) -> Self {
        self.inner.num_storage_buffers = value;
        self
    }

    pub fn with_storage_textures(mut self, value: u32) -> Self {
        self.inner.num_storage_textures = value;
        self
    }

    pub fn with_uniform_buffers(mut self, value: u32) -> Self {
        self.inner.num_uniform_buffers = value;
        self
    }

    pub fn with_code(mut self, fmt: ShaderFormat, code: &'a [u8], stage: ShaderStage) -> Self {
        self.inner.format = fmt.0;
        self.inner.code = code.as_ptr();
        self.inner.code_size = code.len() as usize;
        self.inner.stage = unsafe { std::mem::transmute(stage as u32) };
        self
    }
    pub fn with_entrypoint(mut self, entry_point: &'a CStr) -> Self {
        self.inner.entrypoint = entry_point.as_ptr();
        self
    }
    pub fn build(self) -> Result<Shader, Error> {
        let raw_shader = unsafe { sys::gpu::SDL_CreateGPUShader(self.device.raw(), &self.inner) };
        if !raw_shader.is_null() {
            Ok(Shader {
                inner: Arc::new(ShaderContainer {
                    raw: raw_shader,
                    device: self.device.weak(),
                }),
            })
        } else {
            Err(get_error())
        }
    }
}
