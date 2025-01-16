extern crate sdl3;

use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::render::FRect;
use sdl3::video::Window;
use sdl3::Sdl;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

struct AppState {
    canvas: Canvas<Window>,
    running: bool,
}

impl AppState {
    fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Example Renderer Rectangles", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas();

        Ok(Self {
            canvas,
            running: true,
        })
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Quit { .. } = event {
            self.running = false;
        }
    }

    fn iterate(&mut self, now: u64) {
        let mut rects = vec![FRect::new(0.0, 0.0, 0.0, 0.0); 16];

        // Calculate scale based on elapsed time
        let direction = if (now % 2000) >= 1000 { 1.0 } else { -1.0 };
        let scale = (((now % 1000) as f32 - 500.0) / 500.0) * direction;

        // Clear the screen
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw a single rectangle
        rects[0].x = 100.0;
        rects[0].y = 100.0;
        rects[0].w = 100.0 + (100.0 * scale);
        rects[0].h = 100.0 + (100.0 * scale);
        self.canvas.set_draw_color(Color::RGB(255, 0, 0));
        self.canvas
            .draw_rect(Self::convert_frect_to_rect(&rects[0].into()).into())
            .unwrap();

        // Draw several rectangles
        for i in 0..3 {
            let size = (i + 1) as f32 * 50.0;
            rects[i].w = size + (size * scale);
            rects[i].h = size + (size * scale);
            rects[i].x = (WINDOW_WIDTH as f32 - rects[i].w) / 2.0;
            rects[i].y = (WINDOW_HEIGHT as f32 - rects[i].h) / 2.0;
        }
        self.canvas.set_draw_color(Color::RGB(0, 255, 0));
        for rect in &rects[..3] {
            self.canvas
                .draw_rect(Self::convert_frect_to_rect(rect).into())
                .unwrap();
        }

        // Draw a filled rectangle
        rects[0].x = 400.0;
        rects[0].y = 50.0;
        rects[0].w = 100.0 + (100.0 * scale);
        rects[0].h = 50.0 + (50.0 * scale);
        self.canvas.set_draw_color(Color::RGB(0, 0, 255));
        self.canvas
            .fill_rect(Self::convert_frect_to_rect(&rects[0]))
            .unwrap();

        // Draw multiple filled rectangles
        for i in 0..rects.len() {
            let w = (WINDOW_WIDTH as f32 / rects.len() as f32) as f32;
            let h = i as f32 * 8.0;
            rects[i].x = i as f32 * w;
            rects[i].y = WINDOW_HEIGHT as f32 - h;
            rects[i].w = w;
            rects[i].h = h;
        }
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for rect in &rects {
            self.canvas
                .fill_rect(Self::convert_frect_to_rect(rect))
                .unwrap();
        }

        // Present the updated canvas
        self.canvas.present();
    }

    fn convert_frect_to_rect(frect: &FRect) -> Rect {
        Rect::new(
            frect.x as i32,
            frect.y as i32,
            frect.w as u32,
            frect.h as u32,
        )
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init().unwrap();
    let mut app_state = AppState::new(&sdl_context)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let start_time = sdl3::timer::ticks();
    while app_state.running {
        for event in event_pump.poll_iter() {
            app_state.handle_event(event);
        }

        let now = sdl3::timer::ticks() - start_time;
        app_state.iterate(now as u64);
    }

    Ok(())
}
