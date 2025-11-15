extern crate sdl3;

use sdl3::event::{Event, JoyButtonState, JoystickEvent, KeyState, KeyboardEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let joystick_subsystem = sdl_context.joystick()?;
    let haptic_subsystem = sdl_context.haptic()?;

    let joysticks = joystick_subsystem
        .joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {e}"))?;

    println!("{} joysticks available", joysticks.len());

    // Iterate over all available joysticks and stop once we manage to open one.
    let joystick_id = joysticks
        .into_iter()
        .find_map(|id| match joystick_subsystem.open(id) {
            Ok(c) => {
                println!("Success: opened \"{}\"", c.name());
                Some(id)
            }
            Err(e) => {
                println!("failed: {e:?}");
                None
            }
        })
        .expect("Couldn't open any joystick");

    let mut haptic = haptic_subsystem
        .open_from_joystick_id(joystick_id)
        .map_err(|e| e.to_string())?;

    for event in sdl_context.event_pump()?.wait_iter() {
        match event {
            Event::Joystick(JoystickEvent::Axis {
                axis_index, value, ..
            }) => {
                // Axis motion is an absolute value in the range [-32768, 32767].
                let dead_zone = 10_000;
                if value > dead_zone || value < -dead_zone {
                    println!("Axis {axis_index} moved to {value}");
                }
            }
            Event::Joystick(JoystickEvent::Button {
                button_index,
                state: JoyButtonState::Down,
                ..
            }) => {
                println!("Button {button_index} down");
                // Play a short rumble
                haptic.rumble_play(0.5, 500);
            }
            Event::Joystick(JoystickEvent::Button {
                button_index,
                state: JoyButtonState::Up,
                ..
            }) => println!("Button {button_index} up"),
            Event::Joystick(JoystickEvent::Hat {
                hat_index, state, ..
            }) => {
                println!("Hat {hat_index} moved to {state:?}");
            }
            Event::Keyboard(KeyboardEvent {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                state: KeyState::Down,
                ..
            }) => break,
            Event::Quit(_) => break,
            _ => {}
        }
    }

    Ok(())
}
