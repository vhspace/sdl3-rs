use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::Arc;

use crate::get_error;
use crate::pixels;
use crate::rect::Rect;
use crate::render::{BlendMode, Canvas};
use crate::render::{Texture, TextureCreator, TextureValueError};
use crate::sys;
use libc::c_int;
use std::convert::TryFrom;
use std::mem::transmute;
use std::ptr;
use sys::blendmode::SDL_BLENDMODE_NONE;
use sys::surface::{SDL_ScaleMode, SDL_MUSTLOCK, SDL_SCALEMODE_LINEAR};
use crate::iostream::IOStream;

/// Holds a `SDL_Surface`
///
/// When the `SurfaceContext` is dropped, it frees the `SDL_Surface`
///
/// *INTERNAL USE ONLY*
pub struct SurfaceContext<'a> {
    raw: *mut sys::surface::SDL_Surface,
    _marker: PhantomData<&'a ()>,
}

impl Drop for SurfaceContext<'_> {
    #[inline]
    #[doc(alias = "SDL_DestroySurface")]
    fn drop(&mut self) {
        unsafe {
            sys::surface::SDL_DestroySurface(self.raw);
        }
    }
}

/// Holds an `Arc<SurfaceContext>`.
///
/// Note: If a `Surface` goes out of scope but it cloned its context,
/// then the `SDL_Surface` will not be free'd until there are no more references to the `SurfaceContext`.
pub struct Surface<'a> {
    context: Arc<SurfaceContext<'a>>,
}

/// An unsized Surface reference.
///
/// This type is used whenever Surfaces need to be borrowed from the SDL library, without concern
/// for freeing the Surface.
pub struct SurfaceRef {
    // It's nothing! (it gets transmuted to SDL_Surface later).
    // The empty private field is need to a) make `std::mem::swap()` copy nothing instead of
    // clobbering two surfaces (SDL_Surface's size could change in the future),
    // and b) prevent user initialization of this type.
    _raw: (),
}


impl AsRef<SurfaceRef> for SurfaceRef {
    fn as_ref(&self) -> &SurfaceRef {
        self
    }
}

#[test]
fn test_surface_ref_size() {
    // `SurfaceRef` must be 0 bytes.
    assert_eq!(::std::mem::size_of::<SurfaceRef>(), 0);
}

impl Deref for Surface<'_> {
    type Target = SurfaceRef;

    #[inline]
    fn deref(&self) -> &SurfaceRef {
        unsafe { mem::transmute(self.context.raw) }
    }
}

impl DerefMut for Surface<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut SurfaceRef {
        unsafe { mem::transmute(self.context.raw) }
    }
}

impl AsRef<SurfaceRef> for Surface<'_> {
    #[inline]
    fn as_ref(&self) -> &SurfaceRef {
        unsafe { mem::transmute(self.context.raw) }
    }
}

impl AsMut<SurfaceRef> for Surface<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut SurfaceRef {
        unsafe { mem::transmute(self.context.raw) }
    }
}

impl<'a> Surface<'a> {
    pub unsafe fn from_ll<'b>(raw: *mut sys::surface::SDL_Surface) -> Surface<'b> {
        let context = SurfaceContext {
            raw,
            _marker: PhantomData,
        };
        Surface {
            context: Arc::new(context),
        }
    }

    /// Creates a new surface using a pixel format.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::pixels::PixelFormat;
    /// use sdl3::surface::Surface;
    ///
    /// let surface = Surface::new(512, 512, PixelFormat::RGB24).unwrap();
    /// ```
    pub fn new(
        width: u32,
        height: u32,
        format: pixels::PixelFormat,
    ) -> Result<Surface<'static>, String> {
        let masks = format.into_masks()?;
        Surface::from_pixelmasks(width, height, &masks)
    }

    /// Creates a new surface using pixel masks.
    ///
    /// # Example
    /// ```no_run
    /// use sdl3::pixels::PixelFormat;
    /// use sdl3::surface::Surface;
    ///
    /// let masks = PixelFormat::RGB24.into_masks().unwrap();
    /// let surface = Surface::from_pixelmasks(512, 512, &masks).unwrap();
    /// ```
    #[doc(alias = "SDL_CreateSurface")]
    pub fn from_pixelmasks(
        width: u32,
        height: u32,
        masks: &pixels::PixelMasks,
    ) -> Result<Surface<'static>, String> {
        unsafe {
            if width >= (1 << 31) || height >= (1 << 31) {
                Err("Image is too large.".to_owned())
            } else {
                let raw = sys::surface::SDL_CreateSurface(
                    width as c_int,
                    height as c_int,
                    sys::pixels::SDL_GetPixelFormatForMasks(
                        masks.bpp as c_int,
                        masks.rmask,
                        masks.gmask,
                        masks.bmask,
                        masks.amask,
                    ),
                );

                if raw.is_null() {
                    Err(get_error())
                } else {
                    Ok(Surface::from_ll(raw))
                }
            }
        }
    }

    /// Creates a new surface from an existing buffer, using a pixel format.
    pub fn from_data(
        data: &'a mut [u8],
        width: u32,
        height: u32,
        pitch: u32,
        format: pixels::PixelFormat,
    ) -> Result<Surface<'a>, String> {
        let masks = format.into_masks()?;
        Surface::from_data_pixelmasks(data, width, height, pitch, &masks)
    }

    /// Creates a new surface from an existing buffer, using pixel masks.
    #[doc(alias = "SDL_CreateSurfaceFrom")]
    pub fn from_data_pixelmasks(
        data: &'a mut [u8],
        width: u32,
        height: u32,
        pitch: u32,
        masks: &pixels::PixelMasks,
    ) -> Result<Surface<'a>, String> {
        unsafe {
            if width >= (1 << 31) || height >= (1 << 31) {
                Err("Image is too large.".to_owned())
            } else if pitch >= (1 << 31) {
                Err("Pitch is too large.".to_owned())
            } else {
                let raw = sys::surface::SDL_CreateSurfaceFrom(
                    width as c_int,
                    height as c_int,
                    sys::pixels::SDL_GetPixelFormatForMasks(
                        masks.bpp as c_int,
                        masks.rmask,
                        masks.gmask,
                        masks.bmask,
                        masks.amask,
                    ),
                    data.as_mut_ptr() as *mut _,
                    pitch as c_int,
                );

                if raw.is_null() {
                    Err(get_error())
                } else {
                    Ok(Surface::from_ll(raw))
                }
            }
        }
    }

    /// A convenience function for [`TextureCreator::create_texture_from_surface`].
    ///
    /// ```no_run
    /// use sdl3::pixels::PixelFormat;
    /// use sdl3::surface::Surface;
    /// use sdl3::render::{Canvas, Texture};
    /// use sdl3::video::Window;
    ///
    /// // We init systems.
    /// let sdl_context = sdl3::init().expect("failed to init SDL");
    /// let video_subsystem = sdl_context.video().expect("failed to get video context");
    ///
    /// // We create a window.
    /// let window = video_subsystem.window("sdl3 demo", 800, 600)
    ///     .build()
    ///     .expect("failed to build window");
    ///
    /// // We get the canvas from which we can get the `TextureCreator`.
    /// let mut canvas: Canvas<Window> = window.into_canvas();
    /// let texture_creator = canvas.texture_creator();
    ///
    /// let surface = Surface::new(512, 512, PixelFormat::RGB24).unwrap();
    /// let texture = surface.as_texture(&texture_creator).unwrap();
    /// ```
    #[cfg(not(feature = "unsafe_textures"))]
    pub fn as_texture<'b, T>(
        &self,
        texture_creator: &'b TextureCreator<T>,
    ) -> Result<Texture<'b>, TextureValueError> {
        texture_creator.create_texture_from_surface(self)
    }

    /// A convenience function for [`TextureCreator::create_texture_from_surface`].
    ///
    /// ```no_run
    /// use sdl3::pixels::PixelFormat;
    /// use sdl3::surface::Surface;
    /// use sdl3::render::{Canvas, Texture};
    /// use sdl3::video::Window;
    ///
    /// // We init systems.
    /// let sdl_context = sdl3::init().expect("failed to init SDL");
    /// let video_subsystem = sdl_context.video().expect("failed to get video context");
    ///
    /// // We create a window.
    /// let window = video_subsystem.window("sdl3 demo", 800, 600)
    ///     .build()
    ///     .expect("failed to build window");
    ///
    /// // We get the canvas from which we can get the `TextureCreator`.
    /// let mut canvas: Canvas<Window> = window.into_canvas();
    /// let texture_creator = canvas.texture_creator();
    ///
    /// let surface = Surface::new(512, 512, PixelFormat::RGB24).unwrap();
    /// let texture = surface.as_texture(&texture_creator).unwrap();
    /// ```
    #[cfg(feature = "unsafe_textures")]
    pub fn as_texture<T>(
        &self,
        texture_creator: &TextureCreator<T>,
    ) -> Result<Texture, TextureValueError> {
        texture_creator.create_texture_from_surface(self)
    }

    #[doc(alias = "SDL_LoadBMP_RW")]
    pub fn load_bmp_rw(iostream: &mut IOStream) -> Result<Surface<'static>, String> {
        let raw = unsafe { sys::surface::SDL_LoadBMP_IO(iostream.raw(), false) };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(unsafe { Surface::from_ll(raw) })
        }
    }

    pub fn load_bmp<P: AsRef<Path>>(path: P) -> Result<Surface<'static>, String> {
        let mut file = IOStream::from_file(path, "rb")?;
        Surface::load_bmp_rw(&mut file)
    }

    /// Creates a Software Canvas to allow rendering in the Surface itself. This `Canvas` will
    /// never be accelerated materially, so there is no performance change between `Surface` and
    /// `Canvas` coming from a `Surface`.
    ///
    /// The only change is this case is that `Canvas` has a
    /// better API to draw stuff in the `Surface` in that case, but don't expect any performance
    /// changes, there will be none.
    pub fn into_canvas(self) -> Result<Canvas<Surface<'a>>, String> {
        Canvas::from_surface(self)
    }

    pub fn context(&self) -> Arc<SurfaceContext<'a>> {
        self.context.clone()
    }
}

impl SurfaceRef {
    #[inline]
    pub unsafe fn from_ll<'a>(raw: *const sys::surface::SDL_Surface) -> &'a SurfaceRef {
        &*(raw as *const () as *const SurfaceRef)
    }

    #[inline]
    pub unsafe fn from_ll_mut<'a>(raw: *mut sys::surface::SDL_Surface) -> &'a mut SurfaceRef {
        &mut *(raw as *mut () as *mut SurfaceRef)
    }

    #[inline]
    // this can prevent introducing UB until
    // https://github.com/rust-lang/rust-clippy/issues/5953 is fixed
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[doc(alias = "SDL_Surface")]
    pub fn raw(&self) -> *mut sys::surface::SDL_Surface {
        self as *const SurfaceRef as *mut SurfaceRef as *mut () as *mut sys::surface::SDL_Surface
    }

    #[inline]
    fn raw_ref(&self) -> &sys::surface::SDL_Surface {
        unsafe { &*(self as *const _ as *const () as *const sys::surface::SDL_Surface) }
    }

    pub fn width(&self) -> u32 {
        self.raw_ref().w as u32
    }

    pub fn height(&self) -> u32 {
        self.raw_ref().h as u32
    }

    pub fn pitch(&self) -> u32 {
        self.raw_ref().pitch as u32
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// Gets the rect of the surface.
    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width(), self.height())
    }

    pub fn pixel_format(&self) -> pixels::PixelFormat {
        unsafe { pixels::PixelFormat::from_ll(self.raw_ref().format) }
    }

    pub fn pixel_format_enum(&self) -> pixels::PixelFormat {
        self.pixel_format()
    }

    /// Locks a surface so that the pixels can be directly accessed safely.
    #[doc(alias = "SDL_LockSurface")]
    pub fn with_lock<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        unsafe {
            if !sys::surface::SDL_LockSurface(self.raw()) {
                panic!("could not lock surface");
            }

            let raw_pixels = self.raw_ref().pixels as *const _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);
            let pixels = ::std::slice::from_raw_parts(raw_pixels, len);
            let rv = f(pixels);
            sys::surface::SDL_UnlockSurface(self.raw());
            rv
        }
    }

    /// Locks a surface so that the pixels can be directly accessed safely.
    #[doc(alias = "SDL_LockSurface")]
    pub fn with_lock_mut<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> R {
        unsafe {
            if !sys::surface::SDL_LockSurface(self.raw()) {
                panic!("could not lock surface");
            }

            let raw_pixels = self.raw_ref().pixels as *mut _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);
            let pixels = ::std::slice::from_raw_parts_mut(raw_pixels, len);
            let rv = f(pixels);
            sys::surface::SDL_UnlockSurface(self.raw());
            rv
        }
    }

    /// Returns the Surface's pixel buffer if the Surface doesn't require locking
    /// (e.g. it's a software surface).
    pub unsafe fn without_lock(&self) -> Option<&[u8]> {
        if self.must_lock() {
            None
        } else {
            let raw_pixels = self.raw_ref().pixels as *const _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);

            Some(::std::slice::from_raw_parts(raw_pixels, len))
        }
    }

    /// Returns the Surface's pixel buffer if the Surface doesn't require locking
    /// (e.g. it's a software surface).
    pub unsafe fn without_lock_mut(&mut self) -> Option<&mut [u8]> {
        if self.must_lock() {
            None
        } else {
            let raw_pixels = self.raw_ref().pixels as *mut _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);

            Some(::std::slice::from_raw_parts_mut(raw_pixels, len))
        }
    }

    /// Returns true if the Surface needs to be locked before accessing the Surface pixels.
    pub unsafe fn must_lock(&self) -> bool {
        SDL_MUSTLOCK(self.raw_ref())
    }

    #[doc(alias = "SDL_SaveBMP_RW")]
    pub fn save_bmp_rw(&self, iostream: &mut IOStream) -> Result<(), String> {
        let ret = unsafe { sys::surface::SDL_SaveBMP_IO(self.raw(), iostream.raw(), false) };
        if !ret {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    pub fn save_bmp<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut file = IOStream::from_file(path, "wb")?;
        self.save_bmp_rw(&mut file)
    }

    #[doc(alias = "SDL_SetSurfacePalette")]
    pub fn set_palette(&mut self, palette: &pixels::Palette) -> Result<(), String> {
        let result = unsafe { sys::surface::SDL_SetSurfacePalette(self.raw(), palette.raw()) };

        match result {
            true => Ok(()),
            _ => Err(get_error()),
        }
    }

    #[allow(non_snake_case)]
    #[doc(alias = "SDL_SetSurfaceRLE")]
    pub fn enable_RLE(&mut self) {
        let result = unsafe { sys::surface::SDL_SetSurfaceRLE(self.raw(), true) };

        if !result {
            // Should only panic on a null Surface
            panic!("{}", get_error());
        }
    }

    #[allow(non_snake_case)]
    #[doc(alias = "SDL_SetSurfaceRLE")]
    pub fn disable_RLE(&mut self) {
        let result = unsafe { sys::surface::SDL_SetSurfaceRLE(self.raw(), false) };

        if !result {
            // Should only panic on a null Surface
            panic!("{}", get_error());
        }
    }

    #[doc(alias = "SDL_SetSurfaceColorKey")]
    pub fn set_color_key(&mut self, enable: bool, color: pixels::Color) -> Result<(), String> {
        let key = color.to_u32(&self.pixel_format());
        let result = unsafe { sys::surface::SDL_SetSurfaceColorKey(self.raw(), enable, key) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// The function will fail if the surface doesn't have color key enabled.
    #[doc(alias = "SDL_GetSurfaceColorKey")]
    pub fn color_key(&self) -> Result<pixels::Color, String> {
        let mut key = 0;

        // SDL_GetSurfaceColorKey does not mutate, but requires a non-const pointer anyway.

        let result = unsafe { sys::surface::SDL_GetSurfaceColorKey(self.raw(), &mut key) };

        if result {
            Ok(pixels::Color::from_u32(&self.pixel_format(), key))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_SetSurfaceColorMod")]
    pub fn set_color_mod(&mut self, color: pixels::Color) {
        let (r, g, b) = color.rgb();
        let result = unsafe { sys::surface::SDL_SetSurfaceColorMod(self.raw(), r, g, b) };

        if !result {
            // Should only fail on a null Surface
            panic!("{}", get_error());
        }
    }

    #[doc(alias = "SDL_GetSurfaceColorMod")]
    pub fn color_mod(&self) -> pixels::Color {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        // SDL_GetSurfaceColorMod does not mutate, but requires a non-const pointer anyway.

        let result =
            unsafe { sys::surface::SDL_GetSurfaceColorMod(self.raw(), &mut r, &mut g, &mut b) };

        if result {
            pixels::Color::RGB(r, g, b)
        } else {
            // Should only fail on a null Surface
            panic!("{}", get_error())
        }
    }

    #[doc(alias = "SDL_FillSurfaceRect")]
    pub fn fill_rect<R>(&mut self, rect: R, color: pixels::Color) -> Result<(), String>
    where
        R: Into<Option<Rect>>,
    {
        unsafe {
            let rect = rect.into();
            let rect_ptr = mem::transmute(rect.as_ref()); // TODO find a better way to transform
            // Option<&...> into a *const _
            let format = self.pixel_format();
            let result =
                sys::surface::SDL_FillSurfaceRect(self.raw(), rect_ptr, color.to_u32(&format));
            match result {
                true => Ok(()),
                _ => Err(get_error()),
            }
        }
    }

    #[allow(clippy::clone_on_copy)]
    pub fn fill_rects(&mut self, rects: &[Rect], color: pixels::Color) -> Result<(), String> {
        for rect in rects.iter() {
            self.fill_rect(rect.clone(), color)?;
        }

        Ok(())
    }

    #[doc(alias = "SDL_SetSurfaceAlphaMod")]
    pub fn set_alpha_mod(&mut self, alpha: u8) {
        let result = unsafe { sys::surface::SDL_SetSurfaceAlphaMod(self.raw(), alpha) };

        if !result {
            // Should only fail on a null Surface
            panic!("{}", get_error());
        }
    }

    #[doc(alias = "SDL_GetSurfaceAlphaMod")]
    pub fn alpha_mod(&self) -> u8 {
        let mut alpha = 0;
        let result = unsafe { sys::surface::SDL_GetSurfaceAlphaMod(self.raw(), &mut alpha) };

        match result {
            true => alpha,
            // Should only fail on a null Surface
            _ => panic!("{}", get_error()),
        }
    }

    /// The function will fail if the blend mode is not supported by SDL.
    #[doc(alias = "SDL_SetSurfaceBlendMode")]
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> Result<(), String> {
        let result = unsafe { sys::surface::SDL_SetSurfaceBlendMode(self.raw(), transmute(mode)) };

        match result {
            true => Ok(()),
            _ => Err(get_error()),
        }
    }

    #[doc(alias = "SDL_GetSurfaceBlendMode")]
    pub fn blend_mode(&self) -> BlendMode {
        let mut mode = SDL_BLENDMODE_NONE;
        let result = unsafe { sys::surface::SDL_GetSurfaceBlendMode(self.raw(), &mut mode) };

        match result {
            true => BlendMode::try_from(mode).unwrap(),
            // Should only fail on a null Surface
            _ => panic!("{}", get_error()),
        }
    }

    /// Sets the clip rectangle for the surface.
    ///
    /// If the rectangle is `None`, clipping will be disabled.
    #[doc(alias = "SDL_SetSurfaceClipRect")]
    pub fn set_clip_rect<R>(&mut self, rect: R) -> bool
    where
        R: Into<Option<Rect>>,
    {
        let rect = rect.into();
        unsafe {
            sys::surface::SDL_SetSurfaceClipRect(
                self.raw(),
                match rect {
                    Some(rect) => rect.raw(),
                    None => ptr::null(),
                },
            )
        }
    }

    /// Gets the clip rectangle for the surface.
    ///
    /// Returns `None` if clipping is disabled.
    #[doc(alias = "SDL_GetSurfaceClipRect")]
    pub fn clip_rect(&self) -> Option<Rect> {
        let mut raw = mem::MaybeUninit::uninit();
        unsafe { sys::surface::SDL_GetSurfaceClipRect(self.raw(), raw.as_mut_ptr()) };
        let raw = unsafe { raw.assume_init() };

        if raw.w == 0 || raw.h == 0 {
            None
        } else {
            Some(Rect::from_ll(raw))
        }
    }

    /// Copies the surface into a new one that is optimized for blitting to a surface of a specified pixel format.
    #[doc(alias = "SDL_ConvertSurface")]
    pub fn convert(&self, format: &pixels::PixelFormat) -> Result<Surface<'static>, String> {
        // SDL_ConvertSurface takes a flag as the last parameter, which should be 0 by the docs.
        let surface_ptr = unsafe { sys::surface::SDL_ConvertSurface(self.raw(), format.raw()) };

        if surface_ptr.is_null() {
            Err(get_error())
        } else {
            unsafe { Ok(Surface::from_ll(surface_ptr)) }
        }
    }

    /// Copies the surface into a new one of a specified pixel format.
    #[doc(alias = "SDL_ConvertSurfaceFormat")]
    pub fn convert_format(&self, format: pixels::PixelFormat) -> Result<Surface<'static>, String> {
        // SDL_ConvertSurfaceFormat takes a flag as the last parameter, which should be 0 by the docs.
        let surface_ptr = unsafe { sys::surface::SDL_ConvertSurface(self.raw(), format.raw()) };

        if surface_ptr.is_null() {
            Err(get_error())
        } else {
            unsafe { Ok(Surface::from_ll(surface_ptr)) }
        }
    }

    /// Performs surface blitting (surface copying).
    ///
    /// Returns the final blit rectangle, if a `dst_rect` was provided.
    #[doc(alias = "SDL_BlitSurface")]
    pub fn blit<R1, R2>(
        &self,
        src_rect: R1,
        dst: &mut SurfaceRef,
        dst_rect: R2,
    ) -> Result<Option<Rect>, String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let src_rect = src_rect.into();
        let dst_rect = dst_rect.into();

        unsafe {
            let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null());

            // Copy the rect here to make a mutable copy without requiring
            // a mutable argument
            let mut dst_rect = dst_rect;
            let dst_rect_ptr = dst_rect
                .as_mut()
                .map(|r| r.raw_mut())
                .unwrap_or(ptr::null_mut());
            let result =
                sys::surface::SDL_BlitSurface(self.raw(), src_rect_ptr, dst.raw(), dst_rect_ptr);

            if result {
                Ok(dst_rect)
            } else {
                Err(get_error())
            }
        }
    }

    /// Performs low-level surface blitting.
    ///
    /// Unless you know what you're doing, use `blit()` instead, which will clip the input rectangles.
    /// This function could crash if the rectangles aren't pre-clipped to the surface, and is therefore unsafe.
    #[doc(alias = "SDL_BlitSurfaceUnchecked")]
    pub unsafe fn lower_blit<R1, R2>(
        &self,
        src_rect: R1,
        dst: &mut SurfaceRef,
        dst_rect: R2,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let src_rect = src_rect.into();
        let dst_rect = dst_rect.into();

        // The rectangles don't change, but the function requires mutable pointers.
        let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null()) as *mut _;
        let dst_rect_ptr = dst_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null()) as *mut _;
        if sys::surface::SDL_BlitSurfaceUnchecked(
            self.raw(),
            src_rect_ptr,
            dst.raw(),
            dst_rect_ptr,
        ) {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Performs bilinear scaling between two surfaces of the same format, 32BPP.
    ///
    /// Returns the final blit rectangle, if a `dst_rect` was provided.
    #[doc(alias = "SDL_BlitSurfaceScaled")]
    pub unsafe fn soft_stretch_linear<R1, R2>(
        &self,
        src_rect: R1,
        dst: &mut SurfaceRef,
        dst_rect: R2,
    ) -> Result<Option<Rect>, String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let src_rect = src_rect.into();
        let dst_rect = dst_rect.into();

        let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null());

        // Copy the rect here to make a mutable copy without requiring
        // a mutable argument
        let mut dst_rect = dst_rect;
        let dst_rect_ptr = dst_rect
            .as_mut()
            .map(|r| r.raw_mut())
            .unwrap_or(ptr::null_mut());

        let result = unsafe {
            sys::surface::SDL_BlitSurfaceScaled(
                self.raw(),
                src_rect_ptr,
                dst.raw(),
                dst_rect_ptr,
                SDL_SCALEMODE_LINEAR,
            )
        };
        if result {
            Ok(dst_rect)
        } else {
            Err(get_error())
        }
    }

    /// Performs scaled surface bliting (surface copying).
    ///
    /// Returns the final blit rectangle, if a `dst_rect` was provided.
    #[doc(alias = "SDL_BlitSurfaceScaled")]
    pub fn blit_scaled<R1, R2>(
        &self,
        src_rect: R1,
        dst: &mut SurfaceRef,
        dst_rect: R2,
        scale_mode: SDL_ScaleMode,
    ) -> Result<Option<Rect>, String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let src_rect = src_rect.into();
        let dst_rect = dst_rect.into();
    
        let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null());

        // Copy the rect here to make a mutable copy without requiring
        // a mutable argument
        let mut dst_rect = dst_rect;
        let dst_rect_ptr = dst_rect
            .as_mut()
            .map(|r| r.raw_mut())
            .unwrap_or(ptr::null_mut());
        let result = unsafe {
            sys::surface::SDL_BlitSurfaceScaled(
                self.raw(),
                src_rect_ptr,
                dst.raw(),
                dst_rect_ptr,
                scale_mode,
            )
        };
        if result {
            Ok(dst_rect)
        } else {
            Err(get_error())
        }
    }

    /// Performs low-level scaled surface blitting.
    ///
    /// Unless you know what you're doing, use `blit_scaled()` instead, which will clip the input rectangles.
    /// This function could crash if the rectangles aren't pre-clipped to the surface, and is therefore unsafe.
    #[doc(alias = "SDL_BlitSurfaceUncheckedScaled")]
    pub unsafe fn lower_blit_scaled<R1, R2>(
        &self,
        src_rect: R1,
        dst: &mut SurfaceRef,
        dst_rect: R2,
        scale_mode: SDL_ScaleMode,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        // The rectangles don't change, but the function requires mutable pointers.
        let src_rect_ptr = src_rect
            .into()
            .as_ref()
            .map(|r| r.raw())
            .unwrap_or(ptr::null()) as *mut _;
        let dst_rect_ptr = dst_rect
            .into()
            .as_ref()
            .map(|r| r.raw())
            .unwrap_or(ptr::null()) as *mut _;
        if sys::surface::SDL_BlitSurfaceUncheckedScaled(
            self.raw(),
            src_rect_ptr,
            dst.raw(),
            dst_rect_ptr,
            scale_mode,
        ) {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /*
    pub fn SDL_ConvertPixels(width: c_int, height: c_int, src_format: uint32_t, src: *c_void, src_pitch: c_int, dst_format: uint32_t, dst: *c_void, dst_pitch: c_int) -> c_int;
    */
}
