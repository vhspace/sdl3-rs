extern crate sdl3;

use sdl3::event::Event;
use sdl3::image::{InitFlag, LoadSurface};
use sdl3::keyboard::Keycode;
use sdl3::mouse::Cursor;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::surface::Surface;
use std::env;
use std::path::Path;

pub fn run(png: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl3::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("rust-sdl3 demo: Cursor", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();

    let surface =
        Surface::from_file(png).map_err(|err| format!("failed to load cursor image: {}", err))?;
    let cursor = Cursor::from_surface(surface, 0, 0)
        .map_err(|err| format!("failed to load cursor: {}", err))?;
    cursor.set();

    canvas.clear();
    canvas.present();

    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));

    let mut events = sdl_context.event_pump()?;

    'mainloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::MouseButtonDown { x, y, .. } => {
                    canvas.fill_rect(Rect::new(x as i32, y as i32, 1, 1))?;
                    canvas.present();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run /path/to/image.(png|jpg)")
    } else {
        run(Path::new(&args[1]))?;
    }

    Ok(())
}
