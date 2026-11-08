#[allow(non_snake_case)]
pub mod kind {
    use crate::ttf;

    pub trait Engine {
        // You can't specialize Drop on specific types, so this associated function allows Drop to specialize for T: Engine and handle dropping every engine
        unsafe fn drop(engine: *mut ttf::sys::TTF_TextEngine);
    }
    macro_rules! engines {
        ($($name:ident$(; drop=$drop:path)?),*) => {
            $(
                pub enum $name {}
                impl Engine for $name {
                    #[allow(unused_variables)]
                    #[allow(unreachable_code)]
                    unsafe fn drop(engine: *mut ttf::sys::TTF_TextEngine) {
                        $(unsafe {
                            return $drop(engine);
                        })?
                        // You can't own an TextEngine<Unknown> because it is impossible to drop. OwnedTextEngine<Unknown> should be inconstructable.
                        unreachable!("An unknown text engine should never be dropped")
                    }
                }
            )*
        }
    }
    engines!(
        Unknown,
        Gpu; drop=ttf::sys::TTF_DestroyGPUTextEngine,
        Surface; drop=ttf::sys::TTF_DestroySurfaceTextEngine,
        Renderer; drop=ttf::sys::TTF_DestroyRendererTextEngine
    );
}
use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::{gpu::Device, ttf, Error};
use kind::*;

pub type GpuTextEngine<'a> = TextEngine<'a, Gpu>;
pub type SurfaceTextEngine<'a> = TextEngine<'a, Surface>;
pub type RendererTextEngine<'a> = TextEngine<'a, Renderer>;

#[repr(transparent)]
pub struct TextEngine<'a, E: Engine = Unknown>(ttf::sys::TTF_TextEngine, PhantomData<(E, &'a ())>);
impl<'a, E: Engine> TextEngine<'a, E> {
    // Make a TextEngine from a raw SDL3 TTF_TextEngine.
    // Safety: `engine` MUST be a valid text engine for the given engine type(`E`)
    pub unsafe fn from_raw(engine: *mut ttf::sys::TTF_TextEngine) -> *mut Self {
        engine.cast()
    }

    pub fn raw(&self) -> *const ttf::sys::TTF_TextEngine {
        (&raw const *self).cast()
    }
}

pub struct OwnedTextEngine<'a, E: Engine> {
    engine: NonNull<TextEngine<'a, E>>,
}
impl<'a, E: Engine> Deref for OwnedTextEngine<'a, E> {
    type Target = TextEngine<'a, E>;
    fn deref(&self) -> &Self::Target {
        // Safety: `self.engine` is valid for the lifetime of `self`
        unsafe { self.engine.as_ref() }
    }
}
impl<'a, E: Engine> Drop for OwnedTextEngine<'a, E> {
    fn drop(&mut self) {
        // Safety: `self.engine` is valid for the lifetime of `self`
        unsafe {
            E::drop(self.raw().cast_mut());
        }
    }
}

impl<'a> OwnedTextEngine<'a, Gpu> {
    #[doc(alias = "TTF_CreateGPUTextEngine")]
    pub fn new_gpu_engine(gpu: &'a Device) -> Result<Self, Error> {
        let engine = unsafe {
            NonNull::new(TextEngine::from_raw(ttf::sys::TTF_CreateGPUTextEngine(
                gpu.raw(),
            )))
        };

        match engine {
            None => Err(crate::get_error()),
            Some(engine) => Ok(Self { engine }),
        }
    }
}
impl<'a> OwnedTextEngine<'a, Surface> {
    #[doc(alias = "TTF_CreateSurfaceTextEngine")]
    pub fn new_surface_engine() -> Result<Self, Error> {
        let engine =
            unsafe { NonNull::new(TextEngine::from_raw(ttf::sys::TTF_CreateSurfaceTextEngine())) };

        match engine {
            None => Err(crate::get_error()),
            Some(engine) => Ok(Self { engine }),
        }
    }
}
impl<'a> OwnedTextEngine<'a, Renderer> {
    #[doc(alias = "TTF_CreateRendererTextEngine")]
    pub fn new_renderer_engine<T>(
        renderer: &'a crate::render::TextureCreator<T>,
    ) -> Result<Self, Error> {
        let engine = unsafe {
            NonNull::new(TextEngine::from_raw(
                ttf::sys::TTF_CreateRendererTextEngine(renderer.raw()),
            ))
        };

        match engine {
            None => Err(crate::get_error()),
            Some(engine) => Ok(Self { engine }),
        }
    }
}
