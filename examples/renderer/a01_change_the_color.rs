use sdl3::{event::Event, pixels::Color};

use std::f64::consts::PI;
use std::time::Instant;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(
            "Change the color of the rust-sdl3 screen to a sine wave pattern",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump()?;
    let start_time = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let elapsed_time = start_time.elapsed().as_secs_f64();

        // Calculate color values using sine wave
        let red = 0.5 + 0.5 * (elapsed_time * 2.0 * PI).sin();
        let green = 0.5 + 0.5 * ((elapsed_time * 2.0 * PI) + (2.0 * PI / 3.0)).sin();
        let blue = 0.5 + 0.5 * ((elapsed_time * 2.0 * PI) + (4.0 * PI / 3.0)).sin();

        // Set render color
        canvas.set_draw_color(Color::RGB(
            (red * 255.0) as u8,
            (green * 255.0) as u8,
            (blue * 255.0) as u8,
        ));

        // Clear the canvas
        canvas.clear();

        // Present the rendered frame
        canvas.present();
    }

    Ok(())
}
