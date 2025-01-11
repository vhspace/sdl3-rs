extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{FPoint, FRect};
use sdl3_sys::pixels::SDL_PixelFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-sdl3 resource-manager demo", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(
            unsafe { PixelFormat::from_ll(SDL_PixelFormat::RGBA8888) },
            400,
            300,
        )
        .map_err(|e| e.to_string())?;

    let mut angle = 0.0;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
        angle = (angle + 0.5) % 360.;
        canvas.with_texture_canvas(&mut texture, |texture_canvas| {
            texture_canvas.clear();
            texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
            texture_canvas
                .fill_rect(FRect::new(0.0, 0.0, 400.0, 300.0))
                .expect("could not fill rect");
        });
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let dst = Some(FRect::new(0.0, 0.0, 400.0, 300.0));
        canvas.clear();
        canvas.copy_ex(
            &texture,
            None,
            dst,
            angle,
            Some(FPoint::new(400.0, 300.0)),
            false,
            false,
        )?;
        canvas.present();
    }

    Ok(())
}
