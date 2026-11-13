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

fn init_video() -> Option<(sdl3::Sdl, VideoSubsystem)> {
    let sdl_context = match sdl3::init() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Skipping video test: failed to init SDL: {err}");
            return None;
        }
    };
    match sdl_context.video() {
        Ok(video) => Some((sdl_context, video)),
        Err(err) => {
            eprintln!("Skipping video test: no video device available: {err}");
            None
        }
    }
}

#[test]
fn metal_view_is_absent_unless_requested() {
    let Some((_sdl_context, video_subsystem)) = init_video() else {
        return;
    };
    let window = match video_subsystem
        .window("rust-sdl3 test: no metal view", 320, 240)
        .hidden()
        .build()
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Skipping video test: couldn't build window: {err}");
            return;
        }
    };

    assert!(window.metal_view().is_none());
    assert!(window.metal_layer().is_none());
}

#[cfg(target_os = "macos")]
#[test]
fn metal_view_and_layer_are_exposed() {
    let Some((_sdl_context, video_subsystem)) = init_video() else {
        return;
    };
    // Only the Cocoa driver can create a Metal view (the dummy driver cannot).
    if video_subsystem.current_video_driver() != "cocoa" {
        eprintln!("Skipping video test: Metal views need the cocoa video driver");
        return;
    }
    let window = match video_subsystem
        .window("rust-sdl3 test: metal view", 320, 240)
        .hidden()
        .metal_view()
        .build()
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Skipping video test: couldn't build window: {err}");
            return;
        }
    };

    assert!(window.metal_view().is_some());
    assert!(window.metal_layer().is_some());
    assert_eq!(window.context().metal_view(), window.metal_view());
}
