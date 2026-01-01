extern crate sdl3;

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
            println!("Attempting to open gamepad {}", id.0);

            match gamepad_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
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
        controller.mapping().unwrap_or("".to_owned())
    );

    let (mut lo_freq, mut hi_freq) = (0, 0);

    for event in sdl_context.event_pump()?.wait_iter() {
        use sdl3::event::Event;
        use sdl3::gamepad::Axis;

        match event {
            Event::ControllerAxisMotion {
                axis: Axis::TriggerLeft,
                value: val,
                ..
            } => {
                // Trigger axes go from 0 to 32767, so this should be okay
                lo_freq = (val as u16) * 2;
                match controller.set_rumble(lo_freq, hi_freq, 15000) {
                    Ok(()) => println!("Set rumble to ({lo_freq}, {hi_freq})"),
                    Err(e) => println!("Error setting rumble to ({lo_freq}, {hi_freq}): {e:?}"),
                }
            }
            Event::ControllerAxisMotion {
                axis: Axis::TriggerRight,
                value: val,
                ..
            } => {
                // Trigger axes go from 0 to 32767, so this should be okay
                hi_freq = (val as u16) * 2;
                match controller.set_rumble(lo_freq, hi_freq, 15000) {
                    Ok(()) => println!("Set rumble to ({lo_freq}, {hi_freq})"),
                    Err(e) => println!("Error setting rumble to ({lo_freq}, {hi_freq}): {e:?}"),
                }
            }
            Event::ControllerAxisMotion {
                axis, value: val, ..
            } => {
                // Axis motion is an absolute value in the range
                // [-32768, 32767]. Let's simulate a very rough dead
                // zone to ignore spurious events.
                let dead_zone = 10_000;
                if val > dead_zone || val < -dead_zone {
                    println!("Axis {axis:?} moved to {val}");
                }
            }
            Event::ControllerButtonDown { button, .. } => println!("Button {button:?} down"),
            Event::ControllerButtonUp { button, .. } => println!("Button {button:?} up"),
            Event::ControllerTouchpadDown {
                touchpad,
                finger,
                x,
                y,
                ..
            } => println!("Touchpad {touchpad} down finger:{finger} x:{x} y:{y}"),
            Event::ControllerTouchpadMotion {
                touchpad,
                finger,
                x,
                y,
                ..
            } => println!("Touchpad {touchpad} move finger:{finger} x:{x} y:{y}"),
            Event::ControllerTouchpadUp {
                touchpad,
                finger,
                x,
                y,
                ..
            } => println!("Touchpad {touchpad} up   finger:{finger} x:{x} y:{y}"),
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}
