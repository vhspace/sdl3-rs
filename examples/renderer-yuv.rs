extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use sdl3::render::FRect;
use sdl3_sys::pixels::SDL_PixelFormat;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(
            unsafe { PixelFormat::from_ll(SDL_PixelFormat::IYUV) },
            256,
            256,
        )
        .map_err(|e| e.to_string())?;
    // Create a U-V gradient
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        // `pitch` is the width of the Y component
        // The U and V components are half the width and height of Y

        let w = 256;
        let h = 256;

        // Set Y (constant)
        for y in 0..h {
            for x in 0..w {
                let offset = y * pitch + x;
                buffer[offset] = 128;
            }
        }

        let y_size = pitch * h;

        // Set U and V (X and Y)
        for y in 0..h / 2 {
            for x in 0..w / 2 {
                let u_offset = y_size + y * pitch / 2 + x;
                let v_offset = y_size + (pitch / 2 * h / 2) + y * pitch / 2 + x;
                buffer[u_offset] = (x * 2) as u8;
                buffer[v_offset] = (y * 2) as u8;
            }
        }
    })?;

    canvas.clear();
    canvas.copy(&texture, None, Some(FRect::new(100.0, 100.0, 256.0, 256.0)))?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
    }

    Ok(())
}
