//! GPU-resources
//! 
//! 
mod builders;
use std::{marker::PhantomData, ptr::NonNull};

pub use builders::{
    ComputePipelineBuilder, GraphicsPipelineBuilder, ShaderBuilder, BufferBuilder, 
    TransferBufferBuilder,
};


mod device;
pub use device::OwnedDevice;
pub use device::Device;

use sys::gpu::{SDL_GPUBufferRegion, SDL_GPUDevice, SDL_GPUStorageBufferReadWriteBinding, SDL_GPUStorageTextureReadWriteBinding, SDL_GPUTransferBufferLocation, SDL_MapGPUTransferBuffer, SDL_UnmapGPUTransferBuffer};


use crate::Error;
use crate::{get_error, gpu::BufferRegion};

use super::util::Defer;
use super::Extern;
use super::{StorageBufferReadWriteBinding, StorageTextureReadWriteBinding, TextureSamplerBinding, TransferBufferLocation};


pub unsafe trait GpuCreate: GpuRelease {
    type CreateInfo;
    const CREATE: unsafe extern "C" fn(*mut SDL_GPUDevice, *const Self::CreateInfo) -> *mut Self::SDLType;
}

pub unsafe trait GpuRelease {
    type SDLType;
    const RELEASE: unsafe extern "C" fn(*mut SDL_GPUDevice, *mut Self::SDLType);

    // any additional state that `Owned<Self>` will keep
    type ExtraState;
}

pub struct Owned<'gpu, T: GpuRelease>
{
    raw: NonNull<T>,
    ctx: &'gpu Device,
    extra: T::ExtraState,
}

impl<'gpu, T: GpuCreate + GpuRelease> Owned<'gpu, T> {
    pub(crate) fn new(ctx: &'gpu Device, info: &T::CreateInfo, extra: T::ExtraState) -> Result<Self, Error> {
        unsafe {
            let raw: *mut T::SDLType = T::CREATE(ctx.ll(), info);
            let raw: *mut T = raw.cast();
            if let Some(raw) = NonNull::new(raw) {
                Ok(Owned {
                    raw,
                    ctx,
                    extra,
                })
            } else {
                Err(get_error())
            }
        }
    }
}

impl<'gpu, T: GpuRelease> ::core::ops::Deref for Owned<'gpu, T> {
    type Target = Extern<T::SDLType>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.cast().as_ref() }
    }
}

impl<'gpu, T: GpuRelease> Drop for Owned<'gpu, T> {
    fn drop(&mut self) {
        unsafe {
            T::RELEASE(self.ctx.ll(), self.raw.as_ptr().cast());
        }
    }
}

macro_rules! gpu_resource {
    ($rust_name:ident, $sdl_name:path, $info:path, $create:path, $release:path, $extra:ty) => {

        const _: () = assert!(size_of::<$sdl_name>() == 0);
        
        pub type $rust_name = Extern<$sdl_name>;
        unsafe impl GpuCreate for $rust_name {
            type CreateInfo = $info;
            
            const CREATE: unsafe extern "C" fn(*mut SDL_GPUDevice, *const Self::CreateInfo) -> *mut Self::SDLType
                = $create;
        }
        unsafe impl GpuRelease for $rust_name {
            type SDLType = $sdl_name;
            type ExtraState = $extra;
            
            const RELEASE: unsafe extern "C" fn(*mut SDL_GPUDevice, *mut Self::SDLType)
                = $release;
        }
    };
}

gpu_resource!(ComputePipeline,
    sys::gpu::SDL_GPUComputePipeline,
    sys::gpu::SDL_GPUComputePipelineCreateInfo,
    sys::gpu::SDL_CreateGPUComputePipeline,
    sys::gpu::SDL_ReleaseGPUComputePipeline,
    ()
);

gpu_resource!(GraphicsPipeline,
    sys::gpu::SDL_GPUGraphicsPipeline,
    sys::gpu::SDL_GPUGraphicsPipelineCreateInfo,
    sys::gpu::SDL_CreateGPUGraphicsPipeline,
    sys::gpu::SDL_ReleaseGPUGraphicsPipeline,
    ()
);


gpu_resource!(Sampler,
    sys::gpu::SDL_GPUSampler,
    sys::gpu::SDL_GPUSamplerCreateInfo,
    sys::gpu::SDL_CreateGPUSampler,
    sys::gpu::SDL_ReleaseGPUSampler,
    ()
);

gpu_resource!(Shader,
    sys::gpu::SDL_GPUShader,
    sys::gpu::SDL_GPUShaderCreateInfo,
    sys::gpu::SDL_CreateGPUShader,
    sys::gpu::SDL_ReleaseGPUShader,
    ()
);

gpu_resource!(Texture,
    sys::gpu::SDL_GPUTexture,
    sys::gpu::SDL_GPUTextureCreateInfo,
    sys::gpu::SDL_CreateGPUTexture,
    sys::gpu::SDL_ReleaseGPUTexture,
    (u32, u32)
);

gpu_resource!(TransferBuffer,
    sys::gpu::SDL_GPUTransferBuffer,
    sys::gpu::SDL_GPUTransferBufferCreateInfo,
    sys::gpu::SDL_CreateGPUTransferBuffer,
    sys::gpu::SDL_ReleaseGPUTransferBuffer,
    u32
);

gpu_resource!(Buffer,
    sys::gpu::SDL_GPUBuffer,
    sys::gpu::SDL_GPUBufferCreateInfo,
    sys::gpu::SDL_CreateGPUBuffer,
    sys::gpu::SDL_ReleaseGPUBuffer,
    u32
);



impl<'a> Owned<'a, Texture> {
    pub fn width(&self) -> u32 {
        self.extra.0
    }

    pub fn height(&self) -> u32 {
        self.extra.1
    }
}


impl<'gpu> Owned<'gpu, Buffer> {

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.extra
    }
}

impl Buffer {
    pub fn get(&self, range: std::ops::Range<u32>) -> BufferRegion<'_> {
        assert!(range.end >= range.start);
        BufferRegion {
            inner: SDL_GPUBufferRegion {
                buffer: self.ll(),
                offset: range.start,
                size: range.end - range.start,
                ..Default::default()
            },
            _marker: PhantomData,
        }
    }

    /// Create a read-write binding that cycles this buffer when bound
    pub fn cycled(&self) -> StorageBufferReadWriteBinding<'_> {
        let mut inner = SDL_GPUStorageBufferReadWriteBinding::default();
        inner.cycle = true;
        inner.buffer = self.ll();
        StorageBufferReadWriteBinding {
            inner,
            _marker: PhantomData,
        }
    }

    /// Create a read-write binding that refers to the current contents of the buffer
    pub fn preserved(&self) -> StorageBufferReadWriteBinding<'_> {
        let mut inner = SDL_GPUStorageBufferReadWriteBinding::default();
        inner.cycle = false;
        inner.buffer = self.ll();
        StorageBufferReadWriteBinding {
            inner,
            _marker: PhantomData,
        }
    }
}

impl Texture {
    /// Create a read-write binding that cycles this texture when bound
    pub fn cycled(&self) -> StorageTextureReadWriteBinding<'_> {
        let mut inner = SDL_GPUStorageTextureReadWriteBinding::default();
        inner.cycle = true;
        inner.texture = self.ll();
        StorageTextureReadWriteBinding {
            inner,
            _marker: PhantomData,
        }
    }

    /// Create a read-write binding that refers to the current contents of the texture
    pub fn preserved(&self) -> StorageTextureReadWriteBinding<'_> {
        let mut inner = SDL_GPUStorageTextureReadWriteBinding::default();
        inner.cycle = false;
        inner.texture = self.ll();
        StorageTextureReadWriteBinding {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn with_sampler<'a>(&'a self, sampler: &'a Sampler) -> TextureSamplerBinding<'a> {
        let mut binding = TextureSamplerBinding::default();
        binding.inner.texture = self.ll();
        binding.inner.sampler = sampler.ll();
        binding
    }
}


impl TransferBuffer {
    pub fn get<'a>(&'a self, from: std::ops::RangeFrom<u32>) -> TransferBufferLocation<'a> {
        TransferBufferLocation {
            inner: SDL_GPUTransferBufferLocation {
                transfer_buffer: self.ll(),
                offset: from.start,
                ..Default::default()
            },
            _marker: PhantomData,
        }
    }
}

impl<'gpu> Owned<'gpu, TransferBuffer> {
    #[doc(alias = "SDL_MapGPUTransferBuffer")]
    pub fn mapped_mut<R>(
        &mut self,
        cycle: bool,
        f: impl for<'a> FnOnce(&'a mut [u8]) -> R,
    ) -> Result<R, Error> {
        unsafe {
            let raw = SDL_MapGPUTransferBuffer(self.ctx.ll(), self.ll(), cycle);
            if raw.is_null() {
                return Err(get_error());
            }

            let bytes = std::slice::from_raw_parts_mut(raw as *mut u8, self.extra as usize);

            let _defer = Defer::new(||
                SDL_UnmapGPUTransferBuffer(self.ctx.ll(), self.ll())
            );
    
            Ok(f(bytes))
        }
    }

    pub fn len(&self) -> u32 {
        self.extra
    }
}
