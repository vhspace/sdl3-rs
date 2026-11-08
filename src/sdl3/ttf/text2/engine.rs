//! Thin wrappers around `TTF_TextEngine`, using typestates for different engine types
pub mod kind {
    //! zero-sized type states representing different types of text engine
    use crate::ttf;

    pub trait Engine {
        // You can't specialize Drop on specific types, so this associated function allows Drop to specialize for T: Engine and handle dropping every engine
        unsafe fn drop(engine: *mut ttf::sys::TTF_TextEngine);
    }
    pub trait OwnableEngine: Engine {

    }
    macro_rules! engines {
        (Unknown, $($name:ident; drop=$drop:path),*) => {
            // Done as an exception to the rest of the macro because Unknown can't be drop, but should still appear in `engines!()` just to be more explicit about it
            /// Typestate representing an unknown text engine
            pub enum Unknown {}
            impl Engine for Unknown {
                unsafe fn drop(_engine: *mut ttf::sys::TTF_TextEngine) {
                    // You can't own an TextEngine<Unknown> because it is impossible to drop. OwnedTextEngine<Unknown> should be inconstructable.
                    // Could be made unreachable_unchecked() in the future
                    unreachable!("An unknown text engine can never be dropped")
                }
            }

            $(
                

                #[doc = concat!("Typestate representing the \"", stringify!($name), "\" text engine type")]
                pub enum $name {}
                impl Engine for $name {
                    #[allow(unused_variables)] // Needed due to the unreachable!() invokation at the bottom
                    #[allow(unreachable_code)]
                    unsafe fn drop(engine: *mut ttf::sys::TTF_TextEngine) {
                        unsafe {
                            return $drop(engine);
                        }
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

/// Thin wrapper around SDL'3 `TTF_TextEngine`, carrying the type of the text engine.
/// Some operations are only valid for certain text engines(particularly drawing operations), so this leverages the type system for that.
#[repr(transparent)]
pub struct TextEngine<'a, E: Engine = Unknown>(ttf::sys::TTF_TextEngine, PhantomData<(E, &'a ())>);
impl<'a, E: Engine> TextEngine<'a, E> {
    // Make a TextEngine from a raw SDL3 TTF_TextEngine.
    // Safety: `engine` must be a valid text engine for the given engine type(`E`).
    pub unsafe fn from_raw(engine: *mut ttf::sys::TTF_TextEngine) -> *mut Self {
        engine.cast()
    }

    pub fn raw(&self) -> *const ttf::sys::TTF_TextEngine {
        (&raw const *self).cast()
    }
}

/// A smart pointer around a `TextEngine` representing an owned `TextEngine`
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

impl<'a, E: Engine> OwnedTextEngine<'a, E> {
    /// Get the inner `TextEngine` point, leaking the `TextEngine`
    pub fn into_raw(self) -> NonNull<TextEngine<'a, E>> {
        let engine = self.engine;
        std::mem::forget(self); // Prevent `engine` from getting free'd

        engine
    }
    /// Create an `OwnedTextEngine` directly from a pointer to a `TextEngine`.
    /// # Safety
    /// `engine` must be an exclusively owned `TextEngine`, created by SDL3.
    /// Note, it is __undefined behavior__ to construct `OwnedTextEngine<Unknown>` as `TextEngine<Unknown>` can not be `drop()`'d(every `TextEngine` has it's own destructor function and it is impossible to check which should be used).
    // In the future, a super trait like `OwnedableEngine: Engine` could be defined to make `OwnedTextEngine<Unknown>` inexpressible at the type-system level, but that is additional complexity. 
    pub unsafe fn from_raw(engine: NonNull<TextEngine<'a, E>>) -> Self {
        Self { engine }
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
