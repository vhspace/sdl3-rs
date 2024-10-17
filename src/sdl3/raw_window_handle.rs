extern crate raw_window_handle;

use self::raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use crate::video::Window;

// Access window handle using SDL3 properties
unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        // Windows
        #[cfg(target_os = "windows")]
        {
            use self::raw_window_handle::Win32WindowHandle;
            let mut handle = Win32WindowHandle::empty();

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            handle.hwnd = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_WIN32_HWND_POINTER,
                std::ptr::null_mut(),
            ) as *mut libc::c_void;
            handle.hinstance = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_WIN32_HINSTANCE_POINTER,
                std::ptr::null_mut(),
            ) as *mut libc::c_void;

            return RawWindowHandle::Win32(handle);
        }

        // macOS
        #[cfg(target_os = "macos")]
        unsafe {
            use self::raw_window_handle::AppKitWindowHandle;
            let mut handle = AppKitWindowHandle::empty();

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            handle.ns_window = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_COCOA_WINDOW_POINTER.as_ptr(),
                std::ptr::null_mut(),
            ) as *mut libc::c_void;

            return RawWindowHandle::AppKit(handle);
        }

        // iOS
        #[cfg(target_os = "ios")]
        {
            use self::raw_window_handle::UiKitWindowHandle;
            let mut handle = UiKitWindowHandle::empty();

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            handle.ui_window = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_UIKIT_WINDOW_POINTER,
                std::ptr::null_mut(),
            ) as *mut libc::c_void;
            handle.ui_view = std::ptr::null_mut(); // Assume the consumer of RawWindowHandle will determine this

            return RawWindowHandle::UiKit(handle);
        }

        // Android
        #[cfg(target_os = "android")]
        {
            use self::raw_window_handle::AndroidNdkWindowHandle;
            let mut handle = AndroidNdkWindowHandle::empty();

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            handle.a_native_window = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_ANDROID_NATIVE_WINDOW_POINTER,
                std::ptr::null_mut(),
            ) as *mut libc::c_void;

            return RawWindowHandle::AndroidNdk(handle);
        }

        // Linux (X11 or Wayland)
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "ios"),
            not(target_os = "android")
        ))]
        {
            let video_driver = unsafe { sys::video::SDL_GetCurrentVideoDriver().to_string() };

            match video_driver.as_str() {
                "x11" => {
                    use self::raw_window_handle::XlibWindowHandle;
                    let mut handle = XlibWindowHandle::empty();

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                    handle.display = sys::video::SDL_GetPointerProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_X11_DISPLAY_POINTER,
                        std::ptr::null_mut(),
                    ) as *mut libc::c_void;
                    handle.window = sys::video::SDL_GetNumberProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_X11_WINDOW_NUMBER,
                        0,
                    ) as *mut libc::c_void;

                    return RawWindowHandle::Xlib(handle);
                }
                "wayland" => {
                    use self::raw_window_handle::WaylandWindowHandle;
                    let mut handle = WaylandWindowHandle::empty();

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                    handle.display = sys::video::SDL_GetPointerProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER,
                        std::ptr::null_mut(),
                    ) as *mut libc::c_void;
                    handle.surface = sys::video::SDL_GetPointerProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER,
                        std::ptr::null_mut(),
                    ) as *mut libc::c_void;

                    return RawWindowHandle::Wayland(handle);
                }
                x => {
                    panic!(
                        "{} video driver is not supported, please file an issue with raw-window-handle.",
                        x
                    );
                }
            }
        }

        // Unsupported platform
        {
            panic!("This platform is not supported, please file an issue with raw-window-handle.");
        }
    }
}

// Access display handle using SDL3 properties
unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        // Windows
        #[cfg(target_os = "windows")]
        {
            use self::raw_window_handle::WindowsDisplayHandle;
            let handle = WindowsDisplayHandle::empty();

            return RawDisplayHandle::Windows(handle);
        }

        // macOS
        #[cfg(target_os = "macos")]
        {
            use self::raw_window_handle::AppKitDisplayHandle;
            let handle = AppKitDisplayHandle::empty();

            return RawDisplayHandle::AppKit(handle);
        }

        // iOS
        #[cfg(target_os = "ios")]
        {
            use self::raw_window_handle::UiKitDisplayHandle;
            let handle = UiKitDisplayHandle::empty();

            return RawDisplayHandle::UiKit(handle);
        }

        // Android
        #[cfg(target_os = "android")]
        {
            use self::raw_window_handle::AndroidDisplayHandle;
            let mut handle = AndroidDisplayHandle::empty();

            let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

            handle.a_native_window = sys::video::SDL_GetPointerProperty(
                window_properties,
                sys::video::SDL_PROP_WINDOW_ANDROID_NATIVE_WINDOW_POINTER,
                std::ptr::null_mut(),
            ) as *mut libc::c_void;

            return RawDisplayHandle::Android(handle);
        }

        // Linux (X11 or Wayland)
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "ios"),
            not(target_os = "android")
        ))]
        {
            let video_driver = unsafe { sys::video::SDL_GetCurrentVideoDriver().to_string() };

            match video_driver.as_str() {
                "x11" => {
                    use self::raw_window_handle::XlibDisplayHandle;
                    let mut handle = XlibDisplayHandle::empty();

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                    handle.display = sys::video::SDL_GetPointerProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_X11_DISPLAY_POINTER,
                        std::ptr::null_mut(),
                    ) as *mut libc::c_void;

                    return RawDisplayHandle::Xlib(handle);
                }
                "wayland" => {
                    use self::raw_window_handle::WaylandDisplayHandle;
                    let mut handle = WaylandDisplayHandle::empty();

                    let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                    handle.display = sys::video::SDL_GetPointerProperty(
                        window_properties,
                        sys::video::SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER,
                        std::ptr::null_mut(),
                    ) as *mut libc::c_void;

                    return RawDisplayHandle::Wayland(handle);
                }
                x => {
                    panic!(
                        "{} video driver is not supported, please file an issue with raw-window-handle.",
                        x
                    );
                }
            }
        }

        // Unsupported platform
        {
            panic!("This platform is not supported, please file an issue with raw-window-handle.");
        }
    }
}