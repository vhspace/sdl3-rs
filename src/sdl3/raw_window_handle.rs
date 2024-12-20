extern crate raw_window_handle;

use self::raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle, WindowHandle,
};
use crate::video::Window;
use raw_window_handle::RawDisplayHandle;
use std::{ffi::CStr, num::NonZero, ptr::NonNull};
use sys::properties::{SDL_GetNumberProperty, SDL_GetPointerProperty};

// Access window handle using SDL3 properties
impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // Windows
        #[cfg(target_os = "windows")]
        unsafe {
            use self::raw_window_handle::Win32WindowHandle;

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
            use self::raw_window_handle::AppKitWindowHandle;

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            let ns_view = SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_COCOA_WINDOW_POINTER,
                std::ptr::null_mut(),
            );
            let handle = AppKitWindowHandle::new(NonNull::new_unchecked(ns_view));
            let raw_window_handle = RawWindowHandle::AppKit(handle);

            Ok(WindowHandle::borrow_raw(raw_window_handle))
        }

        // iOS
        #[cfg(target_os = "ios")]
        unsafe {
            use self::raw_window_handle::UiKitWindowHandle;

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
            use self::raw_window_handle::AndroidNdkWindowHandle;

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
                    use self::raw_window_handle::XlibWindowHandle;

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
                    use self::raw_window_handle::WaylandWindowHandle;

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
}

// Access display handle using SDL3 properties
impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // Windows
        #[cfg(target_os = "windows")]
        unsafe {
            use self::raw_window_handle::WindowsDisplayHandle;
            let handle = WindowsDisplayHandle::new();
            let raw_window_handle = RawDisplayHandle::Windows(handle);

            Ok(DisplayHandle::borrow_raw(raw_window_handle))
        }

        // macOS
        #[cfg(target_os = "macos")]
        unsafe {
            use self::raw_window_handle::AppKitDisplayHandle;
            let handle = AppKitDisplayHandle::new();
            let raw_window_handle = RawDisplayHandle::AppKit(handle);

            Ok(DisplayHandle::borrow_raw(raw_window_handle))
        }

        // iOS
        #[cfg(target_os = "ios")]
        unsafe {
            use self::raw_window_handle::UiKitDisplayHandle;
            let handle = UiKitDisplayHandle::new();
            let raw_window_handle = RawDisplayHandle::UiKit(handle);

            Ok(DisplayHandle::borrow_raw(raw_window_handle))
        }

        // Android
        #[cfg(target_os = "android")]
        unsafe {
            use self::raw_window_handle::AndroidDisplayHandle;
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
                    use self::raw_window_handle::XlibDisplayHandle;

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
                    use self::raw_window_handle::WaylandDisplayHandle;

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

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
}
