extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let _window = video_subsystem
        .window("Mouse", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;
    let mut state;

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        // get a mouse state using mouse_state() so as not to call
        // relative_mouse_state() twice and get a false position reading
        if events
            .mouse_state()
            .is_mouse_button_pressed(MouseButton::Left)
        {
            state = events.relative_mouse_state();
            println!("Relative - X = {:?}, Y = {:?}", state.x(), state.y());
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
