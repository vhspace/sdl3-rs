extern crate sdl3;
use std::path::Path;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::render::FRect;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("sdl3", 640, 480)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(sdl3::pixels::Color::RGBA(0, 0, 0, 255));

    let mut event_pump = sdl_context.event_pump()?;

    // animation sheet and extras are available from
    // https://opengameart.org/content/a-platformer-in-the-forest
    let temp_surface = sdl3::surface::Surface::load_bmp(Path::new("assets/characters.bmp"))?;
    let texture = texture_creator
        .create_texture_from_surface(&temp_surface)
        .map_err(|e| e.to_string())?;

    let frames_per_anim = 4;
    let sprite_tile_size = (32., 32.);

    // Baby - walk animation
    let mut source_rect_0 = FRect::new(0., 0., sprite_tile_size.0, sprite_tile_size.0);
    let mut dest_rect_0 = FRect::new(0., 0., sprite_tile_size.0 * 4., sprite_tile_size.0 * 4.);
    // dest_rect_0.center_on(FPoint::new(-64., 120.));

    // King - walk animation
    let mut source_rect_1 = FRect::new(0., 32., sprite_tile_size.0, sprite_tile_size.0);
    let mut dest_rect_1 = FRect::new(0., 32., sprite_tile_size.0 * 4., sprite_tile_size.0 * 4.);
    // dest_rect_1.center_on(FPoint::new(0., 240.));

    // Soldier - walk animation
    let mut source_rect_2 = FRect::new(0., 64., sprite_tile_size.0, sprite_tile_size.0);
    let mut dest_rect_2 = FRect::new(0., 64., sprite_tile_size.0 * 4., sprite_tile_size.0 * 4.);
    // dest_rect_2.center_on(Point::new(440, 360));

    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                _ => {}
            }
        }

        let ticks = sdl3::timer::ticks() as i32;

        // set the current frame for time
        source_rect_0.set_x((32 * ((ticks / 100) % frames_per_anim)) as f32);
        dest_rect_0.set_x((1 * ((ticks / 14) % 768) - 128) as f32);

        source_rect_1.set_x((32 * ((ticks / 100) % frames_per_anim)) as f32);
        dest_rect_1.set_x(((1 * ((ticks / 12) % 768) - 672) * -1) as f32);

        source_rect_2.set_x((32 * ((ticks / 100) % frames_per_anim)) as f32);
        dest_rect_2.set_x((1 * ((ticks / 10) % 768) - 128) as f32);

        canvas.clear();
        // copy the frame to the canvas
        canvas.copy_ex(
            &texture,
            Some(source_rect_0),
            Some(dest_rect_0),
            0.0,
            None,
            false,
            false,
        )?;
        canvas.copy_ex(
            &texture,
            Some(source_rect_1),
            Some(dest_rect_1),
            0.0,
            None,
            true,
            false,
        )?;
        canvas.copy_ex(
            &texture,
            Some(source_rect_2),
            Some(dest_rect_2),
            0.0,
            None,
            false,
            false,
        )?;
        canvas.present();

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
