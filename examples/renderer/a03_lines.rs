extern crate rand;
extern crate sdl3;

use rand::Rng;
use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::render::FPoint;
use sdl3::{render::Canvas, video::Window, Sdl};

pub struct AppState {
    canvas: Canvas<Window>,
}

impl AppState {
    pub fn new(sdl: &Sdl) -> Result<AppState, String> {
        // Initialize SDL
        let video_subsystem = sdl.video().unwrap();

        // Create the window and renderer
        let window = video_subsystem
            .window("Example Renderer Lines", 640, 480)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas();

        Ok(AppState { canvas })
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Quit { .. } => false, // Exit the program
            _ => true,                   // Continue running
        }
    }

    pub fn iterate(&mut self) {
        let line_points = [
            FPoint::new(100.0, 354.0),
            FPoint::new(220.0, 230.0),
            FPoint::new(140.0, 230.0),
            FPoint::new(320.0, 100.0),
            FPoint::new(500.0, 230.0),
            FPoint::new(420.0, 230.0),
            FPoint::new(540.0, 354.0),
            FPoint::new(400.0, 354.0),
            FPoint::new(100.0, 354.0),
        ];

        // Clear the canvas with a grey background
        self.canvas.set_draw_color(Color::RGB(100, 100, 100));
        self.canvas.clear();

        // Draw individual lines (brown)
        self.canvas.set_draw_color(Color::RGB(127, 49, 32));
        self.canvas
            .draw_line(Point::new(240, 450), Point::new(400, 450))
            .unwrap();
        self.canvas
            .draw_line(Point::new(240, 356), Point::new(400, 356))
            .unwrap();
        self.canvas
            .draw_line(Point::new(240, 356), Point::new(240, 450))
            .unwrap();
        self.canvas
            .draw_line(Point::new(400, 356), Point::new(400, 450))
            .unwrap();

        // Draw a series of connected lines (green)
        self.canvas.set_draw_color(Color::RGB(0, 255, 0));
        self.canvas.draw_lines(&line_points[..]).unwrap(); // Use slice to match the expected type

        // Draw lines radiating from a center point in a circle
        let size = 30.0;
        let x = 320.0;
        let y = 95.0 - (size / 2.0);
        let mut rng = rand::thread_rng();

        for i in 0..360 {
            let random_color = Color::RGB(
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
            );
            self.canvas.set_draw_color(random_color);
            self.canvas
                .draw_line(
                    Point::new(x as i32, y as i32),
                    Point::new(
                        (x + (i as f32).to_radians().sin() * size) as i32,
                        (y + (i as f32).to_radians().cos() * size) as i32,
                    ),
                )
                .unwrap();
        }

        // Present the rendered frame
        self.canvas.present();
    }
}

fn main() {
    let sdl_context = sdl3::init().expect("Failed to initialize SDL3");
    let mut app_state = AppState::new(&sdl_context).expect("Failed to create AppState");
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            if !app_state.handle_event(&event) {
                break 'running;
            }
        }
        app_state.iterate();
    }
}
