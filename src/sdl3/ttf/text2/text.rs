use std::{
    cell::Cell, ffi::CStr, fmt::Display, marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull,
};

use crate::{render::FPoint, Error};

use super::{
    super::{sys, Font},
    engine::kind::*,
    StrDestructure, TextEngine,
};

#[derive(Debug)]
pub enum TextModifyError {
    Sdl3(Error),
    InvalidUtf8Boundry,
    OutOfBounds,
}
impl From<Error> for TextModifyError {
    fn from(value: Error) -> Self {
        Self::Sdl3(value)
    }
}
impl Display for TextModifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sdl3(error) => error.fmt(f),
            Self::InvalidUtf8Boundry => write!(f, "Invalid index into string outside UTF8 boundry"),
            Self::OutOfBounds => write!(f, "Invalid index outside of string body")
        }
    }
}
impl std::error::Error for TextModifyError {}



#[repr(transparent)]
pub struct Text<'a, E: Engine = Unknown>(
    sys::TTF_Text,
    PhantomData<(&'a TextEngine<'a,E>, &'a Font<'a>)>,
);
impl<'a, E: Engine> Text<'a, E> {
    #[doc(alias = "TTF_InsertTextString")]
    pub fn insert(&mut self, index: usize, text: &str) -> Result<(), TextModifyError> {
        let text = text.destructure();
        let current = self.str();

        // We already have to check for chararacter boundries, so also check length to provide a more rich error(Error is extremely opaque)
        if index > current.len() {
            return Err(TextModifyError::OutOfBounds);
        }
        if !current.is_char_boundary(index) {
            return Err(TextModifyError::InvalidUtf8Boundry);
        }

        // Safety: SDL3 performs it's own bounds checking, and we just validated utf8 boundries.  SDL3 makes it's own copy of `text`.
        unsafe {
            match sys::TTF_InsertTextString(self.raw(), index as i32, text.ptr, text.len) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }
    #[doc(alias = "TTF_AppendTextString")]
    pub fn append(&mut self, text: &str) -> Result<(), TextModifyError> {
        let text = text.destructure();

        // Safety: Appending a string requires no validation.  SDL3 makes it's own copy of `text`.
        unsafe {
            match sys::TTF_AppendTextString(self.raw(), text.ptr, text.len) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }
    #[doc(alias = "TTF_DeleteTextString")]
    pub fn delete(&mut self, index: usize, size: usize) -> Result<(), TextModifyError> {
        let current = self.str();

        // We already have to check for chararacter boundries, so also check length to provide a more rich error(Error is extremely opaque)
        if index > current.len() || index + size > current.len() {
            return Err(TextModifyError::OutOfBounds);
        }
        if !current.is_char_boundary(index) || !current.is_char_boundary(index + size) {
            return Err(TextModifyError::InvalidUtf8Boundry);
        }

        // Safety: SDL3 performs it's own bounds checking, and we just validated utf8 boundries.
        unsafe {
            match sys::TTF_DeleteTextString(self.raw(), index as i32, size as i32) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }
    #[doc(alias = "TTF_SetTextString")]
    pub fn set(&mut self, text: &str) -> Result<(), Error> {
        let text = text.destructure();

        // Setting a string does not require any validation. SDL3 makes it's own copy of `text`.
        unsafe {
            match sys::TTF_SetTextString(self.raw(), text.ptr, text.len) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }

    #[doc(alias = "TTF_GetTextEngine")]
    pub fn get_engine(&self) -> &'a TextEngine<'a, E> {
        // Safety: An SDL3 `TTF_Text` object must always have a vlid `TTF_engine`
        unsafe { TextEngine::from_raw(sys::TTF_GetTextEngine(self.raw())).as_ref_unchecked() }
    }

    pub fn str(&self) -> &str {
        // Safety: TTF_Text object's strings are always null terminated strings.
        unsafe {
            let str: &CStr = CStr::from_ptr(self.0.text);
            let str = str.to_bytes();

            str::from_utf8_unchecked(str)
        }
    }

    pub fn upcast(&self) -> &Text<'a> {
        // Safety: Any valid Text<E>, for some engine E, is a supertype of Text<Unknown>
        unsafe { std::mem::transmute(self) }
    }
    pub fn upcast_mut(&mut self) -> &mut Text<'a> {
        // Safety: Any valid Text<E>, for some engine E, is a supertype of Text<Unknown>
        unsafe { std::mem::transmute(self) }
    }

    pub fn raw(&self) -> *mut sys::TTF_Text {
        (self as *const Self).cast_mut().cast()
    }
}
impl<'a> Text<'a, Unknown> {
    // Downcast a Text<> object to a specific Text<Engine> type based on the TextEngine it was created with
    // Only implemented for Text<'a, Unknown> because it is meaningless to downcast any other Engine, as renderer types are mutually exclusive.
    pub fn downcast<E: Engine>(&self, engine: &TextEngine<E>) -> Option<&Text<'a, E>> {
        match (&raw const *engine).addr() == self.get_engine().raw().addr() {
            // Safety: If this TTF_Text object has the same TTF_TextEngine as the given engine, it must have been created with that engine.
            true => Some(unsafe { std::mem::transmute(self) }),
            false => None,
        }
    }
    // Mut form of `downcast()`
    pub fn downcast_mut<E: Engine>(&mut self, engine: &TextEngine<E>) -> Option<&mut Text<'a, E>> {
        match (&raw const *engine).addr() == self.get_engine().raw().addr() {
            // Safety: If this TTF_Text object has the same TTF_TextEngine as the given engine, it must have been created with that engine.
            true => Some(unsafe { std::mem::transmute(self) }),
            false => None,
        }
    }
}

impl<'a, E: Engine> Drop for Text<'a, E> {
    fn drop(&mut self) {
        // Safety: `self.raw()` is valid for the lifetime of `self`
        unsafe {
            sys::TTF_DestroyText(self.raw());
        }
    }
}

impl<'a> Text<'a, Gpu> {
    #[doc(alias = "TTF_GetGPUTextDrawData")]
    pub fn atlas_draw_sequence(&self) -> Result<&GpuAtlasDrawSequence, Error> {
        // Safety: As GpuAtlasDrawSequence is a transparet wrapper, it is safe to cast TTF_GPUAtlasDrawSequence to it
        unsafe {
            match NonNull::new(sys::TTF_GetGPUTextDrawData(self.raw())) {
                Some(ptr) => Ok(ptr.cast().as_ref()),
                None => Err(crate::get_error()),
            }
        }
    }
}
impl<'a> Text<'a, Surface> {
    #[doc(alias = "TTF_DrawSurfaceText")]
    pub fn draw_surface(
        &self,
        x: u32,
        y: u32,
        surface: &crate::surface::Surface,
    ) -> Result<(), Error> {
        let success =
            unsafe { sys::TTF_DrawSurfaceText(self.raw(), x as i32, y as i32, surface.raw()) };

        match success {
            false => Err(crate::get_error()),
            true => Ok(()),
        }
    }
}
impl<'a> Text<'a, Renderer> {
    #[doc(alias = "TTF_DrawRendererText")]
    pub fn draw_renderer(&self, x: f32, y: f32) -> Result<(), Error> {
        let success = unsafe { sys::TTF_DrawRendererText(self.raw(), x, y) };

        match success {
            false => Err(crate::get_error()),
            true => Ok(()),
        }
    }
}

pub struct OwnedText<'a, E: Engine> {
    text: NonNull<Text<'a, E>>,
    _type: PhantomData<E>,
}
impl<'a, E: Engine> Deref for OwnedText<'a, E> {
    type Target = Text<'a, E>;
    fn deref(&self) -> &Self::Target {
        // `self.text` is valid for the lifetime of `self`
        unsafe { self.text.as_ref() }
    }
}
impl<'a, E: Engine> DerefMut for OwnedText<'a, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // `self.text` is valid and exclusively owned for the lifetime of `self`
        unsafe { self.text.as_mut() }
    }
}
impl<'a, E: Engine> Drop for OwnedText<'a, E> {
    fn drop(&mut self) {
        // `self.text` is valid and exclusively owned for the lifetime of `self`
        unsafe {
            self.text.drop_in_place();
        }
    }
}

impl<'a, E: Engine> OwnedText<'a, E> {
    #[doc(alias = "TTF_CreateText")]
    pub fn new(engine: &'a TextEngine<E>, font: &'a Font, text: &str) -> Result<Self, Error> {
        let text = text.destructure();
        let text = unsafe {
            NonNull::new(
                sys::TTF_CreateText(engine.raw().cast_mut(), font.raw(), text.ptr, text.len).cast(),
            )
        };

        match text {
            Some(text) => Ok(Self {
                text,
                _type: PhantomData,
            }),
            None => Err(crate::get_error()),
        }
    }

    // Create `OwnedText` from a raw `Text` pointer. The pointer must be exclusively owned.
    pub unsafe fn from_raw(engine: NonNull<Text<E>>) -> Self {
        Self {
            text: engine.cast(),
            _type: PhantomData,
        }
    }
}

pub struct RcText<'a, E: Engine> {
    text: NonNull<Text<'a, E>>,
    _type: PhantomData<E>,
}
impl<'a, E: Engine> RcText<'a, E> {
    pub fn new(mut owned: OwnedText<'a, E>) -> Self {
        owned.0.refcount = 1;
        let text = owned.text;
        std::mem::forget(owned); // Prevent `text` getting Drop()'d

        Self {
            text,
            _type: PhantomData,
        }
    }

    pub fn try_own(self) -> Result<OwnedText<'a, E>, Self> {
        // Safety: Checking for a reference_count of 0 ensures exclusivity
        match self.reference_count().get() {
            1 => Ok(unsafe { OwnedText::from_raw(self.into_raw()) }),
            _ => Err(self),
        }
    }
    pub fn try_mut(&mut self) -> Option<&mut Text<'a, E>> {
        match self.reference_count().get() {
            // Safety: Checking for a reference_count of 0 ensures exclusivity
            1 => Some(unsafe { self.text.as_ptr().as_mut_unchecked() }),
            _ => None,
        }
    }

    unsafe fn into_raw(self) -> NonNull<Text<'a, E>> {
        let ptr = self.text;
        std::mem::forget(self);

        ptr
    }

    fn reference_count(&self) -> &Cell<u32> {
        // Safety: SDL3 itself does not make use of `refcount`, leaving it avalible as an "application reference count"
        // Additionally, `refcount` is accessed in no other way than through this function, and, by extension, through a `&Cell<u32>`
        unsafe {
            Cell::from_mut(
                (&raw mut (*self.text.as_ptr()).0.refcount as *mut u32).as_mut_unchecked(),
            )
        }
    }
}
impl<'a, E: Engine> Clone for RcText<'a, E> {
    fn clone(&self) -> Self {
        // checked_add(1) to protect against overflow, should somemone make(or mem::forget()) u32::MAX many RcText<>'s
        self.reference_count()
            .update(|count| match count.checked_add(1) {
                Some(count) => count,
                None => {
                    // Match behavior of standard Rust Rc<T>. To qoute Rc's source,
                    // > We abort because this is such a degenerate scenario that we don't
                    // > care about what happens -- no real program should ever experience this.
                    eprintln!("Overflow over reference-counted text(RcText), aborting.");
                    std::process::abort()
                }
            });

        Self {
            text: self.text,
            _type: PhantomData,
        }
    }
}
impl<'a, E: Engine> Deref for RcText<'a, E> {
    type Target = Text<'a, E>;
    fn deref(&self) -> &Self::Target {
        // `text` is always valid for the lifetime of RcText
        unsafe { self.text.as_ref() }
    }
}
impl<'a, E: Engine> Drop for RcText<'a, E> {
    fn drop(&mut self) {
        self.reference_count().update(|count| count - 1);

        if self.reference_count().get() == 0 {
            // Safety: Literally just checked if ref count is zero now.
            unsafe {
                self.text.drop_in_place();
            }
        }
    }
}

// This type soley exists to allow checking if `atlas_texture` is None without having to make a sampler binding.
// Can't convert the SDL_Texture to a Texture<> either, because that requires the texture's size, which we don't have,
// so the only thing you can do with the texture is bind it
pub struct GpuAtlasDrawSequenceTexture<'a>(
    NonNull<crate::sys::gpu::SDL_GPUTexture>,
    PhantomData<&'a crate::sys::gpu::SDL_GPUTexture>,
);
impl<'a> GpuAtlasDrawSequenceTexture<'a> {
    pub fn binding<'b>(
        self,
        sampler: &'b crate::gpu::Sampler,
    ) -> crate::gpu::TextureSamplerBinding<'b>
    where
        'a: 'b,
    {
        // Safety: The SDL_GPUTexture must always be valid as an invariant for this type
        unsafe {
            crate::gpu::TextureSamplerBinding::new()
                .with_raw_texture(self.0.as_ptr())
                .with_sampler(sampler)
        }
    }
}

#[repr(transparent)]
pub struct GpuAtlasDrawSequence(sys::TTF_GPUAtlasDrawSequence);
impl GpuAtlasDrawSequence {
    pub fn atlas_texture(&self) -> Option<GpuAtlasDrawSequenceTexture<'_>> {
        match NonNull::new(self.0.atlas_texture) {
            None => None,
            Some(texture) => Some(GpuAtlasDrawSequenceTexture(texture, PhantomData)),
        }
    }

    pub fn vertices(&self) -> &[FPoint] {
        // Safety: `xy` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            std::slice::from_raw_parts(self.0.xy as *const FPoint, self.0.num_vertices as usize)
        }
    }
    pub fn texture_coordinates(&self) -> &[FPoint] {
        // Safety: `uv` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            std::slice::from_raw_parts(self.0.uv as *const FPoint, self.0.num_vertices as usize)
        }
    }

    pub fn indices(&self) -> &[u32] {
        // Safety: `indicies` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            // SDL's type is `int` for indicies, for some reason, even though they can't be negative by definition(A negative vertex index means nothing). So transmute them into a more conventionally u32.
            std::mem::transmute::<&[i32], &[u32]>(std::slice::from_raw_parts(
                self.0.indices,
                self.0.num_indices as usize,
            ))
        }
    }

    pub fn image_type(&self) -> &sys::TTF_ImageType {
        &self.0.image_type
    }

    pub fn next<'a>(&'a self) -> Option<&'a Self> {
        // Safety: This is just traversing a linked list. Thus, the life time of the next element will be the same as the lifetime of the current one.
        unsafe {
            self.0
                .next
                .as_ref()
                .map(|ptr| std::mem::transmute::<_, &'a Self>(ptr))
        }
    }
}
impl<'a> IntoIterator for &'a GpuAtlasDrawSequence {
    type IntoIter = GpuAtlasDrawSequenceIter<'a>;
    type Item = &'a GpuAtlasDrawSequence;
    fn into_iter(self) -> Self::IntoIter {
        GpuAtlasDrawSequenceIter {
            current: Some(self),
        }
    }
}

pub struct GpuAtlasDrawSequenceIter<'a> {
    current: Option<&'a GpuAtlasDrawSequence>,
}
impl<'a> Iterator for GpuAtlasDrawSequenceIter<'a> {
    type Item = &'a GpuAtlasDrawSequence;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.next();

        Some(current)
    }
}
