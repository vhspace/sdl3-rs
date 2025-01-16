extern crate rand;
extern crate sdl3;

use rand::Rng;
use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::{
    render::{Canvas, FPoint},
    video::Window,
    EventPump, Sdl,
};
use std::time::Instant;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const NUM_POINTS: usize = 500;
const MIN_PIXELS_PER_SECOND: f32 = 30.0;
const MAX_PIXELS_PER_SECOND: f32 = 60.0;

struct AppState {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    points: [FPoint; NUM_POINTS],
    point_speeds: [f32; NUM_POINTS],
    last_time: Instant,
}

impl AppState {
    fn new(sdl_context: &Sdl) -> Result<AppState, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Example Renderer Points", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas();
        let event_pump = sdl_context.event_pump().unwrap();

        // Initialize points and speeds
        let mut rng = rand::thread_rng();
        let mut points = [FPoint::new(0.0, 0.0); NUM_POINTS];
        let mut point_speeds = [0.0; NUM_POINTS];

        for i in 0..NUM_POINTS {
            points[i].x = rng.gen_range(0.0..WINDOW_WIDTH as f32);
            points[i].y = rng.gen_range(0.0..WINDOW_HEIGHT as f32);
            point_speeds[i] = rng.gen_range(MIN_PIXELS_PER_SECOND..MAX_PIXELS_PER_SECOND);
        }

        let last_time = Instant::now();

        Ok(AppState {
            canvas,
            event_pump,
            points,
            point_speeds,
            last_time,
        })
    }

    fn handle_event(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return false; // Exit the program
            }
        }
        true
    }

    fn iterate(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f32(); // Seconds since last iteration
        self.last_time = now;

        // Move points
        for i in 0..NUM_POINTS {
            let distance = elapsed * self.point_speeds[i];
            self.points[i].x += distance;
            self.points[i].y += distance;

            if self.points[i].x >= WINDOW_WIDTH as f32 || self.points[i].y >= WINDOW_HEIGHT as f32 {
                // Reset off-screen points
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    self.points[i].x = rng.gen_range(0.0..WINDOW_WIDTH as f32);
                    self.points[i].y = 0.0;
                } else {
                    self.points[i].x = 0.0;
                    self.points[i].y = rng.gen_range(0.0..WINDOW_HEIGHT as f32);
                }
                self.point_speeds[i] = rng.gen_range(MIN_PIXELS_PER_SECOND..MAX_PIXELS_PER_SECOND);
            }
        }

        // Render points
        self.canvas.set_draw_color(Color::RGB(0, 0, 0)); // Black background
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255)); // White points
        self.canvas
            .draw_points(self.points.as_ref()) // Convert array to slice
            .unwrap(); // Draw all points
        self.canvas.present();
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init().unwrap();
    let mut app_state = AppState::new(&sdl_context)?;

    while app_state.handle_event() {
        app_state.iterate();
    }

    Ok(())
}
