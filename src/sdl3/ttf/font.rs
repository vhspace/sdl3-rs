use crate::iostream::IOStream;
use crate::pixels::Color;
use crate::surface::Surface;
use crate::{get_error, Error};
use sdl3_ttf_sys::ttf;
use std::error;
use std::ffi::NulError;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_int;
use sys::surface::SDL_Surface;

use super::Sdl3TtfContext;

/// Information about the hinting of a font.
/// See [wikipedia](https://en.wikipedia.org/wiki/Font_hinting)
pub type Hinting = ttf::TTF_HintingFlags;

bitflags! {
    /// The styling of a font.
    pub struct FontStyle: u32 {
        const NORMAL        = ttf::TTF_STYLE_NORMAL as u32;
        const BOLD          = ttf::TTF_STYLE_BOLD as u32;
        const ITALIC        = ttf::TTF_STYLE_ITALIC as u32;
        const UNDERLINE     = ttf::TTF_STYLE_UNDERLINE as u32;
        const STRIKETHROUGH = ttf::TTF_STYLE_STRIKETHROUGH as u32;
    }
}

/// Information about a specific glyph (character) in a font face.
#[derive(Debug, PartialEq, Clone)]
pub struct GlyphMetrics {
    pub minx: i32,
    pub maxx: i32,
    pub miny: i32,
    pub maxy: i32,
    pub advance: i32,
}

/// The result of an `SDL2_TTF` font operation.
pub type FontResult<T> = Result<T, FontError>;

/// A font-related error.
#[derive(Debug, Clone)]
pub enum FontError {
    /// A Latin-1 encoded byte string is invalid.
    InvalidLatin1Text(NulError),
    /// A SDL2-related error occured.
    SdlError(Error),
}

impl error::Error for FontError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            FontError::InvalidLatin1Text(ref error) => Some(error),
            FontError::SdlError(ref error) => Some(error),
        }
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            FontError::InvalidLatin1Text(ref err) => {
                write!(f, "Invalid Latin-1 bytes: {}", err)
            }
            FontError::SdlError(ref err) => {
                write!(f, "SDL3 error: {}", err)
            }
        }
    }
}

/// A renderable piece of text in the UTF8 or Latin-1 format.
enum RenderableText<'a> {
    Utf8(&'a str),
    Char(String),
}
impl<'a> RenderableText<'a> {
    /// Converts the given text to a c-style string if possible.
    fn convert(&self) -> FontResult<CString> {
        match *self {
            RenderableText::Utf8(text) => Ok(CString::new(text).unwrap()),
            RenderableText::Char(ref string) => Ok(CString::new(string.as_bytes()).unwrap()),
        }
    }
}

/// A builder for a font rendering.
#[must_use]
pub struct PartialRendering<'f, 'text> {
    text: RenderableText<'text>,
    font: &'f Font<'f>,
}

/// Converts the given raw pointer to a surface.
fn convert_to_surface<'a>(raw: *mut SDL_Surface) -> FontResult<Surface<'a>> {
    if (raw as *mut ()).is_null() {
        Err(FontError::SdlError(get_error()))
    } else {
        Ok(unsafe { Surface::from_ll(raw) })
    }
}

impl<'f, 'text> PartialRendering<'f, 'text> {
    /// Renders the text in *solid* mode.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_Solid)
    /// for an explanation.
    #[doc(alias = "TTF_RenderText_Solid")]
    pub fn solid<'b, T>(self, color: T) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let color = color.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => {
                    ttf::TTF_RenderText_Solid(self.font.raw(), source.as_ptr(), 0, color)
                }
            }
        };
        convert_to_surface(raw)
    }

    /// Renders the text in *shaded* mode.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_Shaded)
    /// for an explanation.
    #[doc(alias = "TTF_RenderText_Shaded")]
    pub fn shaded<'b, T>(self, color: T, background: T) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let foreground = color.into().into();
        let background = background.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => ttf::TTF_RenderText_Shaded(
                    self.font.raw(),
                    source.as_ptr(),
                    0,
                    foreground,
                    background,
                ),
            }
        };
        convert_to_surface(raw)
    }

    /// Renders the text in *blended* mode.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_Blended)
    /// for an explanation.
    #[doc(alias = "TTF_RenderText_Blended")]
    pub fn blended<'b, T>(self, color: T) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let color = color.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => {
                    ttf::TTF_RenderText_Blended(self.font.raw(), source.as_ptr(), 0, color)
                }
            }
        };
        convert_to_surface(raw)
    }

    /// Renders the text in *blended* mode but wrapping the words if the width
    /// exceeds the given maximum width.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_Blended_Wrapped)
    /// for an explanation of the mode.
    #[doc(alias = "TTF_RenderText_Blended_Wrapped")]
    pub fn blended_wrapped<'b, T>(self, color: T, wrap_max_width: i32) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let color = color.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => {
                    ttf::TTF_RenderText_Blended_Wrapped(
                        self.font.raw(),
                        source.as_ptr(),
                        0,
                        color,
                        wrap_max_width,
                    )
                }
            }
        };
        convert_to_surface(raw)
    }

    /// Renders the text in *LCD subpixel* mode.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_LCD)
    /// for an explanation.
    #[doc(alias = "TTF_RenderText_LCD")]
    pub fn lcd<'b, T>(self, foreground: T, background: T) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let foreground = foreground.into().into();
        let background = background.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => ttf::TTF_RenderText_LCD(
                    self.font.raw(),
                    source.as_ptr(),
                    0,
                    foreground,
                    background,
                ),
            }
        };
        convert_to_surface(raw)
    }

    /// Renders the text in *LCD subpixel* mode but wrapping the words if the width
    /// exceeds the given maximum width.
    /// See [the SDL3_TTF docs](https://wiki.libsdl.org/SDL3_ttf/TTF_RenderText_LCD_Wrapped)
    /// for an explanation of the mode.
    #[doc(alias = "TTF_RenderText_LCD_Wrapped")]
    pub fn lcd_wrapped<'b, T>(
        self,
        foreground: T,
        background: T,
        wrap_max_width: i32,
    ) -> FontResult<Surface<'b>>
    where
        T: Into<Color>,
    {
        let source = self.text.convert()?;
        let foreground = foreground.into().into();
        let background = background.into().into();
        let raw = unsafe {
            match self.text {
                RenderableText::Utf8(_) | RenderableText::Char(_) => {
                    ttf::TTF_RenderText_LCD_Wrapped(
                        self.font.raw(),
                        source.as_ptr(),
                        0,
                        foreground,
                        background,
                        wrap_max_width,
                    )
                }
            }
        };
        convert_to_surface(raw)
    }
}

/// A loaded TTF font.
pub struct Font<'iostream> {
    raw: *mut ttf::TTF_Font,

    // This will keep TTF alive and avoid having to deal with lifetimes
    #[allow(dead_code)]
    context: Sdl3TtfContext,

    // Iostream is only stored here because it must not outlive
    // the Font struct, and this Iostream should not be used by
    // anything else
    // None means that the Iostream is handled by SDL itself,
    // and Some(iostream) means that the Iostream is handled by the Rust
    // side
    #[allow(dead_code)]
    iostream: Option<IOStream<'iostream>>,
}

impl<'r> Drop for Font<'r> {
    fn drop(&mut self) {
        unsafe {
            // avoid close font after quit()
            if ttf::TTF_WasInit() == 1 {
                ttf::TTF_CloseFont(self.raw);
            }
        }
    }
}

impl<'r> Font<'r> {
    pub(super) fn new(
        context: Sdl3TtfContext,
        raw: *mut ttf::TTF_Font,
        iostream: Option<IOStream<'r>>,
    ) -> Self {
        Self {
            context,
            raw,
            iostream,
        }
    }

    /// Returns the underlying C font object.
    // this can prevent introducing UB until
    // https://github.com/rust-lang/rust-clippy/issues/5953 is fixed
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn raw(&self) -> *mut ttf::TTF_Font {
        self.raw
    }

    /// Starts specifying a rendering of the given UTF-8-encoded text.
    pub fn render<'a, 'b>(&'a self, text: &'b str) -> PartialRendering<'a, 'b> {
        PartialRendering {
            text: RenderableText::Utf8(text),
            font: self,
        }
    }

    /// Starts specifying a rendering of the given UTF-8-encoded character.
    pub fn render_char<'a>(&'a self, ch: char) -> PartialRendering<'a, 'static> {
        let mut s = String::new();
        s.push(ch);
        PartialRendering {
            text: RenderableText::Char(s),
            font: self,
        }
    }

    /// Returns the width and height of the given text when rendered using this
    /// font.
    #[doc(alias = "TTF_GetStringSize")]
    pub fn size_of(&self, text: &str) -> FontResult<(u32, u32)> {
        let c_string = RenderableText::Utf8(text).convert()?;
        let (res, size) = unsafe {
            let mut w = 0; // mutated by C code
            let mut h = 0; // mutated by C code
            let ret = ttf::TTF_GetStringSize(self.raw, c_string.as_ptr(), 0, &mut w, &mut h);
            (ret, (w as u32, h as u32))
        };
        if res {
            Ok(size)
        } else {
            Err(FontError::SdlError(get_error()))
        }
    }

    /// Returns the width and height of the given text when rendered using this
    /// font.
    pub fn size_of_char(&self, ch: char) -> FontResult<(u32, u32)> {
        let mut s = String::new();
        s.push(ch);
        self.size_of(&s)
    }

    /// Returns the font's style flags.
    #[doc(alias = "TTF_GetFontStyle")]
    pub fn get_style(&self) -> FontStyle {
        unsafe {
            let raw = ttf::TTF_GetFontStyle(self.raw);
            FontStyle::from_bits_truncate(raw as u32)
        }
    }

    /// Sets the font's style flags.
    #[doc(alias = "TTF_SetFontStyle")]
    pub fn set_style(&mut self, styles: FontStyle) {
        unsafe { ttf::TTF_SetFontStyle(self.raw, styles.bits()) }
    }

    /// Returns the width of the font's outline.
    #[doc(alias = "TTF_GetFontOutline")]
    pub fn get_outline_width(&self) -> u16 {
        unsafe { ttf::TTF_GetFontOutline(self.raw) as u16 }
    }

    /// Sets the width of the font's outline.
    #[doc(alias = "TTF_SetFontOutline")]
    pub fn set_outline_width(&mut self, width: u16) -> bool {
        unsafe { ttf::TTF_SetFontOutline(self.raw, width as c_int) }
    }

    /// Returns the font's freetype hints.
    #[doc(alias = "TTF_GetFontHinting")]
    pub fn get_hinting(&self) -> Hinting {
        unsafe { ttf::TTF_GetFontHinting(self.raw) }
    }

    /// Sets the font's freetype hints.
    #[doc(alias = "TTF_SetFontHinting")]
    pub fn set_hinting(&mut self, hinting: Hinting) {
        unsafe { ttf::TTF_SetFontHinting(self.raw, hinting) }
    }

    /// Returns whether the font is kerning.
    #[doc(alias = "TTF_GetFontKerning")]
    pub fn get_kerning(&self) -> bool {
        unsafe { ttf::TTF_GetFontKerning(self.raw) }
    }

    /// Sets whether the font should use kerning.
    #[doc(alias = "TTF_SetFontKerning")]
    pub fn set_kerning(&mut self, kerning: bool) {
        unsafe { ttf::TTF_SetFontKerning(self.raw, kerning) }
    }

    /// Returns the total height of the font.
    #[doc(alias = "TTF_GetFontHeight")]
    pub fn height(&self) -> i32 {
        //! Get font maximum total height.
        unsafe { ttf::TTF_GetFontHeight(self.raw) as i32 }
    }

    /// Returns the font's highest ascent (height above base).
    #[doc(alias = "TTF_GetFontAscent")]
    pub fn ascent(&self) -> i32 {
        unsafe { ttf::TTF_GetFontAscent(self.raw) as i32 }
    }

    /// Returns the font's lowest descent (height below base).
    /// This is a negative number.
    #[doc(alias = "TTF_GetFontDescent")]
    pub fn descent(&self) -> i32 {
        unsafe { ttf::TTF_GetFontDescent(self.raw) as i32 }
    }

    /// Returns the recommended line spacing for text rendered with this font.
    #[doc(alias = "TTF_GetFontLineSkip")]
    pub fn recommended_line_spacing(&self) -> i32 {
        unsafe { ttf::TTF_GetFontLineSkip(self.raw) as i32 }
    }

    /// Returns the number of faces in this font.
    #[doc(alias = "TTF_GetNumFontFaces")]
    pub fn face_count(&self) -> u16 {
        unsafe { ttf::TTF_GetNumFontFaces(self.raw) as u16 }
    }

    /// Returns whether the font is monospaced.
    #[doc(alias = "TTF_FontIsFixedWidth")]
    pub fn face_is_fixed_width(&self) -> bool {
        unsafe { ttf::TTF_FontIsFixedWidth(self.raw) }
    }

    /// Returns the family name of the current font face.
    #[doc(alias = "TTF_GetFontFamilyName")]
    pub fn face_family_name(&self) -> Option<String> {
        unsafe {
            // not owns buffer
            let cname = ttf::TTF_GetFontFamilyName(self.raw);
            if cname.is_null() {
                None
            } else {
                Some(String::from_utf8_lossy(CStr::from_ptr(cname).to_bytes()).to_string())
            }
        }
    }

    /// Returns the name of the current font face.
    #[doc(alias = "TTF_GetFontStyleName")]
    pub fn face_style_name(&self) -> Option<String> {
        unsafe {
            let cname = ttf::TTF_GetFontStyleName(self.raw);
            if cname.is_null() {
                None
            } else {
                Some(String::from_utf8_lossy(CStr::from_ptr(cname).to_bytes()).to_string())
            }
        }
    }

    /// Returns the index of the given character in this font face.
    #[doc(alias = "TTF_FontHasGlyph")]
    pub fn find_glyph(&self, ch: char) -> Option<u16> {
        unsafe {
            let ret = ttf::TTF_FontHasGlyph(self.raw, ch as u32);
            if !ret {
                None
            } else {
                Some(ret as u16)
            }
        }
    }

    /// Returns the glyph metrics of the given character in this font face.
    #[doc(alias = "TTF_GetGlyphMetrics")]
    pub fn find_glyph_metrics(&self, ch: char) -> Option<GlyphMetrics> {
        let minx = 0;
        let maxx = 0;
        let miny = 0;
        let maxy = 0;
        let advance = 0;
        let ret = unsafe {
            ttf::TTF_GetGlyphMetrics(
                self.raw,
                ch as u32,
                &minx as *const _ as *mut _,
                &maxx as *const _ as *mut _,
                &miny as *const _ as *mut _,
                &maxy as *const _ as *mut _,
                &advance as *const _ as *mut _,
            )
        };
        if ret {
            Some(GlyphMetrics {
                minx: minx as i32,
                maxx: maxx as i32,
                miny: miny as i32,
                maxy: maxy as i32,
                advance: advance as i32,
            })
        } else {
            None
        }
    }
}
