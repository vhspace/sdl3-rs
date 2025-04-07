use sys::gpu::SDL_GPUFence;

use crate::gpu::{resource::GpuRelease, Extern};


pub type Fence = Extern<SDL_GPUFence>;

unsafe impl GpuRelease for Fence {
    type SDLType = SDL_GPUFence;

    const RELEASE: unsafe extern "C" fn(*mut sys::gpu::SDL_GPUDevice, *mut Self::SDLType)
        = sys::gpu::SDL_ReleaseGPUFence;

    type ExtraState = ();
}