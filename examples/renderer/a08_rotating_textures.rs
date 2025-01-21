extern crate sdl3;

use sdl3::event::Event;
// use sdl3::pixels::PixelFormat;
use sdl3::render::FPoint;
use sdl3::render::{Canvas, FRect, Texture};
use sdl3::video::Window;
use sdl3::Sdl;
// use sdl3_sys::surface::{SDL_FLIP_HORIZONTAL, SDL_FLIP_NONE, SDL_FLIP_VERTICAL};
use std::error::Error;
use std::path::Path;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

struct AppState {
    canvas: Canvas<Window>,
    texture: Texture,
    texture_width: u32,
    texture_height: u32,
    running: bool,
}

impl AppState {
    fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "Example Renderer Rotating Textures",
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .position_centered()
            .build()?;

        let canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();

        // Load the BMP file into a surface
        // let base_path = sdl3::filesystem::base_path();
        // let bmp_path = format!("{}/sample.bmp", base_path);
        let bmp_path = Path::new("./assets/SDL_logo.bmp");
        let surface = sdl3::surface::Surface::load_bmp(bmp_path)?;

        let texture_width = surface.width();
        let texture_height = surface.height();

        // Convert the surface to a texture
        let texture = texture_creator.create_texture_from_surface(&surface)?;

        Ok(Self {
            canvas,
            texture,
            texture_width,
            texture_height,
            running: true,
        })
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Quit { .. } = event {
            self.running = false;
        }
    }

    fn iterate(&mut self) {
        let now = sdl3::timer::ticks();

        // Calculate rotation (360 degrees over 2000ms)
        let rotation = ((now % 2000) as f64 / 2000.0) * 360.0;

        // Clear the canvas
        self.canvas
            .set_draw_color(sdl3::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Center and draw the texture with rotation
        let dst_rect = FRect::new(
            (WINDOW_WIDTH - self.texture_width) as f32 / 2.0,
            (WINDOW_HEIGHT - self.texture_height) as f32 / 2.0,
            self.texture_width as f32,
            self.texture_height as f32,
        );

        let center = FPoint::new(
            self.texture_width as f32 / 2.0,
            self.texture_height as f32 / 2.0,
        );

        let _ = self.canvas.copy_ex(
            &self.texture,
            None,
            Some(dst_rect),
            rotation,
            Some(center),
            true,
            true,
        );
        // .unwrap();

        self.canvas.present();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl3::init()?;
    let mut app_state = AppState::new(&sdl_context)?;
    let mut event_pump = sdl_context.event_pump()?;

    while app_state.running {
        for event in event_pump.poll_iter() {
            app_state.handle_event(event);
        }

        app_state.iterate();
    }

    Ok(())
}
