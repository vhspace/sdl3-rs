extern crate raw_window_handle;

use crate::video::Window;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
#[cfg(target_os = "windows")]
use std::num::NonZero;
use std::{ffi::CStr, ptr::NonNull};
use sys::{
    properties::{SDL_GetNumberProperty, SDL_GetPointerProperty},
    video::SDL_Window,
};

// this queries then stores all handles upon creation, then returns the stored values on request

/// Holds the handles for a [`Window`]. This can be created using [`Window::as_window_handles()`] or [`Window::as_window_handles_mut()`] then can be given to a crate such as wgpu or glutin. This might only be needed for wgpu (which needs a value that is Send/Sync), and the [`Window`] struct already implements [`HasWindowHandle`] and [`HasDisplayHandle`] for crates that do not need a Send/Sync value.
///
/// Note: this implements Send/Sync due to wgpu requirements, but the values returned by [`WindowHandles::window_handle()`] and [`WindowHandles::display_handle()`] can only be used on the same thread that created the window due to raw-window-handle (and OS) requirements
pub struct WindowHandles<'window> {
    window_handle: WindowHandle<'window>,
    display_handle: DisplayHandle<'window>,
}

// Note: these impls are necessary because Wgpu needs the given window handle struct to be Send/Sync
// Safety: WindowHandles only stores data (copies of native handle values) and it is the
// consumer's responsibility to use the returned handles (which are !Send/!Sync) only on
// the correct thread (as required by raw-window-handle). The handles are created when
// the WindowHandles struct is created, and the struct never dereferences them or calls
// into SDL afterwards.
unsafe impl<'window> Send for WindowHandles<'window> {}
unsafe impl<'window> Sync for WindowHandles<'window> {}

impl<'window> HasWindowHandle for WindowHandles<'window> {
    fn window_handle(&self) -> Result<WindowHandle<'window>, HandleError> {
        Ok(self.window_handle)
    }
}

impl<'window> HasDisplayHandle for WindowHandles<'window> {
    fn display_handle(&self) -> Result<DisplayHandle<'window>, HandleError> {
        Ok(self.display_handle)
    }
}

impl Window {
    /// Returns a Send/Sync [`WindowHandles`] struct that can be used by crates like wgpu.
    ///
    /// If you do not need a Send/Sync value, you can simply pass the [`Window`] struct to whatever needs a value that implements [`HasWindowHandle`]/[`HasDisplayHandle`].
    ///
    /// If you need to mutate the window after calling this function, you should create a clone of the window and get the window handles from the cloned value.
    pub fn as_window_handles<'window>(
        &'window self,
    ) -> Result<WindowHandles<'window>, HandleError> {
        Ok(WindowHandles {
            window_handle: get_window_handle(self.raw())?,
            display_handle: get_display_handle(self.raw())?,
        })
    }
}

impl<'window> HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        get_window_handle(self.raw())
    }
}

impl<'window> HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        get_display_handle(self.raw())
    }
}

// Access window handle using SDL3 properties
fn get_window_handle<'window>(
    raw_window: *mut SDL_Window,
) -> Result<WindowHandle<'window>, HandleError> {
    // Windows
    #[cfg(target_os = "windows")]
    unsafe {
        use raw_window_handle::Win32WindowHandle;

        let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

        let hwnd = SDL_GetPointerProperty(
            window_properties,
            sys::video::SDL_PROP_WINDOW_WIN32_HWND_POINTER,
            std::ptr::null_mut(),
        );
        let hinstance = SDL_GetPointerProperty(
            window_properties,
            sys::video::SDL_PROP_WINDOW_WIN32_INSTANCE_POINTER,
            std::ptr::null_mut(),
        );
        let mut handle = Win32WindowHandle::new(NonZero::new_unchecked(hwnd.addr() as isize));
        handle.hinstance = Some(NonZero::new_unchecked(hinstance.addr() as isize));
        let raw_window_handle = RawWindowHandle::Win32(handle);

        Ok(WindowHandle::borrow_raw(raw_window_handle))
    }

    // macOS
    #[cfg(target_os = "macos")]
    unsafe {
        use objc2::{msg_send, runtime::NSObject};
        use raw_window_handle::AppKitWindowHandle;

        let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

        let ns_window = SDL_GetPointerProperty(
            window_properties,
            sys::video::SDL_PROP_WINDOW_COCOA_WINDOW_POINTER,
            std::ptr::null_mut(),
        );
        let ns_view: *mut NSObject = msg_send![ns_window as *mut NSObject, contentView];
        if ns_view.is_null() {
            return Err(HandleError::Unavailable);
        }
        let handle = AppKitWindowHandle::new(NonNull::new_unchecked(ns_view.cast()));
        let raw_window_handle = RawWindowHandle::AppKit(handle);

        Ok(WindowHandle::borrow_raw(raw_window_handle))
    }

    // iOS
    #[cfg(target_os = "ios")]
    unsafe {
        use raw_window_handle::UiKitWindowHandle;

        let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

        let ui_view = SDL_GetPointerProperty(
            window_properties,
            sys::video::SDL_PROP_WINDOW_UIKIT_WINDOW_POINTER,
            std::ptr::null_mut(),
        );
        let handle = UiKitWindowHandle::new(NonNull::new_unchecked(ui_view));
        let raw_window_handle = RawWindowHandle::UiKit(handle);

        Ok(WindowHandle::borrow_raw(raw_window_handle))
    }

    // Android
    #[cfg(target_os = "android")]
    unsafe {
        use raw_window_handle::AndroidNdkWindowHandle;

        let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

        let native_window = SDL_GetPointerProperty(
            window_properties,
            sys::video::SDL_PROP_WINDOW_ANDROID_WINDOW_POINTER,
            std::ptr::null_mut(),
        );
        let handle = AndroidNdkWindowHandle::new(NonNull::new_unchecked(native_window));
        let raw_window_handle = RawWindowHandle::AndroidNdk(handle);

        Ok(WindowHandle::borrow_raw(raw_window_handle))
    }

    // Linux (X11 or Wayland)
    #[cfg(all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    ))]
    unsafe {
        let video_driver = CStr::from_ptr(sys::video::SDL_GetCurrentVideoDriver());

        match video_driver.to_bytes() {
            b"x11" => {
                use raw_window_handle::XlibWindowHandle;

                let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

                let window = SDL_GetNumberProperty(
                    window_properties,
                    sys::video::SDL_PROP_WINDOW_X11_WINDOW_NUMBER,
                    0,
                );
                let handle = XlibWindowHandle::new(window as _);
                let raw_window_handle = RawWindowHandle::Xlib(handle);

                Ok(WindowHandle::borrow_raw(raw_window_handle))
            }
            b"wayland" => {
                use raw_window_handle::WaylandWindowHandle;

                let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

                let window = SDL_GetPointerProperty(
                    window_properties,
                    sys::video::SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER,
                    std::ptr::null_mut(),
                );
                let handle = WaylandWindowHandle::new(NonNull::new_unchecked(window));
                let raw_window_handle = RawWindowHandle::Wayland(handle);

                Ok(WindowHandle::borrow_raw(raw_window_handle))
            }
            _ => {
                panic!("{video_driver:?} video driver is not supported, please file an issue with raw-window-handle.",);
            }
        }
    }
}

// Access display handle using SDL3 properties
fn get_display_handle<'window>(
    raw_window: *mut SDL_Window,
) -> Result<DisplayHandle<'window>, HandleError> {
    // Windows
    #[cfg(target_os = "windows")]
    unsafe {
        use raw_window_handle::WindowsDisplayHandle;
        let handle = WindowsDisplayHandle::new();
        let raw_window_handle = RawDisplayHandle::Windows(handle);

        Ok(DisplayHandle::borrow_raw(raw_window_handle))
    }

    // macOS
    #[cfg(target_os = "macos")]
    unsafe {
        use raw_window_handle::AppKitDisplayHandle;
        let handle = AppKitDisplayHandle::new();
        let raw_window_handle = RawDisplayHandle::AppKit(handle);

        Ok(DisplayHandle::borrow_raw(raw_window_handle))
    }

    // iOS
    #[cfg(target_os = "ios")]
    unsafe {
        use raw_window_handle::UiKitDisplayHandle;
        let handle = UiKitDisplayHandle::new();
        let raw_window_handle = RawDisplayHandle::UiKit(handle);

        Ok(DisplayHandle::borrow_raw(raw_window_handle))
    }

    // Android
    #[cfg(target_os = "android")]
    unsafe {
        use raw_window_handle::AndroidDisplayHandle;
        let handle = AndroidDisplayHandle::new();
        let raw_window_handle = RawDisplayHandle::Android(handle);

        Ok(DisplayHandle::borrow_raw(raw_window_handle))
    }

    // Linux (X11 or Wayland)
    #[cfg(all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    ))]
    unsafe {
        let video_driver = CStr::from_ptr(sys::video::SDL_GetCurrentVideoDriver());

        match video_driver.to_bytes() {
            b"x11" => {
                use raw_window_handle::XlibDisplayHandle;

                let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

                let display = SDL_GetPointerProperty(
                    window_properties,
                    sys::video::SDL_PROP_WINDOW_X11_DISPLAY_POINTER,
                    std::ptr::null_mut(),
                );
                let display = core::ptr::NonNull::<libc::c_void>::new(display);

                let window = SDL_GetNumberProperty(
                    window_properties,
                    sys::video::SDL_PROP_WINDOW_X11_SCREEN_NUMBER,
                    0,
                );
                let handle = XlibDisplayHandle::new(display, window as i32);
                let raw_window_handle = RawDisplayHandle::Xlib(handle);

                Ok(DisplayHandle::borrow_raw(raw_window_handle))
            }
            b"wayland" => {
                use raw_window_handle::WaylandDisplayHandle;

                let window_properties = sys::video::SDL_GetWindowProperties(raw_window);

                let display = SDL_GetPointerProperty(
                    window_properties,
                    sys::video::SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER,
                    std::ptr::null_mut(),
                );
                let Some(display) = core::ptr::NonNull::<libc::c_void>::new(display) else {
                    return Err(HandleError::Unavailable); // I'm unsure if this is the right error type, of if we should just panic here if the display isn't available?
                };
                let handle = WaylandDisplayHandle::new(display);
                let raw_window_handle = RawDisplayHandle::Wayland(handle);

                Ok(DisplayHandle::borrow_raw(raw_window_handle))
            }
            _ => {
                panic!("{video_driver:?} video driver is not supported, please file an issue with raw-window-handle.");
            }
        }
    }
}
