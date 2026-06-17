extern crate raw_window_handle;

use crate::video::Window;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::{ffi::CStr, num::NonZero, ptr::NonNull};
use sys::{properties::{SDL_GetNumberProperty, SDL_GetPointerProperty}, video::SDL_Window};

// this queries then stores all handles upon creation, then returns the stored values on request

#[derive(Debug, PartialEq)]
pub struct WindowAsWindowHandle<'a> {
    window_handle: WindowHandle<'a>,
    display_handle: DisplayHandle<'a>,
}

unsafe impl<'a> Send for WindowAsWindowHandle<'a> {}
unsafe impl<'a> Sync for WindowAsWindowHandle<'a> {}

impl<'a> HasWindowHandle for WindowAsWindowHandle<'a> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(self.window_handle)
    }
}

impl<'a> HasDisplayHandle for WindowAsWindowHandle<'a> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(self.display_handle)
    }
}

impl Window {
    /// Gives a window handle that can be used by crates like wgpu, glutin, etc. Note: using this function means you cannot use methods on `sdl3::video::Window` which take `&mut Self` (such as `window.show()`, `window.minimize()`, etc), if you need to these functions then use `Window::mut_as_window_handle()` instead.
    pub fn as_window_handle<'a>(&'a self) -> Result<WindowAsWindowHandle<'a>, HandleError> {
        Ok(WindowAsWindowHandle {
            window_handle: window_handle(self.raw())?,
            display_handle: display_handle(self.raw())?,
        })
    }
	// Safety: this does break rust's safety rule that you cannot modify something while there are other references to it, but following that rule religiously here would do more harm than good
    /// Gives a window handle that can be used by crates like wgpu, glutin, etc. Unlike `Window::as_window_handle()`, this gives back a mutable reference which can be used for methods which take `&mut Self` (such as `window.show()`, `window.minimize()`, etc)
    pub fn mut_as_window_handle<'a>(&'a mut self) -> Result<(&'a mut Self, WindowAsWindowHandle<'a>), HandleError> {
        let handle = WindowAsWindowHandle {
            window_handle: window_handle(self.raw())?,
            display_handle: display_handle(self.raw())?,
        };
        Ok((self, handle))
    }
}

// Access window handle using SDL3 properties
fn window_handle<'a>(raw_window: *mut SDL_Window) -> Result<WindowHandle<'a>, HandleError> {
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
        use raw_window_handle::AppKitWindowHandle;
        use objc2::{msg_send, runtime::NSObject};

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
                let handle = XlibWindowHandle::new(window as u64);
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
fn display_handle<'a>(raw_window: *mut SDL_Window) -> Result<DisplayHandle<'a>, HandleError> {
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
