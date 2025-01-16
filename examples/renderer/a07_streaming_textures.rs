extern crate sdl3;

use sdl3::event::Event;
use sdl3::pixels::PixelFormat;
use sdl3::render::{Canvas, FRect, Texture};
use sdl3::video::Window;
use sdl3::Sdl;
use sdl3_sys::pixels::SDL_PixelFormat;
use std::error::Error;

const TEXTURE_SIZE: u32 = 150;
const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

struct AppState {
    canvas: Canvas<Window>,
    texture: Texture,
    running: bool,
}

impl AppState {
    fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "examples/renderer/streaming-textures",
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture(
            unsafe { PixelFormat::from_ll(SDL_PixelFormat::RGB24) },
            sdl3::render::TextureAccess::Streaming,
            TEXTURE_SIZE,
            TEXTURE_SIZE,
        )?;

        Ok(Self {
            canvas,
            texture,
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

        // Calculate position for the moving strip
        let direction = if (now % 2000) >= 1000 { 1.0 } else { -1.0 };
        let scale = (((now % 1000) as f32 - 500.0) / 500.0) * direction;

        // Update the texture
        self.texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let bytes_per_pixel = 4;

                // Clear the texture (black background)
                for pixel in buffer.iter_mut() {
                    *pixel = 0;
                }

                // Draw the green strip
                let strip_height = TEXTURE_SIZE / 10;
                let y_position =
                    ((TEXTURE_SIZE - strip_height) as f32 * ((scale + 1.0) / 2.0)) as usize;

                for y in y_position..(y_position + strip_height as usize) {
                    for x in 0..TEXTURE_SIZE as usize {
                        let offset = y * pitch + x * bytes_per_pixel;
                        buffer[offset] = 0; // Red
                        buffer[offset + 1] = 255; // Green
                        buffer[offset + 2] = 0; // Blue
                        buffer[offset + 3] = 255; // Alpha
                    }
                }
            })
            .unwrap();

        // Clear the screen
        self.canvas
            .set_draw_color(sdl3::pixels::Color::RGB(66, 66, 66));
        self.canvas.clear();

        // Draw the updated texture
        let dst_rect = FRect::new(
            (WINDOW_WIDTH - TEXTURE_SIZE) as f32 / 2.0,
            (WINDOW_HEIGHT - TEXTURE_SIZE) as f32 / 2.0,
            TEXTURE_SIZE as f32,
            TEXTURE_SIZE as f32,
        );

        self.canvas
            .copy(&self.texture, None, Some(dst_rect))
            .unwrap();
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
