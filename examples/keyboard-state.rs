extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3_sys::keycode::SDL_KMOD_NONE;
use std::collections::HashSet;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let _window = video_subsystem
        .window("Keyboard", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;

    let mut prev_keys = HashSet::new();

    'running: loop {
        for event in events.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            };
        }

        // Create a set of pressed Keys.
        let keys = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(|scancode| Keycode::from_scancode(scancode, SDL_KMOD_NONE, false))
            .collect();

        // Get the difference between the new and old sets.
        let new_keys = &keys - &prev_keys;
        let old_keys = &prev_keys - &keys;

        if !new_keys.is_empty() || !old_keys.is_empty() {
            println!("new_keys: {new_keys:?}\told_keys:{old_keys:?}");
        }

        prev_keys = keys;

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
