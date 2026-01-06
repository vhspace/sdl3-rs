use sdl3::{
    event::Event,
    keyboard::Keycode,
    messagebox::{show_simple_message_box, MessageBoxFlag},
    pixels::Color,
    render::WindowCanvas,
    timer::ticks,
    Sdl, VideoSubsystem,
};
use sdl3_main::{app_impl, AppResult, AppResultWithState, MainThreadData};
use std::sync::Mutex;

struct App {
    main: MainThreadData<MainData>,
    mx: f32,
    my: f32,
}

struct MainData {
    canvas: WindowCanvas,
    _video: VideoSubsystem,
    _sdl: Sdl,
}

#[app_impl]
impl App {
    fn new() -> Result<Self, Box<dyn ::core::error::Error>> {
        let sdl = sdl3::init()?;
        let video = sdl.video()?;
        let canvas = video.window_and_renderer("Hello callbacks api!", 640, 480)?;
        Ok(Self {
            main: MainThreadData::assert_new(MainData {
                canvas,
                _video: video,
                _sdl: sdl,
            }),
            mx: 0.0,
            my: 0.0,
        })
    }

    fn app_init() -> AppResultWithState<Box<Mutex<Self>>> {
        match Self::new() {
            Ok(app) => AppResultWithState::Continue(Box::new(Mutex::new(app))),
            Err(err) => {
                let error_msg = format!("Error initializing SDL: {err:?}");
                eprintln!("{error_msg}");
                let _ = show_simple_message_box(MessageBoxFlag::ERROR, "Error!", &error_msg, None);
                AppResultWithState::Failure(None)
            }
        }
    }

    fn app_iterate(&mut self) {
        let main = self.main.assert_get_mut();
        let canvas = &mut main.canvas;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(Color::WHITE);
        let _ = canvas.draw_debug_text(&format!("Callbacks running for {} ms", ticks()), (4, 4));
        let _ = canvas.draw_debug_text(&format!("Mouse x: {}", self.mx), (4, 20));
        let _ = canvas.draw_debug_text(&format!("      y: {}", self.my), (4, 28));
        canvas.present();
    }

    // event can also be taken by value or by mut reference
    fn app_event(&mut self, event: &Event) -> AppResult {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => AppResult::Success,

            Event::MouseMotion { x, y, .. } => {
                self.mx = *x;
                self.my = *y;
                AppResult::Continue
            }

            _ => AppResult::Continue,
        }
    }

    // app_quit is optional if dropping App does sufficient cleanup
}
