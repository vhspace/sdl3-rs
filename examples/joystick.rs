use sdl3::get_error;

extern crate sdl3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let joystick_subsystem = sdl_context.joystick()?;

    let joysticks = joystick_subsystem
        .joysticks()
        .map_err(|e| format!("can't get joysticks: {}", e))?;

    println!("{} joysticks available", joysticks.len());

    // Iterate over all available joysticks and stop once we manage to open one.
    let mut joystick = joysticks
        .into_iter()
        .find_map(|joystick| match joystick_subsystem.open(joystick) {
            Ok(c) => {
                println!("Success: opened \"{}\"", c.name());
                Some(c)
            }
            Err(e) => {
                println!("failed: {:?}", e);
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

    let (mut lo_freq, mut hi_freq) = (0, 0);

    for event in sdl_context.event_pump()?.wait_iter() {
        use sdl3::event::Event;

        match event {
            Event::JoyAxisMotion {
                axis_idx,
                value: val,
                ..
            } => {
                // Axis motion is an absolute value in the range
                // [-32768, 32767]. Let's simulate a very rough dead
                // zone to ignore spurious events.
                let dead_zone = 10_000;
                if val > dead_zone || val < -dead_zone {
                    println!("Axis {} moved to {}", axis_idx, val);
                }
            }
            Event::JoyButtonDown { button_idx, .. } => {
                println!("Button {} down", button_idx);
                if button_idx == 0 {
                    lo_freq = 65535;
                } else if button_idx == 1 {
                    hi_freq = 65535;
                }
                if button_idx < 2 {
                    if joystick.set_rumble(lo_freq, hi_freq, 15000) {
                        println!("Set rumble to ({}, {})", lo_freq, hi_freq);
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
            Event::JoyButtonUp { button_idx, .. } => {
                println!("Button {} up", button_idx);
                if button_idx == 0 {
                    lo_freq = 0;
                } else if button_idx == 1 {
                    hi_freq = 0;
                }
                if button_idx < 2 {
                    if joystick.set_rumble(lo_freq, hi_freq, 15000) {
                        println!("Set rumble to ({}, {})", lo_freq, hi_freq);
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
            Event::JoyHatMotion { hat_idx, state, .. } => {
                println!("Hat {} moved to {:?}", hat_idx, state)
            }
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}
