extern crate sdl3;

use std::path::Path;

use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::render::FRect;
use sdl3::render::{Canvas, Texture};
use sdl3::video::Window;
use sdl3::Sdl;

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
    fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Example Renderer Textures", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas();

        // let base_path = sdl3::filesystem::get_base_path().map_err(|e| format!("{:?}", e))?;
        // let bmp_path = format!("{:?}../../assets/thumbnail.png", base_path);
        let bmp_path = Path::new("./assets/sample640_426.bmp");
        let surface = sdl3::surface::Surface::load_bmp(bmp_path).map_err(|e| e.to_string())?;

        let texture_width = surface.width();
        let texture_height = surface.height();

        // Create the texture creator first
        let texture_creator = canvas.texture_creator();

        // Now, use it to create the texture. This will not drop the temporary value.
        let texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;

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
        let now = sdl3::timer::ticks() as u64;

        // Calculate scale based on elapsed time
        let direction = if (now % 2000) >= 1000 { 1.0 } else { -1.0 };
        let scale = (((now % 1000) as f32 - 500.0) / 500.0) * direction;

        // Clear the screen
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw the texture in different positions
        let mut dst_rect = FRect::new(
            0.0,
            0.0,
            self.texture_width as f32,
            self.texture_height as f32,
        );

        // Top left
        dst_rect.x = 100.0 * scale;
        dst_rect.y = 0.0;
        self.canvas
            .copy_ex(&self.texture, None, Some(dst_rect), 0.0, None, false, false)
            .unwrap();

        // Center
        dst_rect.x = (WINDOW_WIDTH as f32 - self.texture_width as f32) / 2.0;
        dst_rect.y = (WINDOW_HEIGHT as f32 - self.texture_height as f32) / 2.0;
        self.canvas
            .copy_ex(&self.texture, None, Some(dst_rect), 0.0, None, false, false)
            .unwrap();

        // Bottom right
        dst_rect.x = (WINDOW_WIDTH as f32 - self.texture_width as f32) - (100.0 * scale);
        dst_rect.y = WINDOW_HEIGHT as f32 - self.texture_height as f32;
        self.canvas
            .copy_ex(&self.texture, None, Some(dst_rect), 0.0, None, false, false)
            .unwrap();

        // Present the updated canvas
        self.canvas.present();
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init().unwrap();
    let mut app_state = AppState::new(&sdl_context)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    while app_state.running {
        for event in event_pump.poll_iter() {
            app_state.handle_event(event);
        }

        app_state.iterate();
    }

    Ok(())
}
