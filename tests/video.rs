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

fn build_canvas(video_subsystem: VideoSubsystem) -> Canvas<Window> {
    let window = video_subsystem
        .window("rust-sdl3 test: Video", 800, 600)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window.into_canvas();

    canvas.clear();
    canvas
}

#[test]
fn window_from_id() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let canvas = build_canvas(video_subsystem.clone());

    let mut window = unsafe {
        video_subsystem
            .window_from_id(canvas.window().id())
            .unwrap()
    };
    window.maximize();
}
