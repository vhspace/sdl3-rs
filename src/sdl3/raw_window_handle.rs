extern crate raw_window_handle;

use self::raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use crate::video::Window;

// Access window handle using SDL3 properties
unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let platform = unsafe { sys::platform::SDL_GetPlatform().to_string() };
        let video_driver = unsafe { sys::video::SDL_GetCurrentVideoDriver().to_string() };

        match platform.as_str() {
            #[cfg(target_os = "windows")]
            "Windows" => unsafe {
                use self::raw_window_handle::Win32WindowHandle;
                let mut handle = Win32WindowHandle::empty();

                let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                handle.hwnd = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_WIN32_HWND_POINTER, std::ptr::null_mut()) as *mut libc::c_void;
                handle.hinstance = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_WIN32_HINSTANCE_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                RawWindowHandle::Win32(handle)
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            "Linux" => {
                match video_driver.as_str() {
                    "x11" => {
                        use self::raw_window_handle::XlibWindowHandle;
                        let mut handle = XlibWindowHandle::empty();

                        let window_properties = SDL_GetWindowProperties(self.raw());

                        handle.display = SDL_GetPointerProperty(window_properties, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, std::ptr::null_mut()) as *mut libc::c_void;
                        handle.window = SDL_GetNumberProperty(window_properties, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0) as *mut libc::c_void;

                        RawWindowHandle::Xlib(handle)
                    }
                    "wayland" => {
                        use self::raw_window_handle::WaylandWindowHandle;
                        let mut handle = WaylandWindowHandle::empty();

                        let window_properties = SDL_GetWindowProperties(self.raw());

                        handle.display = SDL_GetPointerProperty(window_properties, SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER, std::ptr::null_mut()) as *mut libc::c_void;
                        handle.surface = SDL_GetPointerProperty(window_properties, SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                        RawWindowHandle::Wayland(handle)
                    }
                    x => {
                        panic!("{} video driver is not supported, please file an issue with raw-window-handle:", x);
                    }
                }
            }
            #[cfg(target_os = "macos")]
            "macOS" => unsafe {
                use self::raw_window_handle::AppKitWindowHandle;
                let mut handle = AppKitWindowHandle::empty();

                let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                handle.ns_window = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_COCOA_WINDOW_POINTER.as_ptr(), std::ptr::null_mut()) as *mut libc::c_void;

                RawWindowHandle::AppKit(handle)
            }
            #[cfg(any(target_os = "ios"))]
            "iOS" => {
                use self::raw_window_handle::UiKitWindowHandle;
                let mut handle = UiKitWindowHandle::empty();

                let window_properties = SDL_GetWindowProperties(self.raw());

                handle.ui_window = SDL_GetPointerProperty(window_properties, SDL_PROP_WINDOW_UIKIT_WINDOW_POINTER, std::ptr::null_mut()) as *mut libc::c_void;
                handle.ui_view = std::ptr::null_mut(); // Assume the consumer of RawWindowHandle will determine this

                RawWindowHandle::UiKit(handle)
            }
            #[cfg(any(target_os = "android"))]
            "Android" => {
                use self::raw_window_handle::AndroidNdkWindowHandle;
                let mut handle = AndroidNdkWindowHandle::empty();

                let window_properties = SDL_GetWindowProperties(self.raw());

                handle.a_native_window = SDL_GetPointerProperty(window_properties, SDL_PROP_WINDOW_ANDROID_NATIVE_WINDOW_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                RawWindowHandle::AndroidNdk(handle)
            }
            x => {
                panic!("{} window system is not supported, please file an issue with raw-window-handle.", x);
            }
        }
    }
}

// Access display handle using SDL3 properties
unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        let video_driver = unsafe { sys::video::SDL_GetCurrentVideoDriver().to_string() };
        let platform = unsafe { sys::platform::SDL_GetPlatform().to_string() };

        match platform.as_str() {
            #[cfg(target_os = "windows")]
            "Windows" => {
                use self::raw_window_handle::WindowsDisplayHandle;
                let mut handle = WindowsDisplayHandle::empty();

                RawDisplayHandle::Windows(handle)
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            "Linux" => {
                match video_driver.as_str() {
                    "x11" => unsafe {
                        use self::raw_window_handle::XlibDisplayHandle;
                        let mut handle = XlibDisplayHandle::empty();

                        let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                        handle.display = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_X11_DISPLAY_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                        RawDisplayHandle::Xlib(handle)
                    }
                    "wayland" => {
                        use self::raw_window_handle::WaylandDisplayHandle;
                        let mut handle = WaylandDisplayHandle::empty();

                        let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                        handle.display = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                        RawDisplayHandle::Wayland(handle)
                    }
                    x => {
                        panic!("{} video driver is not supported, please file an issue with raw-window-handle.", x);
                    }
                }
            }
            #[cfg(target_os = "macos")]
            "macOS" => {
                use self::raw_window_handle::AppKitDisplayHandle;
                let handle = AppKitDisplayHandle::empty();

                RawDisplayHandle::AppKit(handle)
            }
            #[cfg(any(target_os = "ios"))]
            "iOS" => {
                use self::raw_window_handle::UiKitDisplayHandle;
                let mut handle = UiKitDisplayHandle::empty();

                RawDisplayHandle::UiKit(handle)
            }
            #[cfg(any(target_os = "android"))]
            "Android" => {
                use self::raw_window_handle::AndroidDisplayHandle;
                let mut handle = AndroidDisplayHandle::empty();

                let window_properties = sys::video::SDL_GetWindowProperties(self.raw());

                handle.a_native_window = sys::video::SDL_GetPointerProperty(window_properties, sys::video::SDL_PROP_WINDOW_ANDROID_NATIVE_WINDOW_POINTER, std::ptr::null_mut()) as *mut libc::c_void;

                RawDisplayHandle::Android(handle)
            }
            x => {
                panic!("{} window system is not supported, please file an issue with raw-window-handle.", x);
            }
        }
    }
}