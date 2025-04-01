use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use sys::gpu::SDL_GPUTexture;

use super::Extern;


type Invariant<'a> = PhantomData<fn(&'a ()) -> &'a ()>;


#[doc(alias = "SDL_Texture")]
pub struct SwapchainTexture<'cmd_buf> {
    pub(super) raw: NonNull<Extern<SDL_GPUTexture>>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) _marker: Invariant<'cmd_buf>,
}

impl<'cmd_buf> Deref for SwapchainTexture<'cmd_buf> {
    type Target = Extern<SDL_GPUTexture>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

impl<'cmd_buf> SwapchainTexture<'cmd_buf> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
