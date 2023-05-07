#![no_std]

use core::ptr::null_mut;
use sdl3_sys::*;

fn main() {
    unsafe {
        let mut _window: *mut SDL_Window = null_mut();
        let mut _surface: *mut SDL_Surface = null_mut();
        if SDL_InitSubSystem(SDL_InitFlags::SDL_INIT_VIDEO) {
            panic!("failed to initialize sdl with video");
        };
        _window = SDL_CreateWindow(
            b"hello_sdl" as *const _ as *const i8,
            SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            SDL_WindowFlags::SDL_WINDOW_SHOWN as u32,
        );

        if _window == null_mut() {
            panic!("failed to create window");
        }

        _surface = SDL_GetWindowSurface(_window);
        SDL_FillSurfaceRect(
            _surface,
            null_mut(),
            SDL_MapRGB((*_surface).format, 0xFF, 0xFF, 0x00),
        );
        SDL_UpdateWindowSurface(_window);
        SDL_Delay(5000);
        SDL_DestroyWindow(_window);
        SDL_Quit();
    }
}
