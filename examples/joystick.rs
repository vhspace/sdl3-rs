extern crate sdl3;

use sdl3::event::{Event, JoyButtonState, JoystickEvent, KeyState, KeyboardEvent};
use sdl3::get_error;
use sdl3::keyboard::Keycode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let joystick_subsystem = sdl_context.joystick()?;

    let joysticks = joystick_subsystem
        .joysticks()
        .map_err(|e| format!("can't get joysticks: {e}"))?;

    println!("{} joysticks available", joysticks.len());

    // Iterate over all available joysticks and stop once we manage to open one.
    let mut joystick = joysticks
        .into_iter()
        .find_map(|joystick_id| match joystick_subsystem.open(joystick_id) {
            Ok(c) => {
                println!("Success: opened \"{}\"", c.name());
                Some(c)
            }
            Err(e) => {
                println!("failed: {e:?}");
                None
            }
        })
        .expect("Couldn't open any joystick");

    // Print the joystick's power level
    println!(
        "\"{}\" power level: {:?}",
        joystick.name(),
        joystick.power_info().map_err(|e| e.to_string())?
    );

    let (mut lo_freq, mut hi_freq) = (0u16, 0u16);

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
                if button_index == 0 {
                    lo_freq = 65535;
                } else if button_index == 1 {
                    hi_freq = 65535;
                }
                if button_index < 2 {
                    if joystick.set_rumble(lo_freq, hi_freq, 15_000) {
                        println!("Set rumble to ({lo_freq}, {hi_freq})");
                    } else {
                        println!(
                            "Error setting rumble to ({}, {}): {:?}",
                            lo_freq,
                            hi_freq,
                            get_error()
                        );
                    }
                }
            }
            Event::Joystick(JoystickEvent::Button {
                button_index,
                state: JoyButtonState::Up,
                ..
            }) => {
                println!("Button {button_index} up");
                if button_index == 0 {
                    lo_freq = 0;
                } else if button_index == 1 {
                    hi_freq = 0;
                }
                if button_index < 2 {
                    if joystick.set_rumble(lo_freq, hi_freq, 15_000) {
                        println!("Set rumble to ({lo_freq}, {hi_freq})");
                    } else {
                        println!(
                            "Error setting rumble to ({}, {}): {:?}",
                            lo_freq,
                            hi_freq,
                            get_error()
                        );
                    }
                }
            }
            Event::Joystick(JoystickEvent::Hat {
                hat_index, state, ..
            }) => {
                println!("Hat {hat_index} moved to {state:?}");
            }
            Event::Keyboard(KeyboardEvent {
                keycode: Some(Keycode::Escape),
                state: KeyState::Down,
                ..
            }) => break,
            Event::Quit(_) => break,
            _ => {}
        }
    }

    Ok(())
}
