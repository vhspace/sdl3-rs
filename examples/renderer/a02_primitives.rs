// https://github.com/libsdl-org/SDL/tree/main/examples/renderer
extern crate sdl3;

use rand::Rng;
use sdl3::rect::Rect;
use sdl3::render::FRect;
use sdl3::{event::Event, pixels::Color, rect::Point};

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const NUM_POINTS: usize = 500;

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(
            "SDL3 Renderer Primitives Example",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump()?;

    let mut points: Vec<Point> = Vec::with_capacity(NUM_POINTS);
    let mut rng = rand::thread_rng(); // Using thread_rng instead of just rng

    for _ in 0..NUM_POINTS {
        let x = rng.gen_range(100..540); // Generating random numbers within the bounds
        let y = rng.gen_range(100..380);
        points.push(Point::new(x, y));
    }

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(()),
                _ => {}
            }
        }

        // Clear the canvas
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // Draw red points
        canvas.set_draw_color(Color::RED);
        for point in &points {
            canvas.draw_point(*point)?;
        }

        // Draw blue rectangle
        canvas.set_draw_color(Color::BLUE);
        let rect = Rect::new(100, 100, 440, 280);
        canvas.fill_rect(rect)?;

        // Draw green unfilled rectangle
        canvas.set_draw_color(Color::GREEN);
        let inner_rect = FRect::new(
            (rect.x() + 30) as f32,
            (rect.y() + 30) as f32,
            (rect.width() - 60) as f32,
            (rect.height() - 60) as f32,
        );
        let _ = canvas.draw_rect(inner_rect);

        // Draw yellow lines
        canvas.set_draw_color(Color::YELLOW);
        canvas.draw_line(
            Point::new(0, 0),
            Point::new(WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32),
        )?;
        canvas.draw_line(
            Point::new(0, WINDOW_HEIGHT as i32),
            Point::new(WINDOW_WIDTH as i32, 0),
        )?;

        // Present the rendered frame
        canvas.present();
    }
}
