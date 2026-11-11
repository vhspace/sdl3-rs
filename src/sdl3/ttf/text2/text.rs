//! Thin wrappers around `TTF_Text` for working with different types of text and engines
use std::{
    cell::Cell, ffi::CStr, fmt::Display, marker::{PhantomData, PhantomPinned}, mem::MaybeUninit, num::NonZeroU32, ops::{Deref, DerefMut, Index}, ptr::NonNull,
};

use sdl3_ttf_sys::ttf::{self, TTF_SubString};

use crate::{Error, IntegerOrSdlError, common, get_error, pixels::{Color, FColor}, rect::Point, render::FPoint};

use super::{
    super::{sys, Font},
    engine::kind::*,
    StrDestructure, TextEngine,
};

/// Wrapper for TTF_Direction, represents directions that text may be renderered
#[doc(alias = "TTF_Direction")]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Invalid = sys::TTF_DIRECTION_INVALID.0,
    LeftToRight = sys::TTF_DIRECTION_LTR.0,
    RightToLeft = sys::TTF_DIRECTION_RTL.0,
    TopToBottom = sys::TTF_DIRECTION_TTB.0,
    BottomToTop = sys::TTF_DIRECTION_BTT.0
}
impl From<sys::TTF_Direction>  for Direction {
    // In theory, for returns directly from SDL functions, it might be faster to transmute(), but I doubt that is needed and is more complexity
    fn from(value: sys::TTF_Direction) -> Self {
        match value {
            sys::TTF_DIRECTION_LTR => Self::LeftToRight,
            sys::TTF_DIRECTION_RTL => Self::RightToLeft,
            sys::TTF_DIRECTION_TTB => Self::TopToBottom,
            sys::TTF_DIRECTION_BTT => Self::BottomToTop,
            _ => Self::Invalid
        }
    }
}
impl From<Direction> for sys::TTF_Direction {
    fn from(value: Direction) -> sys::TTF_Direction {
        sys::TTF_Direction(value as u32)
    }
}

/// Wrapper around `TTF_ImageType`
/// 
/// Values mean as follows:  
/// Self::Alpha => The texture atlas's color channels are white and carries only alpha data  
/// Self::Color => The texture atlas's color channels contain image data  
/// Self::SignedDistance => The texture atlas's alpha channel has signed distance field information  
#[repr(i32)]
pub enum ImageType {
    Invalid = sys::TTF_IMAGE_INVALID.0,
    Alpha = sys::TTF_IMAGE_ALPHA.0,
    Color = sys::TTF_IMAGE_COLOR.0,
    SignedDistance = sys::TTF_IMAGE_SDF.0
}
impl From<sys::TTF_ImageType> for ImageType {
    fn from(value: sys::TTF_ImageType) -> Self {
        match value {
            sys::TTF_IMAGE_ALPHA => Self::Alpha,
            sys::TTF_IMAGE_COLOR => Self::Color,
            sys::TTF_IMAGE_SDF => Self::SignedDistance,
            _ => Self::Invalid
        }
    }
}
impl From<ImageType> for sys::TTF_ImageType {
    fn from(value: ImageType) -> Self {
        sys::TTF_ImageType::new(value as i32)
    }
}

pub use super::scripts::Script;

/// Wrapper for the return type of `TTF_GetTextSubStringsForRange`.
/// The lifetime here is cursed and I don't think can be expressed conventionally because `substrings` is an array of `*const TTF_SubString` of the same lifetime as `substrings` itself.
/// IE: &'a [&'a TTF_SubString], only 'a is inexpressible because both the slice and the elements are owned in the same allocation, so it has to borrowed.
pub struct SubstringArray {
    substrings: NonNull<*mut TTF_SubString>,
    length: usize
}
impl SubstringArray {
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn elements<'a>(&'a self) -> &'a [&'a TTF_SubString] {
        // Every part of the `substring` is a valid pointer to a `TTF_SubString` with the same lifetime as `substring` itself
        unsafe {
            std::slice::from_raw_parts(self.substrings.cast::<&'a TTF_SubString>().as_ptr(), self.len())
        }
    } 
}
impl Index<usize> for SubstringArray {
    type Output = TTF_SubString;
    fn index(&self, index: usize) -> &Self::Output {
        self.elements()[index]
    }   
}
impl Drop for SubstringArray {
    fn drop(&mut self) {
        // Safety: `substring` will never be used after `drop()`
        unsafe {
            crate::sys::stdinc::SDL_free(self.substrings.as_ptr().cast())
        }
    }
}

/// Error from accessing an out of bounds part of text
#[derive(Debug)]
pub enum TextAccessError {
    Sdl3(Error),
    OutOfBounds
}
impl TextAccessError {
    fn cast_integer<T: TryInto<i32>>(integer: T) -> Result<i32, TextAccessError> {
        integer.try_into().map_err(|_| Self::OutOfBounds)
    }
}
impl From<Error> for TextAccessError {
    fn from(value: Error) -> Self {
        Self::Sdl3(value)
    }
}
impl Display for TextAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sdl3(error) => error.fmt(f),
            Self::OutOfBounds => write!(f, "Invalid index into string outside of string body")
        }
    }
}

/// An error from operations modifying the text of a `Text` object 
#[derive(Debug)]
pub enum TextModifyError {
    Sdl3(Error),
    InvalidUtf8Boundry,
    OutOfBounds,
}
impl TextModifyError {
    fn cast_integer<T: TryInto<i32>>(integer: T) -> Result<i32, TextModifyError> {
        integer.try_into().map_err(|_| TextModifyError::OutOfBounds)
    }
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

/// Wrapper for `TTF_Text`, including the type of the `TextEngine` used to create it
#[doc(alias = "TTF_GPUAtlasDrawSequence")]
#[repr(transparent)]
pub struct Text<'a, E: Engine = Unknown> {
    _opaque: [u8; 0],
    _phantom: PhantomData<(&'a TextEngine<'a,E>, &'a Font<'a>, *mut u8, PhantomPinned)>,
}
impl<'a, E: Engine> Text<'a, E> {
    /// Insert the given `text` at the given byte `index`.
    /// 
    /// # Errors
    /// If the underlying SDL3 call errors or if `index` is not a UTF8 boundry or is out of range of the text.
    #[doc(alias = "TTF_InsertTextString")]
    pub fn insert(&mut self, index: usize, text: &str) -> Result<(), TextModifyError> {
        let text = text.destructure();
        let current = self.str();

        // We already have to check for chararacter boundries, so also check length to provide a more rich error(SDL has it's own bounds checking, but Error is extremely opaque)
        if index > current.len() {
            return Err(TextModifyError::OutOfBounds);
        }
        if !current.is_char_boundary(index) {
            return Err(TextModifyError::InvalidUtf8Boundry);
        }

        // Safety: SDL3 performs it's own bounds checking, and we just validated utf8 boundries.  SDL3 makes it's own copy of `text`.
        unsafe {
            match sys::TTF_InsertTextString(self.raw(), TextModifyError::cast_integer(index)?, text.ptr, text.len) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }
    /// Append the given `text`.
    /// 
    /// # Errors
    /// If the underlying SDL3 call errors.
    #[doc(alias = "TTF_AppendTextString")]
    pub fn append(&mut self, text: &str) -> Result<(), Error> {
        let text = text.destructure();

        // Safety: Appending a string requires no validation.  SDL3 makes it's own copy of `text`.
        unsafe {
            match sys::TTF_AppendTextString(self.raw(), text.ptr, text.len) {
                false => Err(crate::get_error()),
                true => Ok(()),
            }
        }
    }
    /// Deletes the text at the given byte `index` for the given length `size`.
    /// 
    /// # Errors
    /// If the underlying SDL3 call errors or if either `index` or `index + size` is out of range or does not fall on a utf8 boundr.
    #[doc(alias = "TTF_DeleteTextString")]
    pub fn delete(&mut self, index: usize, size: usize) -> Result<(), TextModifyError> {
        let current = self.str();

        // We already have to check for chararacter boundries, so also check length to provide a more rich error(SDL has it's own bounds checking, but Error is extremely opaque)
        let end = index.checked_add(size).ok_or(TextModifyError::OutOfBounds)?;
        if index > current.len() ||  end > current.len() {
            return Err(TextModifyError::OutOfBounds);
        }
        if !current.is_char_boundary(index) || !current.is_char_boundary(end) {
            return Err(TextModifyError::InvalidUtf8Boundry);
        }

        // Safety: SDL3 performs it's own bounds checking, and we just validated utf8 boundries.
        unsafe {
            match sys::TTF_DeleteTextString(self.raw(), TextModifyError::cast_integer(index)?, TextModifyError::cast_integer(size)?) {
                false => Err(crate::get_error().into()),
                true => Ok(()),
            }
        }
    }
    /// Set the text content to the given `text`.
    /// 
    /// # Errors
    /// If the underlying SDL3 call errors.
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
    #[doc(alias = "TTF_GetTextProperties")]
    //pub fn get_properties(&self) {} // Need to figure out how properties work
    // Can't return a normal `&Font` 'cause  `Font` is not a direct wrapper of `TTF_Font` and can't be reconstructed
    #[doc(alias = "TTF_GetTextFont")]
    pub fn get_font(&self) -> Result<NonNull<sys::TTF_Font>, Error> {
        let font = unsafe { 
            NonNull::new(sys::TTF_GetTextFont(self.raw()))
        };

        match font {
            Some(font) => Ok(font),
            None => Err(get_error())
        }
    }
    #[doc(alias = "TTF_GetTextDirection")]
    pub fn get_direction(&self) -> Direction {
        unsafe {
            sys::TTF_GetTextDirection(self.raw()).into()
        }
    }
    #[doc(alias = "TTF_GetTextScript")]
    pub fn get_script(&self) -> Option<Script> {
        unsafe {
            Script::from_tag(sys::TTF_GetTextScript(self.raw())) 
        }
    }
    #[doc(alias = "TTF_GetTextColor")]
    pub fn get_color(&self) -> Result<Color, Error> {
        let mut data = [MaybeUninit::uninit(); 4];
        let success = unsafe {
            sys::TTF_GetTextColor(
                self.raw(), 
                data[0].as_mut_ptr(), 
                data[1].as_mut_ptr(), 
                data[2].as_mut_ptr(), 
                data[3].as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextColor` succeded, initalizing `data`, and `Color` is repr(C), allowing it to be directly transmuted
            true => unsafe {
                Ok(std::mem::transmute::<_, Color>(data))
            }
        }
    }
    #[doc(alias = "TTF_GetTextColorFloat")]
    pub fn get_color_float(&self) -> Result<FColor, Error> {
        let mut data = [MaybeUninit::uninit(); 4];
        let success = unsafe {
            sys::TTF_GetTextColorFloat(
                self.raw(), 
                data[0].as_mut_ptr(),
                data[1].as_mut_ptr(), 
                data[2].as_mut_ptr(), 
                data[3].as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextColorFloat` succeded, initalizing `data`, and `FColor` is repr(C), allowing it to be directly transmuted
            true => unsafe {
                Ok(std::mem::transmute::<_, FColor>(data))
            }
        }
    }
    #[doc(alias = "TTF_GetTextPosition")]
    pub fn get_position(&self) -> Result<Point, Error> {
        let mut data = [MaybeUninit::uninit(); 2];
        let success = unsafe {
            ttf::TTF_GetTextPosition(
                self.raw(),
                data[0].as_mut_ptr(),
                data[1].as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextPOsition` succeded, initalizing `data`
            true => unsafe {
                Ok(Point::new(data[0].assume_init(), data[1].assume_init()))
            }
        }
    }
    #[doc(alias = "TTF_GetTextWrapWidth")]
    pub fn get_wrap_width(&self) -> Result<Option<NonZeroU32>, Error>{
        let mut wrap_width = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetTextWrapWidth(self.raw(), wrap_width.as_mut_ptr())
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextWrapWidth` has succeeded, initalizing `wrap_width`
            true => unsafe {
                Ok(NonZeroU32::new(wrap_width.assume_init() as u32)) // 0 means "No set wrap width", and a negative wrap width isn't meaningful, so convert to u32
            }
        }
    }
    #[doc(alias = "TTF_GetTextSize")]
    pub fn get_size(&self) -> Result<(u32, u32), Error> {
        let mut data = [MaybeUninit::uninit(); 2];
        let success = unsafe {
            ttf::TTF_GetTextSize(
                self.raw(),
                data[0].as_mut_ptr(),
                data[1].as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextSize` has succeeded, initalizing `data`
            true => unsafe {
                Ok((data[0].assume_init() as u32, data[1].assume_init() as u32)) // Negative size isn't meaningful, so cast to u32
            }
        }
    }
    #[doc(alias = "TTF_GetTextSubString")]
    pub fn get_substring(&self, offset: i32) -> Result<TTF_SubString, Error> {
        let mut substring = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetTextSubString(self.raw(), offset, substring.as_mut_ptr())
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextSubString` succeeded, initalizing `substring`
            true => unsafe {
                Ok(substring.assume_init())
            }
        }
    }
    #[doc(alias = "TTF_GetTextSubStringForLine")]
    pub fn get_substring_for_line(&self, line: usize)  -> Result<TTF_SubString, TextAccessError> {
        let mut substring = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetTextSubStringForLine(
                self.raw(),
                TextAccessError::cast_integer(line)?,
                substring.as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error().into()),
            // Safety: `TTF_GetTextSubStringForLine` succeeded, initalizing `substring`
            true => unsafe {
                Ok(substring.assume_init())
            }
        }
    }    
    #[doc(alias = "TTF_GetTextSubStringsForRange")]
    pub fn get_substrings_for_range(&self, offset: usize, size: Option<u32>) -> Result<SubstringArray, TextAccessError> {
        let size: i32 = size.map(|s| TextAccessError::cast_integer(s)).unwrap_or(Ok(-1))?; // Accessing an impossibly long range would, effectively, always be outside the text range

        let mut substrings_count = MaybeUninit::uninit();
        let success = unsafe {
            NonNull::new(ttf::TTF_GetTextSubStringsForRange(self.raw(), TextAccessError::cast_integer(offset)?, size, substrings_count.as_mut_ptr()))
        };

        match success {
            None => Err(get_error().into()),
            // Safety: 'TTF_GetTextSubStringsForRange` has succeeded, initalizing `substrings` with the quanity of substrings found.`
            Some(substrings) => unsafe {
                Ok(SubstringArray { substrings , length: substrings_count.assume_init() as usize })
            }
        }
    }
    #[doc(alias = "TTF_GetTextSubStringForPoint")]
    pub fn get_substring_for_point(&self, point: Point) -> Result<TTF_SubString, Error> {
        let mut substring = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetTextSubStringForPoint(
                self.raw(),
                point.x,
                point.y,
                substring.as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            // Safety: `TTF_GetTextSubStringForLine` succeeded, initalizing `substring`
            true => unsafe {
                Ok(substring.assume_init())
            }
        }
    }
    #[doc(alias = "TTF_GetPreviousTextSubString")]
    pub fn get_previous_substring(&self, current: TTF_SubString) -> Result<TTF_SubString, Error> {
        let mut substring = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetPreviousTextSubString(
                self.raw(), 
                &raw const current,
                substring.as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            true => unsafe {
                // Safety: `TTF_GetPreviousTextSubString` succeeded, initalizng `substring`
                Ok(substring.assume_init())
            }
        }
    }
    #[doc(alias = "TTF_GetNextTextSubString")]
    pub fn get_next_substring(&self, current: TTF_SubString) -> Result<TTF_SubString, Error> {
        let mut substring = MaybeUninit::uninit();
        let success = unsafe {
            ttf::TTF_GetNextTextSubString(
                self.raw(), 
                &raw const current,
                substring.as_mut_ptr()
            )
        };

        match success {
            false => Err(get_error()),
            true => unsafe {
                // Safety: `TTF_GetNextTextSubString` succeeded, initalizng `substring`
                Ok(substring.assume_init())
            }
        }
    }
    #[doc(alias = "TTF_TextWrapWhitespaceVisible")]
    pub fn get_wrap_whitespace_visible(&self) -> bool {
        unsafe {
            ttf::TTF_TextWrapWhitespaceVisible(self.raw())
        }
    }
    
    #[doc(alias = "TTF_SetTextDirection")]
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextDirection(self.raw(), direction.into())
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }
    pub fn set_script(&mut self, script: Script) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextScript(self.raw(), script.into_tag())
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }
    pub fn set_color(&mut self, color: Color) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextColor(
                self.raw(), 
                color.r,
                color.g,
                color.b,
                color.a
            )
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }
    pub fn set_color_float(&mut self, color: FColor) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextColorFloat(
                self.raw(), 
                color.r, 
                color.g,
                color.b, 
                color.a
            )
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }
    pub fn set_position(&mut self, position: Point) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextPosition(self.raw(), position.x, position.y)
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }
    pub fn set_wrap_width(&mut self, width: Option<NonZeroU32>) -> Result<(), IntegerOrSdlError> {
        let width = width.map(|w| w.into()).unwrap_or(0);
        let success = unsafe {
            ttf::TTF_SetTextWrapWidth(self.raw(), common::validate_int(width, "width")?)
        };

        match success {
            false => Err(get_error().into()),
            true => Ok(())
        }
    }
    pub fn set_wrap_width_whitespace_visible(&mut self, visible: bool) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_SetTextWrapWhitespaceVisible(self.raw(), visible)
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }

    pub fn update_text(&mut self) -> Result<(), Error> {
        let success = unsafe {
            ttf::TTF_UpdateText(self.raw())
        };

        match success {
            false => Err(get_error()),
            true => Ok(())
        }
    }

    /// Get access to the underlying string representing the text
    pub fn str(&self) -> &str {
        // Safety: TTF_Text object's strings are always null terminated strings.
        unsafe {
            let str: &CStr = CStr::from_ptr((*self.raw()).text);
            let str = str.to_bytes();

            str::from_utf8_unchecked(str)
        }
    }

    /// Cast from `Text<E>` to `Text<Unknown>`, erasing the type of the engine used to make the text
    pub fn upcast(&self) -> &Text<'a> {
        // Safety: Any valid Text<E>, for some engine E, is a subtype of Text<Unknown>
        unsafe { std::mem::transmute(self) }
    }
    /// Cast mutably from `Engine<E>` to `Engine<Unknown>`, erasing the type of the engine used to make the text
    pub fn upcast_mut(&mut self) -> &mut Text<'a> {
        // Safety: Any valid Text<E>, for some engine E, is a subtype of Text<Unknown>
        unsafe { std::mem::transmute(self) }
    }

    /// The underlying sys::TTF_Text object. User must ensure const-correctness.
    pub fn raw(&self) -> *mut sys::TTF_Text {
        (self as *const Self).cast_mut().cast()
    }
}
impl<'a> Text<'a, Unknown> {
    /// Downcast `Text<Unknown>` to a specific `Text<Engine>` type based on the `TextEngine` used to create it.
    /// Only implemented for Text<'a, Unknown> because it is meaningless to downcast any other Engine, as renderer types are mutually exclusive.
    /// SDL3 does not expose a way to directly check the type of a `TextEngine`, so you need the original engine used to make the text.
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
    /// Get access to the `GpuAtlasDrawSequence` to render the `Text` using `sdl3::gpu`,
    // Must take `&mut self` because some methods(like modifying fonts) clear draw data, which can cause a use-after-free,
    // This has a poor interaction with `RcText`, since you can't get `&mut Text` from that, but I am not sure that's easily solvable,
    #[doc(alias = "TTF_GetGPUTextDrawData")]
    pub fn atlas_draw_sequence(&mut self) -> Result<Option<&GpuAtlasDrawSequence>, Error> {
        // Safety: As GpuAtlasDrawSequence is a transparet wrapper, it is safe to cast TTF_GPUAtlasDrawSequence to it
        unsafe {
            match NonNull::new(sys::TTF_GetGPUTextDrawData(self.raw())) {
                Some(ptr) => Ok(Some(ptr.cast().as_ref())),

                // `TTF_GetGPUTextDrawData` also returns null for empty text
                None => match self.str() {
                    "" => Ok(None),
                    _  => Err(get_error())
                },
            }
        }
    }
}
impl<'a> Text<'a, Surface> {
    /// Render the text to a `Surface`
    #[doc(alias = "TTF_DrawSurfaceText")]
    pub fn draw_surface(
        &self,
        x: u32,
        y: u32,
        surface: &mut crate::surface::Surface,
    ) -> Result<(), IntegerOrSdlError> {
        let x = common::validate_int(x, "x")?;
        let y = common::validate_int(y, "y")?;

        let success =
            unsafe { sys::TTF_DrawSurfaceText(self.raw(), x, y, surface.raw()) };

        match success {
            false => Err(crate::get_error().into()),
            true => Ok(()),
        }
    }
}
impl<'a> Text<'a, Renderer> {
    /// Render the text with the attached `Renderer`,
    #[doc(alias = "TTF_DrawRendererText")]
    pub fn draw_renderer(&self, x: f32, y: f32) -> Result<(), Error> {
        let success = unsafe { sys::TTF_DrawRendererText(self.raw(), x, y) };

        match success {
            false => Err(crate::get_error()),
            true => Ok(()),
        }
    }
}

/// An exclusively owned wrapper representing `TTF_Text` 
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
    /// Create a new `Text` object using the given `TextEngine` 
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

    /// Create `OwnedText` from a raw `Text` pointer,
    /// # Safety
    /// The engine `engine` MUST be exclusively accessed and created by SDL3,
    pub unsafe fn from_raw(engine: NonNull<Text<E>>) -> Self {
        Self {
            text: engine.cast(),
            _type: PhantomData,
        }
    }
}


/// A reference counted version of `OwnedText`, using SDL3's in-built `refcount` field,
/// 
/// Note: Does not work while with `E = Gpu``, because you can't call `atlas_draw_sequence` without `&mut Text`, meaning it can't be rendered.
pub struct RcText<'a, E: Engine> {
    text: NonNull<Text<'a, E>>,
    _type: PhantomData<E>,
}
impl<'a, E: Engine> RcText<'a, E> {
    pub fn new(owned: OwnedText<'a, E>) -> Self {
        let text = owned.text;
        std::mem::forget(owned); // Prevent `text` getting Drop()'d

        // Safety: We have exclusive mutable access to the reference count
        unsafe {
            (*text.as_ref().raw()).refcount = 1;
        }

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

    /// Get the raw pointer to the `Text`, leaking the `Text` in the process
    pub fn into_raw(self) -> NonNull<Text<'a, E>> {
        let ptr = self.text;
        std::mem::forget(self);

        ptr
    }

    fn reference_count(&self) -> &Cell<u32> {
        // Safety: SDL3 itself does not make use of `refcount`, leaving it avalible as an "application reference count"
        // Additionally, `refcount` is accessed in no other way than through this function, and, by extension, through a `&Cell<u32>`
        unsafe {
            Cell::from_mut(
                (&raw mut (*self.text.as_ref().raw()).refcount as *mut u32).as_mut_unchecked(),
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

/// This type soley exists to allow checking if `atlas_texture` is None without having to make a sampler binding.
/// Can't convert the SDL_Texture to a Texture<> either, because that requires the texture's size, which we don't have,
/// so the only thing you can do with the texture is bind it into a `TextureSamplerBinding`.
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

/// Wrapper around `TTF_GpuAtlasDrawSequence`
#[doc(alias = "TTF_GPUAtlasDrawSequence")]
#[repr(transparent)]
pub struct GpuAtlasDrawSequence(sys::TTF_GPUAtlasDrawSequence);
impl GpuAtlasDrawSequence {
    /// Get access to the `atlas_texture` used by this sequence instruction, if there is one
    pub fn atlas_texture(&self) -> Option<GpuAtlasDrawSequenceTexture<'_>> {
        match NonNull::new(self.0.atlas_texture) {
            None => None,
            Some(texture) => Some(GpuAtlasDrawSequenceTexture(texture, PhantomData)),
        }
    }

    /// Get access to the verticies(`xy`) used for rendering the text
    pub fn vertices(&self) -> &[FPoint] {
        // Safety: `xy` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            std::slice::from_raw_parts(self.0.xy as *const FPoint, self.0.num_vertices as usize)
        }
    }
    /// Get access to the texture coordinates(`uv`) used for rendering the text
    pub fn texture_coordinates(&self) -> &[FPoint] {
        // Safety: `uv` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            std::slice::from_raw_parts(self.0.uv as *const FPoint, self.0.num_vertices as usize)
        }
    }

    /// Get access to the `indicies` of the verticies for rendering the text
    pub fn indices(&self) -> &[u32] {
        // Safety: `indicies` will always be present on any GpuAtlasDrawSequence.
        unsafe {
            // SDL's type is `int` for indicies, for some reason, even though they can't be negative by definition(A negative vertex index means nothing). So cast them into a more conventional u32.
            std::slice::from_raw_parts(
                self.0.indices.cast::<u32>(),
                self.0.num_indices as usize,
            )
        }
    }

    /// Get the `ImageType` assoicated with this text's texture information
    pub fn image_type(&self) -> ImageType {
        self.0.image_type.into()
    }

    /// Get the next instruction in the `GpuAtlasDrawSequence` sequence.
    /// 
    /// Can also be iterated over direction with `.into_iter()`, for easier usage.
    pub fn next<'a>(&'a self) -> Option<&'a Self> {
        // Safety: This is just traversing a linked list. Thus, the life time of the next element will be the same as the lifetime of the current one.
        unsafe {
            self.0
                .next
                .cast::<Self>()
                .as_ref()
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

/// Iterator over every draw instruction in `GpuAtlasDrawSequence`
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
