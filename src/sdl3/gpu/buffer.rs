use crate::{
    get_error,
    gpu::{device::WeakDevice, BufferUsageFlags, Device, TransferBufferUsage, VertexInputRate},
    sys, Error,
};
use std::sync::Arc;
use sys::gpu::{
    SDL_CreateGPUBuffer, SDL_CreateGPUTransferBuffer, SDL_GPUBuffer, SDL_GPUBufferBinding,
    SDL_GPUBufferCreateInfo, SDL_GPUBufferRegion, SDL_GPUTransferBuffer,
    SDL_GPUTransferBufferCreateInfo, SDL_GPUTransferBufferLocation,
    SDL_GPUVertexBufferDescription, SDL_GPUVertexInputRate, SDL_MapGPUTransferBuffer,
    SDL_ReleaseGPUBuffer, SDL_ReleaseGPUTransferBuffer, SDL_UnmapGPUTransferBuffer,
};

#[repr(C)]
#[derive(Default)]
pub struct BufferBinding {
    pub(super) inner: SDL_GPUBufferBinding,
}
impl BufferBinding {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &Buffer) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default)]
pub struct TransferBufferLocation {
    pub(super) inner: SDL_GPUTransferBufferLocation,
}
impl TransferBufferLocation {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_transfer_buffer(mut self, transfer_buffer: &TransferBuffer) -> Self {
        self.inner.transfer_buffer = transfer_buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }
}

#[derive(Default)]
pub struct BufferRegion {
    pub(super) inner: SDL_GPUBufferRegion,
}
impl BufferRegion {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_buffer(mut self, buffer: &Buffer) -> Self {
        self.inner.buffer = buffer.raw();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.inner.offset = offset;
        self
    }

    pub fn with_size(mut self, size: u32) -> Self {
        self.inner.size = size;
        self
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct VertexBufferDescription {
    inner: SDL_GPUVertexBufferDescription,
}
impl VertexBufferDescription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_slot(mut self, value: u32) -> Self {
        self.inner.slot = value;
        self
    }

    pub fn with_pitch(mut self, value: u32) -> Self {
        self.inner.pitch = value;
        self
    }

    pub fn with_input_rate(mut self, value: VertexInputRate) -> Self {
        self.inner.input_rate = SDL_GPUVertexInputRate(value as i32);
        self
    }

    pub fn with_instance_step_rate(mut self, value: u32) -> Self {
        self.inner.instance_step_rate = value;
        self
    }
}

/// Manages the raw `SDL_GPUBuffer` pointer and releases it on drop
struct BufferContainer {
    raw: *mut SDL_GPUBuffer,
    device: WeakDevice,
}
impl Drop for BufferContainer {
    #[doc(alias = "SDL_ReleaseGPUBuffer")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                SDL_ReleaseGPUBuffer(device.raw(), self.raw);
            }
        }
    }
}

#[doc(alias = "SDL_GPUBuffer")]
#[derive(Clone)]
pub struct Buffer {
    inner: Arc<BufferContainer>,
    len: u32,
}
impl Buffer {
    /// Yields the raw SDL_GPUBuffer pointer.
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUBuffer {
        self.inner.raw
    }

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct BufferBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUBufferCreateInfo,
}
impl<'a> BufferBuilder<'a> {
    pub(super) fn new(device: &'a Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    pub fn with_usage(mut self, value: BufferUsageFlags) -> Self {
        self.inner.usage = value.0;
        self
    }

    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<Buffer, Error> {
        let raw_buffer = unsafe { SDL_CreateGPUBuffer(self.device.raw(), &self.inner) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(Buffer {
                len: self.inner.size,
                inner: Arc::new(BufferContainer {
                    raw: raw_buffer,
                    device: self.device.weak(),
                }),
            })
        }
    }
}

/// Mapped memory for a transfer buffer.
pub struct BufferMemMap<'a, T> {
    device: &'a Device,
    transfer_buffer: &'a TransferBuffer,
    mem: *mut T,
}

impl<'a, T> BufferMemMap<'a, T>
where
    T: Copy,
{
    /// Access the memory as a readonly slice.
    pub fn mem(&self) -> &[T] {
        let count = self.transfer_buffer.len() as usize / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts(self.mem, count) }
    }

    /// Access the memory as a mutable slice.
    pub fn mem_mut(&mut self) -> &mut [T] {
        let count = self.transfer_buffer.len() as usize / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts_mut(self.mem, count) }
    }

    #[doc(alias = "SDL_UnmapGPUTransferBuffer")]
    pub fn unmap(self) {
        unsafe { SDL_UnmapGPUTransferBuffer(self.device.raw(), self.transfer_buffer.raw()) };
    }
}

/// Manages the raw `SDL_GPUTransferBuffer` pointer and releases it on drop
struct TransferBufferContainer {
    raw: *mut SDL_GPUTransferBuffer,
    device: WeakDevice,
}
impl Drop for TransferBufferContainer {
    #[doc(alias = "SDL_ReleaseGPUTransferBuffer")]
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                SDL_ReleaseGPUTransferBuffer(device.raw(), self.raw);
            }
        }
    }
}

#[derive(Clone)]
pub struct TransferBuffer {
    inner: Arc<TransferBufferContainer>,
    len: u32,
}
impl TransferBuffer {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUTransferBuffer {
        self.inner.raw
    }

    #[doc(alias = "SDL_MapGPUTransferBuffer")]
    pub fn map<'a, T: Copy>(&'a self, device: &'a Device, cycle: bool) -> BufferMemMap<'a, T> {
        BufferMemMap {
            device,
            transfer_buffer: self,
            mem: unsafe { SDL_MapGPUTransferBuffer(device.raw(), self.raw(), cycle) } as *mut T,
        }
    }

    /// The length of this buffer in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct TransferBufferBuilder<'a> {
    device: &'a Device,
    inner: SDL_GPUTransferBufferCreateInfo,
}
impl<'a> TransferBufferBuilder<'a> {
    pub(super) fn new(device: &'a Device) -> Self {
        Self {
            device,
            inner: Default::default(),
        }
    }

    /// How the buffer will be used.
    pub fn with_usage(mut self, value: TransferBufferUsage) -> Self {
        self.inner.usage = value;
        self
    }

    /// Desired size of the buffer in bytes.
    pub fn with_size(mut self, value: u32) -> Self {
        self.inner.size = value;
        self
    }

    pub fn build(self) -> Result<TransferBuffer, Error> {
        let raw_buffer = unsafe { SDL_CreateGPUTransferBuffer(self.device.raw(), &self.inner) };
        if raw_buffer.is_null() {
            Err(get_error())
        } else {
            Ok(TransferBuffer {
                inner: Arc::new(TransferBufferContainer {
                    raw: raw_buffer,
                    device: self.device.weak(),
                }),
                len: self.inner.size,
            })
        }
    }
}
