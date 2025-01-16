extern crate sdl3;

use sdl3::event::Event;
use sdl3::render::FRect;
use sdl3::render::{Canvas, Texture};
use sdl3::video::Window;
use sdl3::Sdl;
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
                "Example Renderer Scaling & Moving Textures",
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .position_centered()
            .build()?;

        let canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();

        // Load the BMP file into a surface
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

        // Determine direction and scale
        let direction = if (now % 2000) >= 1000 { 1.0 } else { -1.0 };
        let scale = ((now % 1000) as f32 - 500.0) / 500.0 * direction;

        // Calculate horizontal movement
        let movement = ((now % 4000) as f32 / 4000.0) * WINDOW_WIDTH as f32;

        // Clear the canvas
        self.canvas
            .set_draw_color(sdl3::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Calculate the destination rectangle with scaling and horizontal movement
        let width = self.texture_width as f32 + self.texture_width as f32 * scale;
        let height = self.texture_height as f32 + self.texture_height as f32 * scale;
        let x = movement - width / 2.0; // Texture moves across horizontally
        let y = (WINDOW_HEIGHT as f32 - height) / 2.0;

        let dst_rect = FRect::new(x, y, width, height);

        // Render the texture
        self.canvas
            .copy(&self.texture, None, Some(dst_rect))
            .unwrap();

        // Present the updated canvas
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
