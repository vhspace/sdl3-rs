#[cfg(feature = "raw-window-handle")]
mod raw_window_handle_test {
    extern crate raw_window_handle;

    use self::raw_window_handle::{
        HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    };
    use sdl3::{raw_window_handle::WindowHandles, video::Window};

    const fn assert_send_sync<T: Send + Sync>() {}
    const fn assert_has_window_and_display_handle<T: HasWindowHandle + HasDisplayHandle>() {}
    #[allow(dead_code)]
    const ASSERT: () = {
        assert_has_window_and_display_handle::<Window>();
        assert_has_window_and_display_handle::<WindowHandles<'static>>();
        assert_send_sync::<WindowHandles<'static>>();
    };

    /// Tests whether two structs with `HasWindowHandle + HasDisplayHandle` both return Ok(..) and both return the same value
    fn test_window_handle_providers_match(
        left: impl HasWindowHandle + HasDisplayHandle,
        right: impl HasWindowHandle + HasDisplayHandle,
    ) {
        assert_eq!(
            left.window_handle()
                .expect("Failed to get the window handle for the window"),
            right
                .window_handle()
                .expect("Failed to get the window handle for the window")
        );
        assert_eq!(
            left.display_handle()
                .expect("Failed to get the display handle for the window"),
            right
                .display_handle()
                .expect("Failed to get the display handle for the window")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn get_windows_handle() {
        let window = new_hidden_window();
        let window_clone = window.clone();
        let window_as_handle = window_clone.as_window_handles().unwrap();
        let window_handle = match window_as_handle.window_handle() {
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
        let display_handle = match window_as_handle.display_handle() {
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
        test_window_handle_providers_match(window, window_as_handle);
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
        let window_clone = window.clone();
        let window_as_handle = window_clone.as_window_handles().unwrap();
        match window_as_handle.window_handle().unwrap().as_raw() {
            RawWindowHandle::Xlib(x11_handle) => {
                assert_ne!(x11_handle.window, 0, "Window for X11 should not be 0");
                println!("Successfully received linux X11 RawWindowHandle!");
            }
            RawWindowHandle::Wayland(wayland_handle) => {
                assert_ne!(
                    wayland_handle.surface.as_ptr(),
                    std::ptr::null_mut(),
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
        match window_as_handle.display_handle().unwrap().as_raw() {
            RawDisplayHandle::Xlib(x11_display) => {
                assert_ne!(
                    x11_display.display.unwrap().as_ptr(),
                    std::ptr::null_mut(),
                    "Display for X11 should not be null"
                );
            }
            RawDisplayHandle::Wayland(wayland_display) => {
                assert_ne!(
                    wayland_display.display.as_ptr(),
                    std::ptr::null_mut(),
                    "Display for Wayland should not be null"
                );
            }
            x => assert!(
                false,
                "Received wrong RawDisplayHandle type for linux: {:?}",
                x
            ),
        }
        test_window_handle_providers_match(window, window_as_handle);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn get_macos_handle() {
        let window = new_hidden_window();
        let window_clone = window.clone();
        let window_as_handle = window_clone.as_window_handles().unwrap();
        let window_handle = match window_as_handle.window_handle() {
            Ok(v) => v,
            Err(err) => panic!(
                "Received error while getting window handle for MacOs: {:?}",
                err
            ),
        };
        match window_handle.as_raw() {
            RawWindowHandle::AppKit(macos_handle) => {
                assert!(
                    !macos_handle.ns_view.as_ptr().is_null(),
                    "ns_view should not be null"
                );
            }
            x => assert!(
                false,
                "Received wrong RawWindowHandle type for macOS: {:?}",
                x
            ),
        }
        let display_handle = match window_as_handle.display_handle() {
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
        test_window_handle_providers_match(window, window_as_handle);
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
