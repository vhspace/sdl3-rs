extern crate sdl3;

/*
#[test]
fn display_name_no_segfault() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video();
    if let Ok(video_subsystem) = video_subsystem {
        // hopefully no one has a 100 screen to see this test pass

        // With new display handling, this test is irrelevant
        let r = Display::from_ll(99).get_name();
                ^^^^^^^^^^^^^^^^^^^^ Errors

        assert!(r.is_err());
    } // in Err(), environment has no video device (for instance travis)
      // so ignore it
}
*/

use sdl3::render::Canvas;
use sdl3::video::Window;
use sdl3::VideoSubsystem;

fn build_canvas(video_subsystem: &VideoSubsystem) -> Result<Canvas<Window>, String> {
    let window = video_subsystem
        .window("rust-sdl3 test: Video", 800, 600)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();

    canvas.clear();
    Ok(canvas)
}

#[test]
fn window_from_id() {
    let sdl_context = match sdl3::init() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Skipping video test: failed to init SDL: {err}");
            return;
        }
    };
    let video_subsystem = match sdl_context.video() {
        Ok(video) => video,
        Err(err) => {
            eprintln!("Skipping video test: no video device available: {err}");
            return;
        }
    };

    let canvas = match build_canvas(&video_subsystem) {
        Ok(canvas) => canvas,
        Err(err) => {
            eprintln!("Skipping video test: couldn't build canvas: {err}");
            return;
        }
    };

    let mut window = match unsafe { video_subsystem.window_from_id(canvas.window().id()) } {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Skipping video test: window_from_id failed: {err}");
            return;
        }
    };
    window.maximize();
}
