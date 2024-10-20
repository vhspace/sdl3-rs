use crate::get_error;
use crate::sys;
use std::convert::TryInto;
use std::convert::{Into, TryFrom};
use std::ffi::c_int;
use std::fmt::Debug;
use std::ptr::null;
use sys::everything::{SDL_PixelFormat, SDL_PixelFormatDetails};

pub struct Palette {
    raw: *mut sys::pixels::SDL_Palette,
}

impl Palette {
    #[inline]
    /// Creates a new, uninitialized palette
    #[doc(alias = "SDL_CreatePalette")]
    pub fn new(mut capacity: usize) -> Result<Self, String> {
        use crate::common::*;

        let ncolors = {
            // This is kind of a hack. We have to cast twice because
            // ncolors is a c_int, and validate_int only takes a u32.
            // FIXME: Modify validate_int to make this unnecessary
            let u32_max = u32::max_value() as usize;
            if capacity > u32_max {
                capacity = u32_max;
            }

            match validate_int(capacity as u32, "capacity") {
                Ok(len) => len,
                Err(e) => return Err(format!("{}", e)),
            }
        };

        let raw = unsafe { sys::pixels::SDL_CreatePalette(ncolors) };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Palette { raw })
        }
    }

    /// Creates a palette from the provided colors
    #[doc(alias = "SDL_SetPaletteColors")]
    pub fn with_colors(colors: &[Color]) -> Result<Self, String> {
        let pal = Self::new(colors.len())?;

        // Already validated, so don't check again
        let ncolors = colors.len() as ::libc::c_int;

        let result = unsafe {
            let mut raw_colors: Vec<sys::pixels::SDL_Color> =
                colors.iter().map(|color| color.raw()).collect();

            let pal_ptr = (&mut raw_colors[0]) as *mut sys::pixels::SDL_Color;

            sys::pixels::SDL_SetPaletteColors(pal.raw, pal_ptr, 0, ncolors)
        };

        if !result {
            Err(get_error())
        } else {
            Ok(pal)
        }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.raw).ncolors as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for Palette {
    #[doc(alias = "SDL_DestroyPalette")]
    fn drop(&mut self) {
        unsafe {
            sys::pixels::SDL_DestroyPalette(self.raw);
        }
    }
}

impl_raw_accessors!((Palette, *mut sys::pixels::SDL_Palette));

#[test]
fn create_palette() {
    let colors: Vec<_> = (0..0xff).map(|u| Color::RGB(u, 0, 0xff - u)).collect();

    let palette = Palette::with_colors(&colors).unwrap();

    assert!(palette.len() == 255);
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline]
    #[allow(non_snake_case)]
    pub const fn RGB(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 0xff }
    }

    #[inline]
    #[allow(non_snake_case)]
    pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    #[doc(alias = "SDL_MapRGBA")]
    pub fn to_u32(self, format: &PixelFormat) -> u32 {
        unsafe {
            sys::pixels::SDL_MapRGBA(
                format.pixel_format_details(),
                null(),
                self.r,
                self.g,
                self.b,
                self.a,
            )
        }
    }

    #[doc(alias = "SDL_GetRGBA")]
    pub fn from_u32(format: &PixelFormat, pixel: u32) -> Color {
        let (mut r, mut g, mut b, mut a) = (0, 0, 0, 0);

        unsafe {
            sys::pixels::SDL_GetRGBA(
                pixel,
                format.pixel_format_details(),
                null(),
                &mut r,
                &mut g,
                &mut b,
                &mut a,
            )
        };
        Color::RGBA(r, g, b, a)
    }

    pub fn invert(self) -> Color {
        Color::RGBA(255 - self.r, 255 - self.g, 255 - self.b, 255 - self.a)
    }

    #[inline]
    pub const fn rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    #[inline]
    pub const fn rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    // Implemented manually and kept private, because reasons
    #[inline]
    const fn raw(self) -> sys::pixels::SDL_Color {
        sys::pixels::SDL_Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    pub const WHITE: Color = Color::RGBA(255, 255, 255, 255);
    pub const BLACK: Color = Color::RGBA(0, 0, 0, 255);
    pub const GRAY: Color = Color::RGBA(128, 128, 128, 255);
    pub const GREY: Color = Color::GRAY;
    pub const RED: Color = Color::RGBA(255, 0, 0, 255);
    pub const GREEN: Color = Color::RGBA(0, 255, 0, 255);
    pub const BLUE: Color = Color::RGBA(0, 0, 255, 255);
    pub const MAGENTA: Color = Color::RGBA(255, 0, 255, 255);
    pub const YELLOW: Color = Color::RGBA(255, 255, 0, 255);
    pub const CYAN: Color = Color::RGBA(0, 255, 255, 255);
}

impl Into<sys::pixels::SDL_Color> for Color {
    fn into(self) -> sys::pixels::SDL_Color {
        self.raw()
    }
}

impl From<sys::pixels::SDL_Color> for Color {
    fn from(raw: sys::pixels::SDL_Color) -> Color {
        Color::RGBA(raw.r, raw.g, raw.b, raw.a)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Color {
        Color::RGB(r, g, b)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Color {
        Color::RGBA(r, g, b, a)
    }
}

pub struct PixelMasks {
    /// Bits per pixel; usually 15, 16, or 32
    pub bpp: u8,
    /// The red mask
    pub rmask: u32,
    /// The green mask
    pub gmask: u32,
    /// The blue mask
    pub bmask: u32,
    /// The alpha mask
    pub amask: u32,
}

/// A pixel format, i.e. a set of masks that define how to pack and unpack pixel data.
/// This is used to convert between pixel data and surface data.
/// It wraps an SDL_PixelFormat.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PixelFormat {
    raw: SDL_PixelFormat,
}

impl_raw_accessors!((PixelFormat, sys::pixels::SDL_PixelFormat));
impl_raw_constructor!((PixelFormat, PixelFormat(raw: sys::pixels::SDL_PixelFormat )));

impl Debug for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PixelFormat({:?})", self.raw.0)
    }
}

impl PixelFormat {
    pub unsafe fn pixel_format_details(&self) -> *const SDL_PixelFormatDetails {
        sys::pixels::SDL_GetPixelFormatDetails(self.raw)
    }

    #[doc(alias = "SDL_GetPixelFormatForMasks")]
    pub fn from_masks(masks: PixelMasks) -> PixelFormat {
        unsafe {
            PixelFormat::from_ll(sys::pixels::SDL_GetPixelFormatForMasks(
                masks.bpp as i32,
                masks.rmask,
                masks.gmask,
                masks.bmask,
                masks.amask,
            ))
        }
    }

    #[doc(alias = "SDL_GetMasksForPixelFormat")]
    pub fn into_masks(self) -> Result<PixelMasks, String> {
        let mut bpp = 0;
        let mut rmask = 0;
        let mut gmask = 0;
        let mut bmask = 0;
        let mut amask = 0;
        let result = unsafe {
            sys::pixels::SDL_GetMasksForPixelFormat(
                self.raw, &mut bpp, &mut rmask, &mut gmask, &mut bmask, &mut amask,
            )
        };
        if !result {
            Err(get_error())
        } else {
            Ok(PixelMasks {
                bpp: bpp as u8,
                rmask,
                gmask,
                bmask,
                amask,
            })
        }
    }

    /// Calculates the total byte size of an image buffer, given its pitch
    /// and height.
    pub fn byte_size_from_pitch_and_height(self, pitch: usize, height: usize) -> usize {
        match self.raw {
            SDL_PixelFormat::YV12 | SDL_PixelFormat::IYUV => {
                // YUV is 4:2:0.
                // `pitch` is the width of the Y component, and
                // `height` is the height of the Y component.
                // U and V have half the width and height of Y.
                pitch * height + 2 * (pitch / 2 * height / 2)
            }
            _ => pitch * height,
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn byte_size_of_pixels(self, num_of_pixels: usize) -> usize {
        match self.raw {
            SDL_PixelFormat::RGB332 => num_of_pixels,
            SDL_PixelFormat::XRGB4444
            | SDL_PixelFormat::XRGB1555
            | SDL_PixelFormat::XBGR1555
            | SDL_PixelFormat::ARGB4444
            | SDL_PixelFormat::RGBA4444
            | SDL_PixelFormat::ABGR4444
            | SDL_PixelFormat::BGRA4444
            | SDL_PixelFormat::ARGB1555
            | SDL_PixelFormat::RGBA5551
            | SDL_PixelFormat::ABGR1555
            | SDL_PixelFormat::BGRA5551
            | SDL_PixelFormat::RGB565
            | SDL_PixelFormat::BGR565 => num_of_pixels * 2,
            SDL_PixelFormat::RGB24 | SDL_PixelFormat::BGR24 => num_of_pixels * 3,
            SDL_PixelFormat::XRGB8888
            | SDL_PixelFormat::RGBX8888
            | SDL_PixelFormat::XBGR8888
            | SDL_PixelFormat::BGRX8888
            | SDL_PixelFormat::ARGB8888
            | SDL_PixelFormat::RGBA8888
            | SDL_PixelFormat::ABGR8888
            | SDL_PixelFormat::BGRA8888
            | SDL_PixelFormat::ARGB2101010 => num_of_pixels * 4,
            // YUV formats
            // FIXME: rounding error here?
            SDL_PixelFormat::YV12 | SDL_PixelFormat::IYUV => num_of_pixels / 2 * 3,
            SDL_PixelFormat::YUY2 | SDL_PixelFormat::UYVY | SDL_PixelFormat::YVYU => {
                num_of_pixels * 2
            }
            // Unsupported formats
            SDL_PixelFormat::INDEX8 => num_of_pixels,
            SDL_PixelFormat::UNKNOWN
            | SDL_PixelFormat::INDEX1LSB
            | SDL_PixelFormat::INDEX1MSB
            | SDL_PixelFormat::INDEX4LSB
            | SDL_PixelFormat::INDEX4MSB
            | _ => panic!("not supported format: {:?}", self),
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn byte_size_per_pixel(self) -> usize {
        match self.raw {
            SDL_PixelFormat::RGB332 => 1,
            SDL_PixelFormat::XRGB4444
            | SDL_PixelFormat::XRGB1555
            | SDL_PixelFormat::XBGR1555
            | SDL_PixelFormat::ARGB4444
            | SDL_PixelFormat::RGBA4444
            | SDL_PixelFormat::ABGR4444
            | SDL_PixelFormat::BGRA4444
            | SDL_PixelFormat::ARGB1555
            | SDL_PixelFormat::RGBA5551
            | SDL_PixelFormat::ABGR1555
            | SDL_PixelFormat::BGRA5551
            | SDL_PixelFormat::RGB565
            | SDL_PixelFormat::BGR565 => 2,
            SDL_PixelFormat::RGB24 | SDL_PixelFormat::BGR24 => 3,
            SDL_PixelFormat::XRGB8888
            | SDL_PixelFormat::RGBX8888
            | SDL_PixelFormat::XBGR8888
            | SDL_PixelFormat::BGRX8888
            | SDL_PixelFormat::ARGB8888
            | SDL_PixelFormat::RGBA8888
            | SDL_PixelFormat::ABGR8888
            | SDL_PixelFormat::BGRA8888
            | SDL_PixelFormat::ARGB2101010 => 4,
            // YUV formats
            SDL_PixelFormat::YV12 | SDL_PixelFormat::IYUV => 1,
            SDL_PixelFormat::YUY2 | SDL_PixelFormat::UYVY | SDL_PixelFormat::YVYU => 2,
            // Unsupported formats
            SDL_PixelFormat::INDEX8 => 1,
            SDL_PixelFormat::UNKNOWN
            | SDL_PixelFormat::INDEX1LSB
            | SDL_PixelFormat::INDEX1MSB
            | SDL_PixelFormat::INDEX4LSB
            | SDL_PixelFormat::INDEX4MSB
            | _ => panic!("not supported format: {:?}", self),
        }
    }

    pub fn supports_alpha(self) -> bool {
        match self.raw {
            SDL_PixelFormat::ARGB4444 | SDL_PixelFormat::ARGB1555 | SDL_PixelFormat::ARGB8888 | SDL_PixelFormat::ARGB2101010 | SDL_PixelFormat::ABGR4444 | SDL_PixelFormat::ABGR1555 | SDL_PixelFormat::ABGR8888
            | SDL_PixelFormat::BGRA4444 | SDL_PixelFormat::BGRA5551 | SDL_PixelFormat::BGRA8888 | SDL_PixelFormat::RGBA4444 | SDL_PixelFormat::RGBA5551 | SDL_PixelFormat::RGBA8888 => true,
            _ => false,
        }
    }
}

impl From<PixelFormat> for SDL_PixelFormat {
    fn from(pf: PixelFormat) -> SDL_PixelFormat {
        pf.raw
    }
}

impl From<i64> for PixelFormat {
    fn from(format: i64) -> PixelFormat {
        let format_c_int: c_int = format
            .try_into()
            .expect("Pixel format value out of range for c_int");
        let pixel_format = SDL_PixelFormat(format_c_int);
        PixelFormat { raw: pixel_format }
    }
}

impl TryFrom<SDL_PixelFormat> for PixelFormat {
    type Error = String;

    #[doc(alias = "SDL_GetPixelFormatDetails")]
    fn try_from(format: SDL_PixelFormat) -> Result<Self, Self::Error> {
        unsafe {
            let pf_ptr = sys::pixels::SDL_GetPixelFormatDetails(format.into());
            if pf_ptr.is_null() {
                Err(get_error())
            } else {
                Ok(PixelFormat::from_ll((*pf_ptr).format))
            }
        }
    }
}
