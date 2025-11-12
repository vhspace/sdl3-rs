extern crate sdl3;

use sdl3::event::{
    ControllerButtonState, ControllerEvent, ControllerTouchpadKind, Event, KeyState, KeyboardEvent,
};
use sdl3::gamepad::Axis;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This is required for certain controllers to work on Windows without the
    // video subsystem enabled:
    sdl3::hint::set("SDL_JOYSTICK_THREAD", "1");

    let sdl_context = sdl3::init()?;
    let gamepad_subsystem = sdl_context.gamepad()?;

    let gamepad_joysticks_ids = gamepad_subsystem
        .gamepads()
        .map_err(|e| format!("can't enumerate gamepads: {e}"))?;

    println!("{} gamepads available", gamepad_joysticks_ids.len());

    // Iterate over all available joysticks and look for game controllers.
    let mut controller = gamepad_joysticks_ids
        .into_iter()
        .find_map(|id| {
            println!("Attempting to open gamepad {id}");

            match gamepad_subsystem.open(id) {
                Ok(c) => {
                    println!(
                        "Success: opened \"{}\"",
                        c.name().unwrap_or_else(|| "(unnamed)".to_owned())
                    );
                    Some(c)
                }
                Err(e) => {
                    println!("failed: {e:?}");
                    None
                }
            }
        })
        .expect("Couldn't open any gamepad");

    println!(
        "Controller mapping: {}",
        controller.mapping().unwrap_or_default()
    );

    let (mut lo_freq, mut hi_freq) = (0u16, 0u16);

    for event in sdl_context.event_pump()?.wait_iter() {
        match event {
            Event::Controller(ControllerEvent::Axis {
                axis: Axis::TriggerLeft,
                value: val,
                ..
            }) => {
                lo_freq = (val as u16) * 2;
                if let Err(e) = controller.set_rumble(lo_freq, hi_freq, 15_000) {
                    println!("Error setting rumble to ({lo_freq}, {hi_freq}): {e:?}");
                } else {
                    println!("Set rumble to ({lo_freq}, {hi_freq})");
                }
            }
            Event::Controller(ControllerEvent::Axis {
                axis: Axis::TriggerRight,
                value: val,
                ..
            }) => {
                hi_freq = (val as u16) * 2;
                if let Err(e) = controller.set_rumble(lo_freq, hi_freq, 15_000) {
                    println!("Error setting rumble to ({lo_freq}, {hi_freq}): {e:?}");
                } else {
                    println!("Set rumble to ({lo_freq}, {hi_freq})");
                }
            }
            Event::Controller(ControllerEvent::Axis { axis, value, .. }) => {
                let dead_zone = 10_000;
                if value > dead_zone || value < -dead_zone {
                    println!("Axis {axis:?} moved to {value}");
                }
            }
            Event::Controller(ControllerEvent::Button {
                button,
                state: ControllerButtonState::Down,
                ..
            }) => println!("Button {button:?} down"),
            Event::Controller(ControllerEvent::Button {
                button,
                state: ControllerButtonState::Up,
                ..
            }) => println!("Button {button:?} up"),
            Event::Controller(ControllerEvent::Touchpad {
                touchpad,
                finger,
                kind: ControllerTouchpadKind::Down,
                x,
                y,
                ..
            }) => println!("Touchpad {touchpad} down finger:{finger} x:{x} y:{y}"),
            Event::Controller(ControllerEvent::Touchpad {
                touchpad,
                finger,
                kind: ControllerTouchpadKind::Motion,
                x,
                y,
                ..
            }) => println!("Touchpad {touchpad} move finger:{finger} x:{x} y:{y}"),
            Event::Controller(ControllerEvent::Touchpad {
                touchpad,
                finger,
                kind: ControllerTouchpadKind::Up,
                x,
                y,
                ..
            }) => println!("Touchpad {touchpad} up   finger:{finger} x:{x} y:{y}"),
            Event::Quit(_) => break,
            // Allow escape key to quit even if no controller events occur
            Event::Keyboard(KeyboardEvent {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                state: KeyState::Down,
                ..
            }) => break,
            _ => {}
        }
    }

    Ok(())
}
