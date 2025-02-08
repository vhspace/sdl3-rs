use crate::common::{validate_int, IntegerOrSdlError};
use crate::get_error;
use crate::pixels::PixelFormat;
use crate::rect::Rect;
use crate::render::{create_renderer, WindowCanvas};
use crate::surface::SurfaceRef;
use crate::Error;
use crate::EventPump;
use crate::VideoSubsystem;
use libc::{c_char, c_int, c_uint, c_void};
use std::convert::TryFrom;
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::{fmt, mem, ptr};
use sys::properties::{
    SDL_CreateProperties, SDL_DestroyProperties, SDL_SetNumberProperty, SDL_SetStringProperty,
};
use sys::stdinc::{SDL_FunctionPointer, SDL_free, Uint64};
use sys::video::{SDL_DisplayMode, SDL_DisplayModeData, SDL_DisplayOrientation, SDL_WindowFlags};

use crate::sys;

pub use crate::sys::vulkan::{VkInstance, VkSurfaceKHR};

pub struct WindowSurfaceRef<'a>(&'a mut SurfaceRef, &'a Window);

impl Deref for WindowSurfaceRef<'_> {
    type Target = SurfaceRef;

    #[inline]
    fn deref(&self) -> &SurfaceRef {
        self.0
    }
}

impl DerefMut for WindowSurfaceRef<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut SurfaceRef {
        self.0
    }
}

impl WindowSurfaceRef<'_> {
    /// Updates the change made to the inner Surface to the Window it was created from.
    ///
    /// This would effectively be the theoretical equivalent of `present` from a Canvas.
    #[doc(alias = "SDL_UpdateWindowSurface")]
    pub fn update_window(&self) -> Result<(), Error> {
        unsafe {
            if sys::video::SDL_UpdateWindowSurface(self.1.context.raw) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Same as `update_window`, but only update the parts included in `rects` to the Window it was
    /// created from.
    #[doc(alias = "SDL_UpdateWindowSurfaceRects")]
    pub fn update_window_rects(&self, rects: &[Rect]) -> Result<(), Error> {
        unsafe {
            if sys::video::SDL_UpdateWindowSurfaceRects(
                self.1.context.raw,
                Rect::raw_slice(rects),
                rects.len() as c_int,
            ) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Gives up this WindowSurfaceRef, allowing to use the window freely again. Before being
    /// destroyed, calls `update_window` one last time.
    ///
    /// If you don't want to `update_window` one last time, simply Drop this struct. However
    /// beware, since the Surface will still be in the state you left it the next time you will
    /// call `window.surface()` again.
    pub fn finish(self) -> Result<(), Error> {
        self.update_window()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GLProfile {
    /// OpenGL core profile - deprecated functions are disabled
    Core,
    /// OpenGL compatibility profile - deprecated functions are allowed
    Compatibility,
    /// OpenGL ES profile - only a subset of the base OpenGL functionality is available
    GLES,
    /// Unknown profile - SDL will tend to return 0 if you ask when no particular profile
    /// has been defined or requested.
    Unknown(i32),
}

trait GLAttrTypeUtil {
    fn to_gl_value(self) -> i32;
    fn from_gl_value(value: i32) -> Self;
}

impl GLAttrTypeUtil for u8 {
    fn to_gl_value(self) -> i32 {
        self as i32
    }
    fn from_gl_value(value: i32) -> u8 {
        value as u8
    }
}

impl GLAttrTypeUtil for bool {
    fn to_gl_value(self) -> i32 {
        if self {
            1
        } else {
            0
        }
    }
    fn from_gl_value(value: i32) -> bool {
        value != 0
    }
}

impl GLAttrTypeUtil for GLProfile {
    fn to_gl_value(self) -> i32 {
        use self::GLProfile::*;

        match self {
            Unknown(i) => i,
            Core => 1,
            Compatibility => 2,
            GLES => 4,
        }
    }
    fn from_gl_value(value: i32) -> GLProfile {
        use self::GLProfile::*;

        match value {
            1 => Core,
            2 => Compatibility,
            4 => GLES,
            i => Unknown(i),
        }
    }
}

macro_rules! gl_attr {
    ($($attr_name:ident, $set_property:ident, $get_property:ident, $t:ty, $doc:expr);* $(;)?) => {
        $(
        #[doc = "**Sets** the attribute: "]
        #[doc = $doc]
        #[inline]
        pub fn $set_property(&self, value: $t) {
            gl_set_attribute!($attr_name, value.to_gl_value());
        }

        #[doc = "**Gets** the attribute: "]
        #[doc = $doc]
        #[inline]
        pub fn $get_property(&self) -> $t {
            let value = gl_get_attribute!($attr_name);
            GLAttrTypeUtil::from_gl_value(value)
        }
        )*
    };
}

/// OpenGL context getters and setters
///
/// # Example
/// ```no_run
/// use sdl3::video::GLProfile;
///
/// let sdl_context = sdl3::init().unwrap();
/// let video_subsystem = sdl_context.video().unwrap();
/// let gl_attr = video_subsystem.gl_attr();
///
/// // Don't use deprecated OpenGL functions
/// gl_attr.set_context_profile(GLProfile::Core);
///
/// // Set the context into debug mode
/// gl_attr.set_context_flags().debug().set();
///
/// // Set the OpenGL context version (OpenGL 3.2)
/// gl_attr.set_context_version(3, 2);
///
/// // Enable anti-aliasing
/// gl_attr.set_multisample_buffers(1);
/// gl_attr.set_multisample_samples(4);
///
/// let window = video_subsystem.window("sdl3 demo: Video", 800, 600).build().unwrap();
///
/// // Yes, we're still using the Core profile
/// assert_eq!(gl_attr.context_profile(), GLProfile::Core);
/// // ... and we're still using OpenGL 3.2
/// assert_eq!(gl_attr.context_version(), (3, 2));
/// ```
pub mod gl_attr {
    use super::{GLAttrTypeUtil, GLProfile};
    use crate::get_error;
    use crate::sys;
    use std::marker::PhantomData;

    /// OpenGL context getters and setters. Obtain with `VideoSubsystem::gl_attr()`.
    pub struct GLAttr<'a> {
        _marker: PhantomData<&'a crate::VideoSubsystem>,
    }

    impl crate::VideoSubsystem {
        /// Obtains access to the OpenGL window attributes.
        pub fn gl_attr(&self) -> GLAttr {
            GLAttr {
                _marker: PhantomData,
            }
        }
    }

    macro_rules! gl_set_attribute {
        ($attr:ident, $value:expr) => {{
            let result =
                unsafe { sys::video::SDL_GL_SetAttribute(sys::video::SDL_GLAttr::$attr, $value) };

            if !result {
                // Panic and print the attribute that failed.
                panic!(
                    "couldn't set attribute {}: {}",
                    stringify!($attr),
                    get_error()
                );
            }
        }};
    }

    macro_rules! gl_get_attribute {
        ($attr:ident) => {{
            let mut value = 0;
            let result = unsafe {
                sys::video::SDL_GL_GetAttribute(sys::video::SDL_GLAttr::$attr, &mut value)
            };
            if !result {
                // Panic and print the attribute that failed.
                panic!(
                    "couldn't get attribute {}: {}",
                    stringify!($attr),
                    get_error()
                );
            }
            value
        }};
    }

    impl GLAttr<'_> {
        gl_attr! {
            RED_SIZE, set_red_size, red_size, u8, "the minimum number of bits for the red channel of the color buffer; defaults to 3";
            GREEN_SIZE, set_green_size, green_size, u8, "the minimum number of bits for the green channel of the color buffer; defaults to 3";
            BLUE_SIZE, set_blue_size, blue_size, u8, "the minimum number of bits for the blue channel of the color buffer; defaults to 2";
            ALPHA_SIZE, set_alpha_size, alpha_size, u8, "the minimum number of bits for the alpha channel of the color buffer; defaults to 0";
            BUFFER_SIZE, set_buffer_size, buffer_size, u8, "the minimum number of bits for frame buffer size; defaults to 0";
            DOUBLEBUFFER, set_double_buffer, double_buffer, bool, "whether the output is single or double buffered; defaults to double buffering on";
            DEPTH_SIZE, set_depth_size, depth_size, u8, "the minimum number of bits in the depth buffer; defaults to 16";
            STENCIL_SIZE, set_stencil_size, stencil_size, u8, "the minimum number of bits in the stencil buffer; defaults to 0";
            ACCUM_RED_SIZE, set_accum_red_size, accum_red_size, u8, "the minimum number of bits for the red channel of the accumulation buffer; defaults to 0";
            ACCUM_GREEN_SIZE, set_accum_green_size, accum_green_size, u8, "the minimum number of bits for the green channel of the accumulation buffer; defaults to 0";
            ACCUM_BLUE_SIZE, set_accum_blue_size, accum_blue_size, u8, "the minimum number of bits for the blue channel of the accumulation buffer; defaults to 0";
            ACCUM_ALPHA_SIZE, set_accum_alpha_size, accum_alpha_size, u8, "the minimum number of bits for the alpha channel of the accumulation buffer; defaults to 0";
            STEREO, set_stereo, stereo, bool, "whether the output is stereo 3D; defaults to off";
            MULTISAMPLEBUFFERS, set_multisample_buffers, multisample_buffers, u8, "the number of buffers used for multisample anti-aliasing; defaults to 0";
            MULTISAMPLESAMPLES, set_multisample_samples, multisample_samples, u8, "the number of samples used around the current pixel used for multisample anti-aliasing; defaults to 0";
            ACCELERATED_VISUAL, set_accelerated_visual, accelerated_visual, bool, "whether to require hardware acceleration; false to force software rendering; defaults to allow either";
            CONTEXT_MAJOR_VERSION, set_context_major_version, context_major_version, u8, "OpenGL context major version";
            CONTEXT_MINOR_VERSION, set_context_minor_version, context_minor_version, u8, "OpenGL context minor version";
            CONTEXT_PROFILE_MASK, set_context_profile, context_profile, GLProfile, "type of GL context (Core, Compatibility, ES)";
            SHARE_WITH_CURRENT_CONTEXT, set_share_with_current_context, share_with_current_context, bool, "OpenGL context sharing; defaults to false";
            FRAMEBUFFER_SRGB_CAPABLE, set_framebuffer_srgb_compatible, framebuffer_srgb_compatible, bool, "requests sRGB capable visual; defaults to false (>= SDL 2.0.1)";
            CONTEXT_NO_ERROR, set_context_no_error, context_no_error, bool, "disables OpenGL error checking; defaults to false (>= SDL 2.0.6)";
        }

        /// **Sets** the OpenGL context major and minor versions.
        #[inline]
        pub fn set_context_version(&self, major: u8, minor: u8) {
            self.set_context_major_version(major);
            self.set_context_minor_version(minor);
        }

        /// **Gets** the OpenGL context major and minor versions as a tuple.
        #[inline]
        pub fn context_version(&self) -> (u8, u8) {
            (self.context_major_version(), self.context_minor_version())
        }
    }

    /// The type that allows you to build a OpenGL context configuration.
    pub struct ContextFlagsBuilder<'a> {
        flags: i32,
        _marker: PhantomData<&'a crate::VideoSubsystem>,
    }

    impl<'a> ContextFlagsBuilder<'a> {
        /// Finishes the builder and applies the GL context flags to the GL context.
        #[inline]
        pub fn set(&self) {
            gl_set_attribute!(CONTEXT_FLAGS, self.flags);
        }

        /// Sets the context into "debug" mode.
        #[inline]
        pub fn debug(&mut self) -> &mut ContextFlagsBuilder<'a> {
            self.flags |= 0x0001;
            self
        }

        /// Sets the context into "forward compatible" mode.
        #[inline]
        pub fn forward_compatible(&mut self) -> &mut ContextFlagsBuilder<'a> {
            self.flags |= 0x0002;
            self
        }

        #[inline]
        pub fn robust_access(&mut self) -> &mut ContextFlagsBuilder<'a> {
            self.flags |= 0x0004;
            self
        }

        #[inline]
        pub fn reset_isolation(&mut self) -> &mut ContextFlagsBuilder<'a> {
            self.flags |= 0x0008;
            self
        }
    }

    pub struct ContextFlags {
        flags: i32,
    }

    impl ContextFlags {
        #[inline]
        pub const fn has_debug(&self) -> bool {
            self.flags & 0x0001 != 0
        }

        #[inline]
        pub const fn has_forward_compatible(&self) -> bool {
            self.flags & 0x0002 != 0
        }

        #[inline]
        pub const fn has_robust_access(&self) -> bool {
            self.flags & 0x0004 != 0
        }

        #[inline]
        pub const fn has_reset_isolation(&self) -> bool {
            self.flags & 0x0008 != 0
        }
    }

    impl GLAttr<'_> {
        /// **Sets** any combination of OpenGL context configuration flags.
        ///
        /// Note that calling this will reset any existing context flags.
        ///
        /// # Example
        /// ```no_run
        /// let sdl_context = sdl3::init().unwrap();
        /// let video_subsystem = sdl_context.video().unwrap();
        /// let gl_attr = video_subsystem.gl_attr();
        ///
        /// // Sets the GL context into debug mode.
        /// gl_attr.set_context_flags().debug().set();
        /// ```
        pub fn set_context_flags(&self) -> ContextFlagsBuilder {
            ContextFlagsBuilder {
                flags: 0,
                _marker: PhantomData,
            }
        }

        /// **Gets** the applied OpenGL context configuration flags.
        ///
        /// # Example
        /// ```no_run
        /// let sdl_context = sdl3::init().unwrap();
        /// let video_subsystem = sdl_context.video().unwrap();
        /// let gl_attr = video_subsystem.gl_attr();
        ///
        /// // Is the GL context in debug mode?
        /// if gl_attr.context_flags().has_debug() {
        ///     println!("Debug mode");
        /// }
        /// ```
        pub fn context_flags(&self) -> ContextFlags {
            let flags = gl_get_attribute!(CONTEXT_FLAGS);

            ContextFlags { flags }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DisplayMode {
    pub display_id: sys::video::SDL_DisplayID,
    pub format: PixelFormat,
    pub w: i32,
    pub h: i32,
    pub pixel_density: f32,
    pub refresh_rate: f32,
    pub refresh_rate_numerator: i32,
    pub refresh_rate_denominator: i32,
    internal: *mut SDL_DisplayModeData,
}

impl DisplayMode {
    pub fn new(
        display_id: sys::video::SDL_DisplayID,
        format: PixelFormat,
        w: i32,
        h: i32,
        pixel_density: f32,
        refresh_rate: f32,
        refresh_rate_numerator: i32,
        refresh_rate_denominator: i32,
        internal: *mut SDL_DisplayModeData,
    ) -> DisplayMode {
        DisplayMode {
            display_id,
            format,
            w,
            h,
            pixel_density,
            refresh_rate,
            refresh_rate_numerator,
            refresh_rate_denominator,
            internal,
        }
    }

    pub unsafe fn from_ll(raw: &SDL_DisplayMode) -> DisplayMode {
        DisplayMode::new(
            raw.displayID,
            PixelFormat::try_from(raw.format).unwrap_or(PixelFormat::unknown()),
            raw.w,
            raw.h,
            raw.pixel_density,
            raw.refresh_rate,
            raw.refresh_rate_numerator,
            raw.refresh_rate_denominator,
            raw.internal,
        )
    }

    pub fn to_ll(&self) -> SDL_DisplayMode {
        SDL_DisplayMode {
            displayID: self.display_id,
            format: self.format.into(),
            w: self.w,
            h: self.h,
            pixel_density: self.pixel_density,
            refresh_rate: self.refresh_rate,
            refresh_rate_numerator: self.refresh_rate_numerator,
            refresh_rate_denominator: self.refresh_rate_denominator,
            internal: self.internal,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FullscreenType {
    Off = 0,
    True = 0x00_00_00_01,
    Desktop = 0x00_00_10_01,
}

impl FullscreenType {
    pub fn from_window_flags(window_flags: u32) -> FullscreenType {
        if window_flags & FullscreenType::Desktop as u32 == FullscreenType::Desktop as u32 {
            FullscreenType::Desktop
        } else if window_flags & FullscreenType::True as u32 == FullscreenType::True as u32 {
            FullscreenType::True
        } else {
            FullscreenType::Off
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WindowPos {
    Undefined,
    Centered,
    Positioned(i32),
}

impl From<i32> for WindowPos {
    fn from(pos: i32) -> Self {
        WindowPos::Positioned(pos)
    }
}

fn to_ll_windowpos(pos: WindowPos) -> c_int {
    match pos {
        WindowPos::Undefined => sys::video::SDL_WINDOWPOS_UNDEFINED_MASK as c_int,
        WindowPos::Centered => sys::video::SDL_WINDOWPOS_CENTERED_MASK as c_int,
        WindowPos::Positioned(x) => x as c_int,
    }
}

pub struct GLContext {
    raw: sys::video::SDL_GLContext,
}

impl Drop for GLContext {
    #[doc(alias = "SDL_GL_DeleteContext")]
    fn drop(&mut self) {
        unsafe {
            sys::video::SDL_GL_DestroyContext(self.raw);
        }
    }
}

impl GLContext {
    /// Returns true if the OpenGL context is the current one in the thread.
    #[doc(alias = "SDL_GL_GetCurrentContext")]
    pub fn is_current(&self) -> bool {
        let current_raw = unsafe { sys::video::SDL_GL_GetCurrentContext() };
        self.raw == current_raw
    }
}

/// Holds a `SDL_Window`
///
/// When the `WindowContext` is dropped, it destroys the `SDL_Window`
pub struct WindowContext {
    subsystem: VideoSubsystem,
    raw: *mut sys::video::SDL_Window,
    pub(crate) metal_view: sys::metal::SDL_MetalView,
}

impl Drop for WindowContext {
    #[inline]
    #[doc(alias = "SDL_DestroyWindow")]
    fn drop(&mut self) {
        unsafe {
            if !self.metal_view.is_null() {
                sys::metal::SDL_Metal_DestroyView(self.metal_view);
            }
            sys::video::SDL_DestroyWindow(self.raw)
        };
    }
}

impl WindowContext {
    #[inline]
    /// Unsafe if the `*mut SDL_Window` is used after the `WindowContext` is dropped
    pub unsafe fn from_ll(
        subsystem: VideoSubsystem,
        raw: *mut sys::video::SDL_Window,
        metal_view: sys::metal::SDL_MetalView,
    ) -> WindowContext {
        WindowContext {
            subsystem: subsystem.clone(),
            raw,
            metal_view,
        }
    }
}

/// Represents a setting for vsync/swap interval.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum SwapInterval {
    Immediate = 0,
    VSync = 1,
    LateSwapTearing = -1,
}

impl From<i32> for SwapInterval {
    fn from(i: i32) -> Self {
        match i {
            -1 => SwapInterval::LateSwapTearing,
            0 => SwapInterval::Immediate,
            1 => SwapInterval::VSync,
            other => panic!(
                "Invalid value for SwapInterval: {}; valid values are -1, 0, 1",
                other
            ),
        }
    }
}

/// Represents orientation of a display.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum Orientation {
    /// The display orientation can’t be determined
    Unknown = sys::video::SDL_DisplayOrientation::UNKNOWN.0,
    /// The display is in landscape mode, with the right side up, relative to portrait mode
    Landscape = sys::video::SDL_DisplayOrientation::LANDSCAPE.0,
    /// The display is in landscape mode, with the left side up, relative to portrait mode
    LandscapeFlipped = sys::video::SDL_DisplayOrientation::LANDSCAPE_FLIPPED.0,
    /// The display is in portrait mode
    Portrait = sys::video::SDL_DisplayOrientation::PORTRAIT.0,
    /// The display is in portrait mode, upside down
    PortraitFlipped = sys::video::SDL_DisplayOrientation::PORTRAIT_FLIPPED.0,
}

impl Orientation {
    pub fn from_ll(orientation: sys::video::SDL_DisplayOrientation) -> Orientation {
        match orientation {
            sys::video::SDL_DisplayOrientation::UNKNOWN => Orientation::Unknown,
            sys::video::SDL_DisplayOrientation::LANDSCAPE => Orientation::Landscape,
            sys::video::SDL_DisplayOrientation::LANDSCAPE_FLIPPED => Orientation::LandscapeFlipped,
            sys::video::SDL_DisplayOrientation::PORTRAIT => Orientation::Portrait,
            sys::video::SDL_DisplayOrientation::PORTRAIT_FLIPPED => Orientation::PortraitFlipped,
            _ => Orientation::Unknown,
        }
    }

    pub fn to_ll(self) -> sys::video::SDL_DisplayOrientation {
        match self {
            Orientation::Unknown => sys::video::SDL_ORIENTATION_UNKNOWN,
            Orientation::Landscape => sys::video::SDL_ORIENTATION_LANDSCAPE,
            Orientation::LandscapeFlipped => sys::video::SDL_ORIENTATION_LANDSCAPE_FLIPPED,
            Orientation::Portrait => sys::video::SDL_ORIENTATION_PORTRAIT,
            Orientation::PortraitFlipped => sys::video::SDL_ORIENTATION_PORTRAIT_FLIPPED,
        }
    }
}

/// Represents a setting for a window flash operation.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum FlashOperation {
    /// Cancel any window flash state
    Cancel = sys::video::SDL_FlashOperation::CANCEL.0,
    /// Flash the window briefly to get attention
    Briefly = sys::video::SDL_FlashOperation::BRIEFLY.0,
    /// Flash the window until it gets focus
    UntilFocused = sys::video::SDL_FlashOperation::UNTIL_FOCUSED.0,
}

impl FlashOperation {
    pub fn from_ll(flash_operation: sys::video::SDL_FlashOperation) -> FlashOperation {
        match flash_operation {
            sys::video::SDL_FlashOperation::CANCEL => FlashOperation::Cancel,
            sys::video::SDL_FlashOperation::BRIEFLY => FlashOperation::Briefly,
            sys::video::SDL_FlashOperation::UNTIL_FOCUSED => FlashOperation::UntilFocused,
            _ => FlashOperation::Cancel,
        }
    }

    pub fn to_ll(self) -> sys::video::SDL_FlashOperation {
        match self {
            FlashOperation::Cancel => sys::video::SDL_FLASH_CANCEL,
            FlashOperation::Briefly => sys::video::SDL_FLASH_BRIEFLY,
            FlashOperation::UntilFocused => sys::video::SDL_FLASH_UNTIL_FOCUSED,
        }
    }
}

/// Represents the "shell" of a `Window`.
///
/// You can set get and set many of the `SDL_Window` properties (i.e., border, size, `PixelFormat`, etc)
///
/// However, you cannot directly access the pixels of the `Window`.
/// It needs to be converted to a `Canvas` to access the rendering functions.
///
/// Note: If a `Window` goes out of scope but it cloned its context,
/// then the `SDL_Window` will not be destroyed until there are no more references to the `WindowContext`.
/// This may happen when a `TextureCreator<Window>` outlives the `Canvas<Window>`
#[derive(Clone)]
pub struct Window {
    context: Arc<WindowContext>, // Arc may not be needed, added because wgpu expects Window to be send/sync, though even with Arc this technically still isn't send/sync
}

impl From<WindowContext> for Window {
    fn from(context: WindowContext) -> Window {
        Window {
            context: Arc::new(context),
        }
    }
}

impl_raw_accessors!((GLContext, sys::video::SDL_GLContext));

impl VideoSubsystem {
    /// Initializes a new `WindowBuilder`; a convenience method that calls `WindowBuilder::new()`.
    pub fn window(&self, title: &str, width: u32, height: u32) -> WindowBuilder {
        WindowBuilder::new(self, title, width, height)
    }

    /// Create a window with a renderer.
    #[doc(alias = "SDL_CreateWindowAndRenderer")]
    pub fn window_and_renderer(
        &self,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<WindowCanvas, Error> {
        let mut sdl_window = null_mut();
        let mut renderer = null_mut();
        let result = unsafe {
            sys::render::SDL_CreateWindowAndRenderer(
                title.as_ptr() as *const c_char,
                width as c_int,
                height as c_int,
                0,
                &mut sdl_window,
                &mut renderer,
            )
        };
        if !result {
            return Err(get_error());
        }
        // do we need to add an option to create a metal view here?
        let window =
            unsafe { Window::from_ll(self.clone(), sdl_window, 0 as sys::metal::SDL_MetalView) };

        Ok(WindowCanvas::from_window_and_renderer(window, renderer))
    }

    /// Initializes a new `PopupWindowBuilder`; a convenience method that calls `PopupWindowBuilder::new()`.
    pub unsafe fn popup_window(
        &self,
        window: &Window,
        width: u32,
        height: u32,
    ) -> PopupWindowBuilder {
        PopupWindowBuilder::new(self, window, width, height)
    }

    #[doc(alias = "SDL_GetCurrentVideoDriver")]
    pub fn current_video_driver(&self) -> &'static str {
        use std::str;

        unsafe {
            let buf = sys::video::SDL_GetCurrentVideoDriver();
            assert!(!buf.is_null());

            str::from_utf8(CStr::from_ptr(buf as *const _).to_bytes()).unwrap()
        }
    }

    #[doc(alias = "SDL_GetNumVideoDrivers")]
    pub fn num_video_drivers(&self) -> Result<i32, Error> {
        let result = unsafe { sys::video::SDL_GetNumVideoDrivers() };
        if result < 0 {
            Err(get_error())
        } else {
            Ok(result as i32)
        }
    }

    #[doc(alias = "SDL_GetDisplays")]
    pub fn displays(&self) -> Result<Vec<sys::video::SDL_DisplayID>, String> {
        unsafe {
            let mut count: c_int = 0;
            let displays_ptr = sys::video::SDL_GetDisplays(&mut count);
            if displays_ptr.is_null() {
                return Err(get_error());
            }

            let displays_slice = std::slice::from_raw_parts(displays_ptr, count as usize);
            let displays_vec = displays_slice.to_vec();
            SDL_free(displays_ptr as *mut c_void);

            Ok(displays_vec)
        }
    }

    /// Get the name of the display at the index `display_name`.
    ///
    /// Will return an error if the index is out of bounds or if SDL experienced a failure; inspect
    /// the returned string for further info.
    #[doc(alias = "SDL_GetDisplayName")]
    pub fn display_name(&self, display_index: u32) -> Result<String, Error> {
        unsafe {
            let display = sys::video::SDL_GetDisplayName(display_index);
            if display.is_null() {
                Err(get_error())
            } else {
                Ok(CStr::from_ptr(display as *const _)
                    .to_str()
                    .unwrap()
                    .to_owned())
            }
        }
    }

    #[doc(alias = "SDL_GetDisplayBounds")]
    pub fn display_bounds(&self, display_index: u32) -> Result<Rect, Error> {
        let mut out = mem::MaybeUninit::uninit();
        let result = unsafe { sys::video::SDL_GetDisplayBounds(display_index, out.as_mut_ptr()) };

        if result {
            let out = unsafe { out.assume_init() };
            Ok(Rect::from_ll(out))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_GetDisplayUsableBounds")]
    pub fn display_usable_bounds(&self, display_index: u32) -> Result<Rect, Error> {
        let mut out = mem::MaybeUninit::uninit();
        let result =
            unsafe { sys::video::SDL_GetDisplayUsableBounds(display_index, out.as_mut_ptr()) };
        if result {
            let out = unsafe { out.assume_init() };
            Ok(Rect::from_ll(out))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_GetFullscreenDisplayModes")]
    pub fn display_modes(
        &self,
        display_id: sys::video::SDL_DisplayID,
    ) -> Result<Vec<DisplayMode>, Error> {
        unsafe {
            let mut num_modes: c_int = 0;
            let modes = sys::video::SDL_GetFullscreenDisplayModes(display_id, &mut num_modes);
            // modes is a pointer to an array of DisplayMode
            // num_modes is the number of DisplayMode in the array
            if modes.is_null() {
                Err(get_error())
            } else {
                let mut result = Vec::with_capacity(num_modes as usize);
                for i in 0..num_modes {
                    let mode = *modes.offset(i as isize);
                    result.push(DisplayMode::from_ll(&*mode));
                }
                SDL_free(modes as *mut c_void);
                Ok(result)
            }
        }
    }

    #[doc(alias = "SDL_GetDesktopDisplayMode")]
    pub fn desktop_display_mode(&self, display_index: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let raw_mode = sys::video::SDL_GetDesktopDisplayMode(display_index);
            if raw_mode.is_null() {
                return Err(get_error());
            }
            Ok(DisplayMode::from_ll(&*raw_mode))
        }
    }

    /// Get primary display ID.
    #[doc(alias = "SDL_GetPrimaryDisplay")]
    pub fn get_primary_display_id(&self) -> sys::video::SDL_DisplayID {
        unsafe { sys::video::SDL_GetPrimaryDisplay() }
    }

    #[doc(alias = "SDL_GetCurrentDisplayMode")]
    pub fn current_display_mode(&self, display_index: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let raw_mode = sys::video::SDL_GetCurrentDisplayMode(display_index);
            if raw_mode.is_null() {
                return Err(get_error());
            }
            Ok(DisplayMode::from_ll(&*raw_mode))
        }
    }

    #[doc(alias = "SDL_GetClosestFullscreenDisplayMode")]
    pub fn closest_display_mode(
        &self,
        display_index: u32,
        mode: &DisplayMode,
        include_high_density_modes: bool,
    ) -> Result<DisplayMode, Error> {
        unsafe {
            // Allocate uninitialized memory for SDL_DisplayMode
            let mut mode_out = std::mem::MaybeUninit::<sys::video::SDL_DisplayMode>::uninit();

            // Call the SDL function, passing a pointer to the uninitialized memory
            let ok = sys::video::SDL_GetClosestFullscreenDisplayMode(
                display_index,
                mode.w,
                mode.h,
                mode.refresh_rate,
                include_high_density_modes,
                mode_out.as_mut_ptr(),
            );

            if !ok {
                Err(get_error())
            } else {
                // Now it's safe to assume the memory is initialized
                let mode_out = mode_out.assume_init();
                Ok(DisplayMode::from_ll(&mode_out))
            }
        }
    }

    /// Return orientation of a display or Unknown if orientation could not be determined.
    #[doc(alias = "SDL_GetDisplayOrientation")]
    pub fn display_orientation(&self, display_index: u32) -> SDL_DisplayOrientation {
        unsafe { sys::video::SDL_GetCurrentDisplayOrientation(display_index) }
    }

    #[doc(alias = "SDL_ScreenSaverEnabled")]
    pub fn is_screen_saver_enabled(&self) -> bool {
        unsafe { sys::video::SDL_ScreenSaverEnabled() }
    }

    #[doc(alias = "SDL_EnableScreenSaver")]
    pub fn enable_screen_saver(&self) {
        unsafe { sys::video::SDL_EnableScreenSaver() };
    }

    #[doc(alias = "SDL_DisableScreenSaver")]
    pub fn disable_screen_saver(&self) {
        unsafe { sys::video::SDL_DisableScreenSaver() };
    }

    /// Loads the default OpenGL library.
    ///
    /// This should be done after initializing the video driver, but before creating any OpenGL windows.
    /// If no OpenGL library is loaded, the default library will be loaded upon creation of the first OpenGL window.
    ///
    /// If a different library is already loaded, this function will return an error.
    #[doc(alias = "SDL_GL_LoadLibrary")]
    pub fn gl_load_library_default(&self) -> Result<(), Error> {
        unsafe {
            if sys::video::SDL_GL_LoadLibrary(ptr::null()) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Loads the OpenGL library using a platform-dependent OpenGL library name (usually a file path).
    ///
    /// This should be done after initializing the video driver, but before creating any OpenGL windows.
    /// If no OpenGL library is loaded, the default library will be loaded upon creation of the first OpenGL window.
    ///
    /// If a different library is already loaded, this function will return an error.
    #[doc(alias = "SDL_GL_LoadLibrary")]
    pub fn gl_load_library<P: AsRef<::std::path::Path>>(&self, path: P) -> Result<(), Error> {
        unsafe {
            // TODO: use OsStr::to_cstring() once it's stable
            let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
            if sys::video::SDL_GL_LoadLibrary(path.as_ptr() as *const c_char) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Unloads the current OpenGL library.
    ///
    /// To completely unload the library, this should be called for every successful load of the
    /// OpenGL library.
    #[doc(alias = "SDL_GL_UnloadLibrary")]
    pub fn gl_unload_library(&self) {
        unsafe {
            sys::video::SDL_GL_UnloadLibrary();
        }
    }

    /// Gets the pointer to the named OpenGL function.
    ///
    /// This is useful for OpenGL wrappers such as [`gl-rs`](https://github.com/bjz/gl-rs).
    #[doc(alias = "SDL_GL_GetProcAddress")]
    pub fn gl_get_proc_address(&self, procname: &str) -> SDL_FunctionPointer {
        match CString::new(procname) {
            Ok(procname) => unsafe {
                sys::video::SDL_GL_GetProcAddress(procname.as_ptr() as *const c_char)
            },
            // string contains a nul byte - it won't match anything.
            Err(_) => None,
        }
    }

    #[doc(alias = "SDL_GL_ExtensionSupported")]
    pub fn gl_extension_supported(&self, extension: &str) -> bool {
        match CString::new(extension) {
            Ok(extension) => unsafe {
                sys::video::SDL_GL_ExtensionSupported(extension.as_ptr() as *const c_char)
            },
            // string contains a nul byte - it won't match anything.
            Err(_) => false,
        }
    }

    #[doc(alias = "SDL_GL_GetCurrentWindow")]
    pub fn gl_get_current_window_id(&self) -> Result<u32, Error> {
        let raw = unsafe { sys::video::SDL_GL_GetCurrentWindow() };
        if raw.is_null() {
            Err(get_error())
        } else {
            let id = unsafe { sys::video::SDL_GetWindowID(raw) };
            Ok(id)
        }
    }

    /// Releases the thread's current OpenGL context, i.e. sets the current OpenGL context to nothing.
    #[doc(alias = "SDL_GL_MakeCurrent")]
    pub fn gl_release_current_context(&self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_GL_MakeCurrent(ptr::null_mut(), ptr::null_mut()) };

        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_GL_SetSwapInterval")]
    pub fn gl_set_swap_interval<S: Into<SwapInterval>>(&self, interval: S) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_GL_SetSwapInterval(interval.into() as c_int) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_GL_GetSwapInterval")]
    pub fn gl_get_swap_interval(&self) -> Result<SwapInterval, Error> {
        unsafe {
            let mut interval = 0;
            let result = sys::video::SDL_GL_GetSwapInterval(&mut interval);
            if result {
                Ok(SwapInterval::from(interval))
            } else {
                Err(get_error())
            }
        }
    }

    /// Loads the default Vulkan library.
    ///
    /// This should be done after initializing the video driver, but before creating any Vulkan windows.
    /// If no Vulkan library is loaded, the default library will be loaded upon creation of the first Vulkan window.
    ///
    /// If a different library is already loaded, this function will return an error.
    #[doc(alias = "SDL_Vulkan_LoadLibrary")]
    pub fn vulkan_load_library_default(&self) -> Result<(), Error> {
        unsafe {
            if sys::vulkan::SDL_Vulkan_LoadLibrary(ptr::null()) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Loads the Vulkan library using a platform-dependent Vulkan library name (usually a file path).
    ///
    /// This should be done after initializing the video driver, but before creating any Vulkan windows.
    /// If no Vulkan library is loaded, the default library will be loaded upon creation of the first Vulkan window.
    ///
    /// If a different library is already loaded, this function will return an error.
    #[doc(alias = "SDL_Vulkan_LoadLibrary")]
    pub fn vulkan_load_library<P: AsRef<::std::path::Path>>(&self, path: P) -> Result<(), Error> {
        unsafe {
            // TODO: use OsStr::to_cstring() once it's stable
            let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
            if sys::vulkan::SDL_Vulkan_LoadLibrary(path.as_ptr() as *const c_char) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Unloads the current Vulkan library.
    ///
    /// To completely unload the library, this should be called for every successful load of the
    /// Vulkan library.
    #[doc(alias = "SDL_Vulkan_UnloadLibrary")]
    pub fn vulkan_unload_library(&self) {
        unsafe {
            sys::vulkan::SDL_Vulkan_UnloadLibrary();
        }
    }

    /// Gets the pointer to the
    /// [`vkGetInstanceProcAddr`](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkGetInstanceProcAddr.html)
    /// Vulkan function. This function can be called to retrieve the address of other Vulkan
    /// functions.
    #[doc(alias = "SDL_Vulkan_GetVkGetInstanceProcAddr")]
    pub fn vulkan_get_proc_address_function(&self) -> SDL_FunctionPointer {
        unsafe { sys::vulkan::SDL_Vulkan_GetVkGetInstanceProcAddr() }
    }
}

#[derive(Debug, Clone)]
pub enum WindowBuildError {
    HeightOverflows(u32),
    WidthOverflows(u32),
    InvalidTitle(NulError),
    SdlError(Error),
}

impl fmt::Display for WindowBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WindowBuildError::*;

        match *self {
            HeightOverflows(h) => write!(f, "Window height ({}) is too high.", h),
            WidthOverflows(w) => write!(f, "Window width ({}) is too high.", w),
            InvalidTitle(ref e) => write!(f, "Invalid window title: {}", e),
            SdlError(ref e) => write!(f, "SDL error: {}", e),
        }
    }
}

impl error::Error for WindowBuildError {
    fn description(&self) -> &str {
        use self::WindowBuildError::*;

        match *self {
            HeightOverflows(_) => "window height overflow",
            WidthOverflows(_) => "window width overflow",
            InvalidTitle(_) => "invalid window title",
            SdlError(ref e) => &e.0,
        }
    }
}

/// The type that allows you to build windows.
#[derive(Debug)]
pub struct WindowBuilder {
    title: String,
    width: u32,
    height: u32,
    x: WindowPos,
    y: WindowPos,
    window_flags: u32,
    create_metal_view: bool,
    /// The window builder cannot be built on a non-main thread, so prevent cross-threaded moves and references.
    /// `!Send` and `!Sync`,
    subsystem: VideoSubsystem,
}

impl WindowBuilder {
    /// Initializes a new `WindowBuilder`.
    pub fn new(v: &VideoSubsystem, title: &str, width: u32, height: u32) -> WindowBuilder {
        WindowBuilder {
            title: title.to_owned(),
            width,
            height,
            x: WindowPos::Undefined,
            y: WindowPos::Undefined,
            window_flags: 0,
            subsystem: v.clone(),
            create_metal_view: false,
        }
    }

    /// Builds the window.
    #[doc(alias = "SDL_CreateWindow")]
    pub fn build(&self) -> Result<Window, WindowBuildError> {
        use self::WindowBuildError::*;
        let title = match CString::new(self.title.clone()) {
            Ok(t) => t,
            Err(err) => return Err(InvalidTitle(err)),
        };
        if self.width >= (1 << 31) {
            return Err(WidthOverflows(self.width));
        }
        if self.height >= (1 << 31) {
            return Err(HeightOverflows(self.width));
        }

        let raw_width = self.width as c_int;
        let raw_height = self.height as c_int;
        unsafe {
            let props = SDL_CreateProperties();
            SDL_SetStringProperty(
                props,
                sys::video::SDL_PROP_WINDOW_CREATE_TITLE_STRING,
                title.as_ptr(),
            );

            if self.x != WindowPos::Undefined {
                SDL_SetNumberProperty(
                    props,
                    sys::video::SDL_PROP_WINDOW_CREATE_X_NUMBER,
                    to_ll_windowpos(self.x).into(),
                );
            }
            if self.y != WindowPos::Undefined {
                SDL_SetNumberProperty(
                    props,
                    sys::video::SDL_PROP_WINDOW_CREATE_Y_NUMBER,
                    to_ll_windowpos(self.y).into(),
                );
            }

            SDL_SetNumberProperty(
                props,
                sys::video::SDL_PROP_WINDOW_CREATE_WIDTH_NUMBER,
                raw_width.into(),
            );
            SDL_SetNumberProperty(
                props,
                sys::video::SDL_PROP_WINDOW_CREATE_HEIGHT_NUMBER,
                raw_height.into(),
            );
            let flags_cstr = CString::new("SDL.window.create.flags").unwrap();
            SDL_SetNumberProperty(props, flags_cstr.as_ptr(), self.window_flags.into());

            let raw = sys::video::SDL_CreateWindowWithProperties(props);
            SDL_DestroyProperties(props);
            let mut metal_view = 0 as sys::metal::SDL_MetalView;
            #[cfg(target_os = "macos")]
            if self.create_metal_view {
                {
                    metal_view = sys::metal::SDL_Metal_CreateView(raw);
                }
            }

            if raw.is_null() {
                Err(SdlError(get_error()))
            } else {
                {
                    Ok(Window::from_ll(self.subsystem.clone(), raw, metal_view))
                }
            }
        }
    }

    /// Gets the underlying window flags.
    pub fn window_flags(&self) -> u32 {
        self.window_flags
    }

    /// Sets the underlying window flags.
    /// This will effectively undo any previous build operations, excluding window size and position.
    pub fn set_window_flags(&mut self, flags: u32) -> &mut WindowBuilder {
        self.window_flags = flags;
        self
    }

    /// Sets the window position.
    pub fn position(&mut self, x: i32, y: i32) -> &mut WindowBuilder {
        self.x = WindowPos::Positioned(x);
        self.y = WindowPos::Positioned(y);
        self
    }

    /// Centers the window.
    pub fn position_centered(&mut self) -> &mut WindowBuilder {
        self.x = WindowPos::Centered;
        self.y = WindowPos::Centered;
        self
    }

    /// Sets the window to fullscreen.
    pub fn fullscreen(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_FULLSCREEN as u32;
        self
    }

    /// Sets the window to be usable with an OpenGL context
    pub fn opengl(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_OPENGL as u32;
        self
    }

    /// Sets the window to be usable with a Vulkan instance
    pub fn vulkan(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_VULKAN as u32;
        self
    }

    /// Hides the window.
    pub fn hidden(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_HIDDEN as u32;
        self
    }

    /// Removes the window decoration.
    pub fn borderless(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_BORDERLESS as u32;
        self
    }

    /// Sets the window to be resizable.
    pub fn resizable(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_RESIZABLE as u32;
        self
    }

    /// Minimizes the window.
    pub fn minimized(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_MINIMIZED as u32;
        self
    }

    /// Maximizes the window.
    pub fn maximized(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_MAXIMIZED as u32;
        self
    }

    /// Sets the window to have grabbed input focus.
    pub fn input_grabbed(&mut self) -> &mut WindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_MOUSE_GRABBED as u32;
        self
    }

    /// Create a SDL_MetalView when constructing the window.
    /// This is required when using the raw_window_handle feature on macOS.
    /// Has no effect no other platforms.
    pub fn metal_view(&mut self) -> &mut WindowBuilder {
        self.create_metal_view = true;
        self
    }
}

/// The type that allows you to build popup windows.
pub struct PopupWindowBuilder {
    parent_window: Window,
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
    window_flags: u32,
    create_metal_view: bool,
    /// The window builder cannot be built on a non-main thread, so prevent cross-threaded moves and references.
    /// `!Send` and `!Sync`,
    subsystem: VideoSubsystem,
}

impl PopupWindowBuilder {
    /// Initializes a new `PopupWindowBuilder`.
    pub unsafe fn new(
        v: &VideoSubsystem,
        parent_window: &Window,
        width: u32,
        height: u32,
    ) -> PopupWindowBuilder {
        PopupWindowBuilder {
            parent_window: Window::from_ref(parent_window.context()),
            width,
            height,
            offset_x: 0,
            offset_y: 0,
            window_flags: 0,
            subsystem: v.clone(),
            create_metal_view: false,
        }
    }

    /// Builds the popup window
    #[doc(alias = "SDL_CreatePopupWindow")]
    pub fn build(&self) -> Result<Window, WindowBuildError> {
        use self::WindowBuildError::*;
        if self.width >= (1 << 31) {
            return Err(WidthOverflows(self.width));
        }
        if self.height >= (1 << 31) {
            return Err(HeightOverflows(self.width));
        }
        if (self.window_flags & sys::video::SDL_WINDOW_TOOLTIP as u32 != 0)
            && (self.window_flags & sys::video::SDL_WINDOW_POPUP_MENU as u32 != 0)
        {
            return Err(SdlError(Error(
                "SDL_WINDOW_TOOLTIP and SDL_WINDOW_POPUP are mutually exclusive".to_owned(),
            )));
        }
        if (self.window_flags & sys::video::SDL_WINDOW_TOOLTIP as u32 == 0)
            && (self.window_flags & sys::video::SDL_WINDOW_POPUP_MENU as u32 == 0)
        {
            return Err(SdlError(Error(
                "SDL_WINDOW_TOOLTIP or SDL_WINDOW_POPUP are required for popup windows".to_owned(),
            )));
        }

        let raw_width = self.width as c_int;
        let raw_height = self.height as c_int;
        unsafe {
            let raw = sys::video::SDL_CreatePopupWindow(
                self.parent_window.raw(),
                self.offset_x,
                self.offset_y,
                raw_width,
                raw_height,
                self.window_flags.into(),
            );
            let mut metal_view = 0 as sys::metal::SDL_MetalView;
            #[cfg(target_os = "macos")]
            if self.create_metal_view {
                metal_view = sys::metal::SDL_Metal_CreateView(raw);
            }

            if raw.is_null() {
                Err(SdlError(get_error()))
            } else {
                Ok(Window::from_ll(self.subsystem.clone(), raw, metal_view))
            }
        }
    }

    /// Gets the underlying window flags.
    pub fn window_flags(&self) -> u32 {
        self.window_flags
    }

    /// Sets the underlying window flags.
    /// This will effectively undo any previous build operations, excluding window size and position.
    pub fn set_window_flags(&mut self, flags: u32) -> &mut PopupWindowBuilder {
        self.window_flags = flags;
        self
    }

    /// Sets the window offset relative to the parent window.
    pub fn offset(&mut self, x: i32, y: i32) -> &mut PopupWindowBuilder {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Sets the window to be usable with an OpenGL context
    pub fn opengl(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_OPENGL as u32;
        self
    }

    /// Sets the window to be usable with a Vulkan instance
    pub fn vulkan(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_VULKAN as u32;
        self
    }

    /// Hides the window.
    pub fn hidden(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_HIDDEN as u32;
        self
    }

    /// Sets the window to be resizable.
    pub fn resizable(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_RESIZABLE as u32;
        self
    }

    /// Sets the window to have grabbed input focus.
    pub fn input_grabbed(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_MOUSE_GRABBED as u32;
        self
    }

    /// Create a SDL_MetalView when constructing the window.
    /// This is required when using the raw_window_handle feature on MacOS.
    /// Has no effect no other platforms.
    pub fn metal_view(&mut self) -> &mut PopupWindowBuilder {
        self.create_metal_view = true;
        self
    }

    /// Sets the window to be a tooltip.
    pub fn tooltip(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_TOOLTIP as u32;
        self
    }

    /// Sets the window to be a popup menu.
    pub fn popup_menu(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_POPUP_MENU as u32;
        self
    }

    /// Sets the window to be transparent
    pub fn transparent(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_TRANSPARENT as u32;
        self
    }

    /// Sets the window to be shown on top of all other windows
    pub fn always_on_top(&mut self) -> &mut PopupWindowBuilder {
        self.window_flags |= sys::video::SDL_WINDOW_ALWAYS_ON_TOP as u32;
        self
    }
}

impl From<Window> for WindowCanvas {
    fn from(window: Window) -> WindowCanvas {
        create_renderer(window, None).unwrap()
    }
}

impl Window {
    #[inline]
    // this can prevent introducing UB until
    // https://github.com/rust-lang/rust-clippy/issues/5953 is fixed
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn raw(&self) -> *mut sys::video::SDL_Window {
        self.context.raw
    }

    #[inline]
    pub unsafe fn from_ll(
        subsystem: VideoSubsystem,
        raw: *mut sys::video::SDL_Window,
        metal_view: sys::metal::SDL_MetalView,
    ) -> Window {
        let context = WindowContext::from_ll(subsystem, raw, metal_view);
        context.into()
    }

    #[inline]
    /// Create a new `Window` without taking ownership of the `WindowContext`
    pub const unsafe fn from_ref(context: Arc<WindowContext>) -> Window {
        Window { context }
    }

    #[inline]
    pub fn subsystem(&self) -> &VideoSubsystem {
        &self.context.subsystem
    }

    /// Initializes a new `WindowCanvas';
    pub fn into_canvas(self) -> WindowCanvas {
        self.into()
    }

    pub fn context(&self) -> Arc<WindowContext> {
        self.context.clone()
    }

    #[doc(alias = "SDL_GetWindowID")]
    pub fn id(&self) -> u32 {
        unsafe { sys::video::SDL_GetWindowID(self.context.raw) }
    }

    #[doc(alias = "SDL_GL_CreateContext")]
    pub fn gl_create_context(&self) -> Result<GLContext, Error> {
        let result = unsafe { sys::video::SDL_GL_CreateContext(self.context.raw) };
        if result.is_null() {
            Err(get_error())
        } else {
            Ok(GLContext { raw: result })
        }
    }

    #[doc(alias = "SDL_GL_GetCurrentContext")]
    pub unsafe fn gl_get_current_context(&self) -> Option<GLContext> {
        let context_raw = sys::video::SDL_GL_GetCurrentContext();

        if !context_raw.is_null() {
            Some(GLContext { raw: context_raw })
        } else {
            None
        }
    }

    /// Set the window's OpenGL context to the current context on the thread.
    #[doc(alias = "SDL_GL_MakeCurrent")]
    pub fn gl_set_context_to_current(&self) -> Result<(), Error> {
        unsafe {
            let context_raw = sys::video::SDL_GL_GetCurrentContext();

            if !context_raw.is_null()
                && sys::video::SDL_GL_MakeCurrent(self.context.raw, context_raw)
            {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    #[doc(alias = "SDL_GL_MakeCurrent")]
    pub fn gl_make_current(&self, context: &GLContext) -> Result<(), Error> {
        unsafe {
            if sys::video::SDL_GL_MakeCurrent(self.context.raw, context.raw) {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    #[doc(alias = "SDL_GL_SwapWindow")]
    pub fn gl_swap_window(&self) {
        unsafe { sys::video::SDL_GL_SwapWindow(self.context.raw) };
    }

    /// Get the names of the Vulkan instance extensions needed to create a surface with `vulkan_create_surface`.
    #[doc(alias = "SDL_Vulkan_GetInstanceExtensions")]
    pub fn vulkan_instance_extensions(&self) -> Result<Vec<String>, Error> {
        let mut count: c_uint = 0;
        // returns a pointer to an array of extension names
        let extension_names_raw =
            unsafe { sys::vulkan::SDL_Vulkan_GetInstanceExtensions(&mut count) };
        if extension_names_raw.is_null() {
            return Err(get_error());
        }

        // Create a slice from the raw pointer to the array
        let names_slice =
            unsafe { std::slice::from_raw_parts(extension_names_raw, count as usize) };

        // Convert the C strings to Rust Strings
        let mut extension_names = Vec::with_capacity(count as usize);
        for &ext in names_slice {
            if ext.is_null() {
                return Err(Error(
                    "Received null pointer for extension name".to_string(),
                ));
            }
            let c_str = unsafe { CStr::from_ptr(ext) };
            extension_names.push(c_str.to_string_lossy().into_owned());
        }

        Ok(extension_names)
    }

    /// Create a Vulkan rendering surface for a window.
    ///
    /// The `VkInstance` must be created using a prior call to the
    /// [`vkCreateInstance`](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkCreateInstance.html)
    /// function in the Vulkan library.
    #[doc(alias = "SDL_Vulkan_CreateSurface")]
    pub fn vulkan_create_surface(&self, instance: VkInstance) -> Result<VkSurfaceKHR, Error> {
        #[cfg(feature = "ash")]
        let mut surface: VkSurfaceKHR = VkSurfaceKHR::default();

        #[cfg(not(feature = "ash"))]
        let mut surface: VkSurfaceKHR = 0 as _;
        if unsafe {
            sys::vulkan::SDL_Vulkan_CreateSurface(self.context.raw, instance, null(), &mut surface)
        } {
            Ok(surface)
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_GetDisplayForWindow")]
    pub fn display_index(&self) -> Result<i32, Error> {
        let result = unsafe { sys::video::SDL_GetDisplayForWindow(self.context.raw) };
        if result == 0 {
            Err(get_error())
        } else {
            Ok(result as i32)
        }
    }

    #[doc(alias = "SDL_SetWindowFullscreenMode")]
    pub fn set_display_mode<D>(&mut self, display_mode: D) -> Result<(), Error>
    where
        D: Into<Option<DisplayMode>>,
    {
        unsafe {
            let result = sys::video::SDL_SetWindowFullscreenMode(
                self.context.raw,
                match display_mode.into() {
                    Some(ref mode) => &mode.to_ll(),
                    None => ptr::null(),
                },
            );
            if result {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    #[doc(alias = "SDL_GetWindowFullscreenMode")]
    pub fn display_mode(&self) -> Option<DisplayMode> {
        unsafe {
            // returns a pointer to the mode, or NULL if the window will be fullscreen desktop
            let mode_raw = sys::video::SDL_GetWindowFullscreenMode(self.context.raw);
            if mode_raw.is_null() {
                return None;
            }
            Some(DisplayMode::from_ll(&*mode_raw))
        }
    }

    #[doc(alias = "SDL_GetWindowICCProfile")]
    pub fn icc_profile(&self) -> Result<Vec<u8>, Error> {
        unsafe {
            let mut size: usize = 0;
            let data = sys::video::SDL_GetWindowICCProfile(self.context.raw, &mut size as *mut _);
            if data.is_null() {
                return Err(get_error());
            }
            let mut result = vec![0; size];
            result.copy_from_slice(std::slice::from_raw_parts(data as *const u8, size));
            SDL_free(data);
            Ok(result)
        }
    }

    #[doc(alias = "SDL_GetWindowPixelFormat")]
    pub fn window_pixel_format(&self) -> PixelFormat {
        unsafe {
            PixelFormat::try_from(sys::video::SDL_GetWindowPixelFormat(self.context.raw)).unwrap()
        }
    }

    #[doc(alias = "SDL_GetWindowFlags")]
    pub fn window_flags(&self) -> SDL_WindowFlags {
        unsafe { sys::video::SDL_GetWindowFlags(self.context.raw) }
    }

    /// Does the window have input focus?
    pub fn has_input_focus(&self) -> bool {
        0 != self.window_flags() & sys::video::SDL_WINDOW_INPUT_FOCUS as Uint64
    }

    /// Has the window grabbed input focus?
    pub fn has_input_grabbed(&self) -> bool {
        0 != self.window_flags() & sys::video::SDL_WINDOW_MOUSE_GRABBED as Uint64
    }

    /// Does the window have mouse focus?
    pub fn has_mouse_focus(&self) -> bool {
        0 != self.window_flags() & sys::video::SDL_WINDOW_MOUSE_FOCUS as Uint64
    }

    /// Is the window maximized?
    pub fn is_maximized(&self) -> bool {
        0 != self.window_flags() & sys::video::SDL_WINDOW_MAXIMIZED as Uint64
    }

    /// Is the window minimized?
    pub fn is_minimized(&self) -> bool {
        0 != self.window_flags() & sys::video::SDL_WINDOW_MINIMIZED as Uint64
    }

    #[doc(alias = "SDL_SetWindowTitle")]
    pub fn set_title(&mut self, title: &str) -> Result<(), NulError> {
        let title = CString::new(title)?;
        unsafe {
            sys::video::SDL_SetWindowTitle(self.context.raw, title.as_ptr() as *const c_char);
        }
        Ok(())
    }

    #[doc(alias = "SDL_GetWindowTitle")]
    pub fn title(&self) -> &str {
        unsafe {
            let buf = sys::video::SDL_GetWindowTitle(self.context.raw);

            // The window title must be encoded in UTF-8.
            CStr::from_ptr(buf as *const _).to_str().unwrap()
        }
    }

    /// Use this function to set the icon for a window.
    ///
    /// # Example:
    /// ```compile_fail
    /// // requires "--features 'image'"
    /// use sdl3::surface::Surface;
    ///
    /// let window_icon = Surface::from_file("/path/to/icon.png")?;
    /// window.set_icon(window_icon);
    /// ```
    #[doc(alias = "SDL_SetWindowIcon")]
    pub fn set_icon<S: AsRef<SurfaceRef>>(&mut self, icon: S) -> bool {
        unsafe { sys::video::SDL_SetWindowIcon(self.context.raw, icon.as_ref().raw()) }
    }

    //pub fn SDL_SetWindowData(window: *SDL_Window, name: *c_char, userdata: *c_void) -> *c_void; //TODO: Figure out what this does
    //pub fn SDL_GetWindowData(window: *SDL_Window, name: *c_char) -> *c_void;

    #[doc(alias = "SDL_SetWindowPosition")]
    pub fn set_position(&mut self, x: WindowPos, y: WindowPos) -> bool {
        unsafe {
            sys::video::SDL_SetWindowPosition(
                self.context.raw,
                to_ll_windowpos(x),
                to_ll_windowpos(y),
            )
        }
    }

    #[doc(alias = "SDL_GetWindowPosition")]
    pub fn position(&self) -> (i32, i32) {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        unsafe { sys::video::SDL_GetWindowPosition(self.context.raw, &mut x, &mut y) };
        (x as i32, y as i32)
    }

    /// Use this function to get the size of a window's borders (decorations) around the client area.
    ///
    /// # Remarks
    /// This function is only supported on X11, otherwise an error is returned.
    #[doc(alias = "SDL_GetWindowBordersSize")]
    pub fn border_size(&self) -> Result<(u16, u16, u16, u16), Error> {
        let mut top: c_int = 0;
        let mut left: c_int = 0;
        let mut bottom: c_int = 0;
        let mut right: c_int = 0;
        let result = unsafe {
            sys::video::SDL_GetWindowBordersSize(
                self.context.raw,
                &mut top,
                &mut left,
                &mut bottom,
                &mut right,
            )
        };
        if result {
            Ok((top as u16, left as u16, bottom as u16, right as u16))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_SetWindowSize")]
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<(), IntegerOrSdlError> {
        let w = validate_int(width, "width")?;
        let h = validate_int(height, "height")?;
        unsafe {
            sys::video::SDL_SetWindowSize(self.context.raw, w, h);
        }
        Ok(())
    }

    // see notes about getting window sizes on high DPI displays:
    // https://github.com/libsdl-org/SDL/blob/main/docs/README-highdpi.md

    #[doc(alias = "SDL_GetWindowSize")]
    pub fn size(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { sys::video::SDL_GetWindowSize(self.context.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "SDL_GetDisplayContentScale")]
    pub fn display_content_scale(&self, display_id: sys::video::SDL_DisplayID) -> f32 {
        unsafe { sys::video::SDL_GetDisplayContentScale(display_id) }
    }

    #[doc(alias = "SDL_GetWindowPixelDensity")]
    pub fn pixel_density(&self) -> f32 {
        unsafe { sys::video::SDL_GetWindowPixelDensity(self.context.raw) }
    }

    #[doc(alias = "SDL_GetWindowSizeInPixels")]
    pub fn size_in_pixels(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { sys::video::SDL_GetWindowSizeInPixels(self.context.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "SDL_SetWindowMinimumSize")]
    pub fn set_minimum_size(&mut self, width: u32, height: u32) -> Result<(), IntegerOrSdlError> {
        let w = validate_int(width, "width")?;
        let h = validate_int(height, "height")?;
        unsafe {
            sys::video::SDL_SetWindowMinimumSize(self.context.raw, w, h);
        }
        Ok(())
    }

    #[doc(alias = "SDL_GetWindowMinimumSize")]
    pub fn minimum_size(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { sys::video::SDL_GetWindowMinimumSize(self.context.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "SDL_SetWindowMaximumSize")]
    pub fn set_maximum_size(&mut self, width: u32, height: u32) -> Result<(), IntegerOrSdlError> {
        let w = validate_int(width, "width")?;
        let h = validate_int(height, "height")?;
        unsafe {
            sys::video::SDL_SetWindowMaximumSize(self.context.raw, w, h);
        }
        Ok(())
    }

    #[doc(alias = "SDL_GetWindowMaximumSize")]
    pub fn maximum_size(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { sys::video::SDL_GetWindowMaximumSize(self.context.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "SDL_SetWindowBordered")]
    pub fn set_bordered(&mut self, bordered: bool) -> bool {
        unsafe { sys::video::SDL_SetWindowBordered(self.context.raw, bordered) }
    }

    #[doc(alias = "SDL_ShowWindow")]
    pub fn show(&mut self) -> bool {
        unsafe { sys::video::SDL_ShowWindow(self.context.raw) }
    }

    #[doc(alias = "SDL_HideWindow")]
    pub fn hide(&mut self) -> bool {
        unsafe { sys::video::SDL_HideWindow(self.context.raw) }
    }

    #[doc(alias = "SDL_RaiseWindow")]
    pub fn raise(&mut self) -> bool {
        unsafe { sys::video::SDL_RaiseWindow(self.context.raw) }
    }

    #[doc(alias = "SDL_MaximizeWindow")]
    pub fn maximize(&mut self) -> bool {
        unsafe { sys::video::SDL_MaximizeWindow(self.context.raw) }
    }

    #[doc(alias = "SDL_MinimizeWindow")]
    pub fn minimize(&mut self) -> bool {
        unsafe { sys::video::SDL_MinimizeWindow(self.context.raw) }
    }

    #[doc(alias = "SDL_RestoreWindow")]
    pub fn restore(&mut self) -> bool {
        unsafe { sys::video::SDL_RestoreWindow(self.context.raw) }
    }

    pub fn fullscreen_state(&self) -> FullscreenType {
        FullscreenType::from_window_flags(self.window_flags() as u32)
    }

    #[doc(alias = "SDL_SetWindowFullscreen")]
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), Error> {
        unsafe {
            let result = sys::video::SDL_SetWindowFullscreen(self.context.raw, fullscreen);
            if result {
                Ok(())
            } else {
                Err(get_error())
            }
        }
    }

    /// Returns a WindowSurfaceRef, which can be used like a regular Surface. This is an
    /// alternative way to the Renderer (Canvas) way to modify pixels directly in the Window.
    ///
    /// For this to happen, simply create a `WindowSurfaceRef` via this method, use the underlying
    /// Surface however you like, and when the changes of the Surface must be applied to the
    /// screen, call `update_window` if you intend to keep using the WindowSurfaceRef afterwards,
    /// or `finish` if you don't intend to use it afterwards.
    ///
    /// The Renderer way is of course much more flexible and recommended; even though you only want
    /// to support Software Rendering (which is what using Surface is), you can still create a
    /// Renderer which renders in a Software-based manner, so try to rely on a Renderer as much as
    /// possible !
    #[doc(alias = "SDL_GetWindowSurface")]
    pub fn surface<'a>(&'a self, _e: &'a EventPump) -> Result<WindowSurfaceRef<'a>, Error> {
        let raw = unsafe { sys::video::SDL_GetWindowSurface(self.context.raw) };

        if raw.is_null() {
            Err(get_error())
        } else {
            let surface_ref = unsafe { SurfaceRef::from_ll_mut(raw) };
            Ok(WindowSurfaceRef(surface_ref, self))
        }
    }

    #[doc(alias = "SDL_SetWindowKeyboardGrab")]
    pub fn set_keyboard_grab(&mut self, grabbed: bool) -> bool {
        unsafe { sys::video::SDL_SetWindowKeyboardGrab(self.context.raw, grabbed) }
    }

    #[doc(alias = "SDL_SetWindowMouseGrab")]
    pub fn set_mouse_grab(&mut self, grabbed: bool) -> bool {
        unsafe { sys::video::SDL_SetWindowMouseGrab(self.context.raw, grabbed) }
    }

    #[doc(alias = "SDL_GetWindowKeyboardGrab")]
    pub fn keyboard_grab(&self) -> bool {
        unsafe { sys::video::SDL_GetWindowKeyboardGrab(self.context.raw) }
    }

    #[doc(alias = "SDL_GetWindowMouseGrab")]
    pub fn mouse_grab(&self) -> bool {
        unsafe { sys::video::SDL_GetWindowMouseGrab(self.context.raw) }
    }

    #[doc(alias = "SDL_SetWindowMouseRect")]
    pub fn set_mouse_rect<R>(&self, rect: R) -> Result<(), Error>
    where
        R: Into<Option<Rect>>,
    {
        let rect = rect.into();
        let rect_raw_ptr = match rect {
            Some(ref rect) => rect.raw(),
            None => ptr::null(),
        };

        unsafe {
            if sys::video::SDL_SetWindowMouseRect(self.context.raw, rect_raw_ptr) {
                Err(get_error())
            } else {
                Ok(())
            }
        }
    }

    #[doc(alias = "SDL_GetWindowMouseRect")]
    pub fn mouse_rect(&self) -> Option<Rect> {
        unsafe {
            let raw_rect = sys::video::SDL_GetWindowMouseRect(self.context.raw);
            if raw_rect.is_null() {
                None
            } else {
                Some(Rect::new(
                    (*raw_rect).x,
                    (*raw_rect).y,
                    (*raw_rect).w as u32,
                    (*raw_rect).h as u32,
                ))
            }
        }
    }

    /// Set the transparency of the window. The given value will be clamped internally between
    /// `0.0` (fully transparent), and `1.0` (fully opaque).
    ///
    /// This method returns an error if opacity isn't supported by the current platform.
    #[doc(alias = "SDL_SetWindowOpacity")]
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowOpacity(self.context.raw, opacity) };
        if !result {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Returns the transparency of the window, as a value between `0.0` (fully transparent), and
    /// `1.0` (fully opaque).
    ///
    /// If opacity isn't supported by the current platform, this method returns `Ok(1.0)` instead
    /// of an error.
    #[doc(alias = "SDL_GetWindowOpacity")]
    pub fn opacity(&self) -> Result<f32, Error> {
        let opacity = unsafe { sys::video::SDL_GetWindowOpacity(self.context.raw) };
        if opacity == -1.0f32 {
            Err(get_error())
        } else {
            Ok(opacity)
        }
    }

    /// Requests a window to demand attention from the user.
    #[doc(alias = "SDL_FlashWindow")]
    pub fn flash(&mut self, operation: FlashOperation) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_FlashWindow(self.context.raw, operation.to_ll()) };
        if result {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

#[derive(Copy, Clone)]
#[doc(alias = "SDL_GetVideoDriver")]
pub struct DriverIterator {
    length: i32,
    index: i32,
}

impl Iterator for DriverIterator {
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        if self.index >= self.length {
            None
        } else {
            use std::str;

            unsafe {
                let buf = sys::video::SDL_GetVideoDriver(self.index);
                assert!(!buf.is_null());
                self.index += 1;

                Some(str::from_utf8(CStr::from_ptr(buf as *const _).to_bytes()).unwrap())
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.length as usize;
        (l, Some(l))
    }
}

impl ExactSizeIterator for DriverIterator {}

/// Gets an iterator of all video drivers compiled into the SDL2 library.
#[inline]
#[doc(alias = "SDL_GetVideoDriver")]
pub fn drivers() -> DriverIterator {
    // This function is thread-safe and doesn't require the video subsystem to be initialized.
    // The list of drivers are read-only and statically compiled into SDL2, varying by platform.

    // SDL_GetNumVideoDrivers can never return a negative value.
    DriverIterator {
        length: unsafe { sys::video::SDL_GetNumVideoDrivers() },
        index: 0,
    }
}
