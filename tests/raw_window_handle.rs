#[cfg(feature = "raw-window-handle")]
mod raw_window_handle_test {
    extern crate raw_window_handle;
    extern crate sdl3;

    use self::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle, RawDisplayHandle};
    use self::sdl3::video::Window;

    #[cfg(target_os = "windows")]
    #[test]
    fn get_windows_handle() {
        use raw_window_handle::RawDisplayHandle;

        let window = new_hidden_window();
        let window_handle = match window.window_handle() {
            Ok(v) => v,
            Err(err) => panic!(
                "Received error while getting window handle for Windows: {:?}",
                err
            ),
        };
        let raw_handle = match window_handle.as_raw() {
            RawWindowHandle::Win32(v) => v,
            x => panic!("Received wrong RawWindowHandle type for Windows: {:?}", x),
        };
        assert_ne!(raw_handle.hwnd.get(), 0);
        println!("Successfully received Windows RawWindowHandle!");
        let display_handle = match window.display_handle() {
            Ok(v) => v,
            Err(err) => panic!(
                "Received error while getting display handle for Windows: {:?}",
                err
            ),
        };
        match display_handle.as_raw() {
            RawDisplayHandle::Windows(_) => {}
            x => panic!("Received wrong RawDisplayHandle type for Windows: {:?}", x),
        }
        println!("Successfully received Windows RawDisplayHandle!");
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[test]
    fn get_linux_handle() {
        let window = new_hidden_window();
        match window.raw_window_handle() {
            RawWindowHandle::Xlib(x11_handle) => {
                assert_ne!(x11_handle.window, 0, "Window for X11 should not be 0");
                println!("Successfully received linux X11 RawWindowHandle!");
            }
            RawWindowHandle::Wayland(wayland_handle) => {
                assert_ne!(
                    wayland_handle.surface, 0 as *mut libc::c_void,
                    "Surface for Wayland should not be null"
                );
                println!("Successfully received linux Wayland RawWindowHandle!");
            }
            x => assert!(
                false,
                "Received wrong RawWindowHandle type for linux: {:?}",
                x
            ),
        }
        match window.raw_display_handle() {
            RawDisplayHandle::Xlib(x11_display) => {
                assert_ne!(
                    x11_display.display, 0 as *mut libc::c_void,
                    "Display for X11 should not be null"
                );
            }
            RawDisplayHandle::Wayland(wayland_display) => {
                assert_ne!(
                    wayland_display.display, 0 as *mut libc::c_void,
                    "Display for Wayland should not be null"
                );
            }
            x => assert!(
                false,
                "Received wrong RawDisplayHandle type for linux: {:?}",
                x
            ),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn get_macos_handle() {
        let window = new_hidden_window();
        let window_handle = match window.window_handle() {
            Ok(v) => v,
            Err(err) => panic!(
                "Received error while getting window handle for MacOs: {:?}",
                err
            ),
        };
        match window_handle.as_raw() {
            RawWindowHandle::AppKit(macos_handle) => {
                assert!(!macos_handle.ns_view.as_ptr().is_null(), "ns_view should not be null");
            }
            x => assert!(
                false,
                "Received wrong RawWindowHandle type for macOS: {:?}",
                x
            ),
        }
        let display_handle = match window.display_handle() {
            Ok(v) => v,
            Err(err) => panic!(
                "Received error while getting display handle for MacOS: {:?}",
                err
            ),
        };
        match display_handle.as_raw() {
            RawDisplayHandle::AppKit(_) => {}
            x => assert!(
                false,
                "Received wrong RawDisplayHandle type for macOS: {:?}",
                x
            ),
        }
    }

    pub fn new_hidden_window() -> Window {
        let context = sdl3::init().unwrap();
        let video_subsystem = context.video().unwrap();
        video_subsystem
            .window("Hello, World!", 800, 600)
            .hidden()
            .metal_view()
            .build()
            .unwrap()
    }
}
