//!
//! A binding for the library `sdl3_image`
//!
//!
//! Note that you need to build with the
//! feature `image` for this module to be enabled,
//! like so:
//!
//! ```bash
//! $ cargo build --features "image"
//! ```
//!
//! If you want to use this with from inside your own
//! crate, you will need to add this in your Cargo.toml
//!
//! ```toml
//! [dependencies.sdl3]
//! version = ...
//! default-features = false
//! features = ["image"]
//! ```

use crate::iostream::IOStream;
use crate::render::{Texture, TextureCreator};
use crate::surface::Surface;
use crate::version::Version;
use crate::{get_error, Error};
use sdl3_image_sys::image;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use sys;

/// Static method extensions for creating Surfaces
pub trait LoadSurface: Sized {
    // Self is only returned here to type hint to the compiler.
    // The syntax for type hinting in this case is not yet defined.
    // The intended return value is Result<~Surface, Error>.
    fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, Error>;
    fn from_xpm_array(xpm: *const *const i8) -> Result<Self, Error>;
}

/// Method extensions to Surface for saving to disk
pub trait SaveSurface {
    fn save<P: AsRef<Path>>(&self, filename: P) -> Result<(), Error>;
    fn save_io(&self, dst: &mut IOStream) -> Result<(), Error>;
}

impl<'a> LoadSurface for Surface<'a> {
    fn from_file<P: AsRef<Path>>(filename: P) -> Result<Surface<'a>, Error> {
        //! Loads an SDL Surface from a file
        unsafe {
            let c_filename = CString::new(filename.as_ref().to_str().unwrap()).unwrap();
            let raw = image::IMG_Load(c_filename.as_ptr() as *const _);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(Surface::from_ll(raw))
            }
        }
    }

    fn from_xpm_array(xpm: *const *const i8) -> Result<Surface<'a>, Error> {
        //! Loads an SDL Surface from XPM data
        unsafe {
            let raw = image::IMG_ReadXPMFromArray(xpm as *mut *mut c_char);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(Surface::from_ll(raw))
            }
        }
    }
}

impl<'a> SaveSurface for Surface<'a> {
    fn save<P: AsRef<Path>>(&self, filename: P) -> Result<(), Error> {
        //! Saves an SDL Surface to a file
        unsafe {
            let c_filename = CString::new(filename.as_ref().to_str().unwrap()).unwrap();
            if image::IMG_SavePNG(self.raw(), c_filename.as_ptr() as *const _) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    fn save_io(&self, dst: &mut IOStream) -> Result<(), Error> {
        //! Saves an SDL Surface to an IOStream
        unsafe {
            if image::IMG_SavePNG_IO(self.raw(), dst.raw(), false) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }
}

/// Method extensions for creating Textures from a `TextureCreator`
pub trait LoadTexture {
    fn load_texture<P: AsRef<Path>>(&self, filename: P) -> Result<Texture, Error>;
    fn load_texture_bytes(&self, buf: &[u8]) -> Result<Texture, Error>;
}

impl<T> LoadTexture for TextureCreator<T> {
    fn load_texture<P: AsRef<Path>>(&self, filename: P) -> Result<Texture, Error> {
        //! Loads an SDL Texture from a file
        unsafe {
            let c_filename = CString::new(filename.as_ref().to_str().unwrap()).unwrap();
            let raw = image::IMG_LoadTexture(self.raw(), c_filename.as_ptr() as *const _);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(self.raw_create_texture(raw))
            }
        }
    }

    #[doc(alias = "IMG_LoadTexture")]
    fn load_texture_bytes(&self, buf: &[u8]) -> Result<Texture, Error> {
        //! Loads an SDL Texture from a buffer that the format must be something supported by sdl3_image (png, jpeg, ect, but NOT RGBA8888 bytes for instance)
        unsafe {
            let buf = sdl3_sys::iostream::SDL_IOFromMem(
                buf.as_ptr() as *mut libc::c_void,
                buf.len() as usize,
            );
            let raw = image::IMG_LoadTexture_IO(self.raw(), buf, true); // close(free) buff after load
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(self.raw_create_texture(raw))
            }
        }
    }
}

/// Returns the version of the dynamically linked `SDL_image` library
pub fn get_linked_version() -> Version {
    unsafe { Version::from_ll(image::IMG_Version()) }
}

#[inline]
fn to_surface_result<'a>(raw: *mut sys::surface::SDL_Surface) -> Result<Surface<'a>, Error> {
    if (raw as *mut ()).is_null() {
        Err(get_error())
    } else {
        unsafe { Ok(Surface::from_ll(raw)) }
    }
}

pub trait ImageIOStream {
    /// load as a surface. except TGA
    fn load(&self) -> Result<Surface<'static>, Error>;
    /// load as a surface. This can load all supported image formats.
    fn load_typed(&self, _type: &str) -> Result<Surface<'static>, Error>;

    fn load_cur(&self) -> Result<Surface<'static>, Error>;
    fn load_ico(&self) -> Result<Surface<'static>, Error>;
    fn load_bmp(&self) -> Result<Surface<'static>, Error>;
    fn load_pnm(&self) -> Result<Surface<'static>, Error>;
    fn load_xpm(&self) -> Result<Surface<'static>, Error>;
    fn load_xcf(&self) -> Result<Surface<'static>, Error>;
    fn load_pcx(&self) -> Result<Surface<'static>, Error>;
    fn load_gif(&self) -> Result<Surface<'static>, Error>;
    fn load_jpg(&self) -> Result<Surface<'static>, Error>;
    fn load_tif(&self) -> Result<Surface<'static>, Error>;
    fn load_png(&self) -> Result<Surface<'static>, Error>;
    fn load_tga(&self) -> Result<Surface<'static>, Error>;
    fn load_lbm(&self) -> Result<Surface<'static>, Error>;
    fn load_xv(&self) -> Result<Surface<'static>, Error>;
    fn load_webp(&self) -> Result<Surface<'static>, Error>;

    fn is_cur(&self) -> bool;
    fn is_ico(&self) -> bool;
    fn is_bmp(&self) -> bool;
    fn is_pnm(&self) -> bool;
    fn is_xpm(&self) -> bool;
    fn is_xcf(&self) -> bool;
    fn is_pcx(&self) -> bool;
    fn is_gif(&self) -> bool;
    fn is_jpg(&self) -> bool;
    fn is_tif(&self) -> bool;
    fn is_png(&self) -> bool;
    fn is_lbm(&self) -> bool;
    fn is_xv(&self) -> bool;
    fn is_webp(&self) -> bool;
}

impl<'a> ImageIOStream for IOStream<'a> {
    fn load(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_Load_IO(self.raw(), false) };
        to_surface_result(raw)
    }
    fn load_typed(&self, _type: &str) -> Result<Surface<'static>, Error> {
        let raw = unsafe {
            let c_type = CString::new(_type.as_bytes()).unwrap();
            image::IMG_LoadTyped_IO(self.raw(), false, c_type.as_ptr() as *const _)
        };
        to_surface_result(raw)
    }

    fn load_cur(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadCUR_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_ico(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadICO_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_bmp(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadBMP_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_pnm(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadPNM_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xpm(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadXPM_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xcf(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadXCF_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_pcx(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadPCX_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_gif(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadGIF_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_jpg(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadJPG_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_tif(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadTIF_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_png(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadPNG_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_tga(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadTGA_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_lbm(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadLBM_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xv(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadXV_IO(self.raw()) };
        to_surface_result(raw)
    }
    fn load_webp(&self) -> Result<Surface<'static>, Error> {
        let raw = unsafe { image::IMG_LoadWEBP_IO(self.raw()) };
        to_surface_result(raw)
    }

    fn is_cur(&self) -> bool {
        unsafe { image::IMG_isCUR(self.raw()) }
    }
    fn is_ico(&self) -> bool {
        unsafe { image::IMG_isICO(self.raw()) }
    }
    fn is_bmp(&self) -> bool {
        unsafe { image::IMG_isBMP(self.raw()) }
    }
    fn is_pnm(&self) -> bool {
        unsafe { image::IMG_isPNM(self.raw()) }
    }
    fn is_xpm(&self) -> bool {
        unsafe { image::IMG_isXPM(self.raw()) }
    }
    fn is_xcf(&self) -> bool {
        unsafe { image::IMG_isXCF(self.raw()) }
    }
    fn is_pcx(&self) -> bool {
        unsafe { image::IMG_isPCX(self.raw()) }
    }
    fn is_gif(&self) -> bool {
        unsafe { image::IMG_isGIF(self.raw()) }
    }
    fn is_jpg(&self) -> bool {
        unsafe { image::IMG_isJPG(self.raw()) }
    }
    fn is_tif(&self) -> bool {
        unsafe { image::IMG_isTIF(self.raw()) }
    }
    fn is_png(&self) -> bool {
        unsafe { image::IMG_isPNG(self.raw()) }
    }
    fn is_lbm(&self) -> bool {
        unsafe { image::IMG_isLBM(self.raw()) }
    }
    fn is_xv(&self) -> bool {
        unsafe { image::IMG_isXV(self.raw()) }
    }
    fn is_webp(&self) -> bool {
        unsafe { image::IMG_isWEBP(self.raw()) }
    }
}
