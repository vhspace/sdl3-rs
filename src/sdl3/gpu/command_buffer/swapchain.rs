use std::ops::Deref;

use crate::gpu::Texture;

#[doc(alias = "SDL_Texture")]
pub struct SwapchainTexture<'a> {
    pub(super) tex: &'a Texture,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl<'a> Deref for SwapchainTexture<'a> {
    type Target = Texture;

    fn deref(&self) -> &Self::Target {
        &self.tex
    }
}

impl<'a> SwapchainTexture<'a> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
