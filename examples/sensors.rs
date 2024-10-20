extern crate sdl3;
use std::time::{Duration, Instant};

use sdl3::{event::Event, sensor::SensorType};

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let game_controller_subsystem = sdl_context.gamepad()?;

    let available = game_controller_subsystem
        .num_gamepads()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))?;

    println!("{} joysticks available", available);

    // Iterate over all available joysticks and look for game controllers.
    let controller = (0..available)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                println!("{} is not a game controller", id);
                return None;
            }

            println!("Attempting to open controller {}", id);

            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    println!("Success: opened \"{}\"", c.name());
                    Some(c)
                }
                Err(e) => {
                    println!("failed: {:?}", e);
                    None
                }
            }
        })
        .expect("Couldn't open any controller");

    unsafe {
        if !controller.has_sensor(SensorType::Accelerometer) {
            return Err(format!(
                "{} doesn't support the accelerometer",
                controller.name()
            ));
        }
    }
    unsafe {
        if !controller.has_sensor(SensorType::Gyroscope) {
            return Err(format!(
                "{} doesn't support the gyroscope",
                controller.name()
            ));
        }
    }

    controller
        .sensor_set_enabled(SensorType::Accelerometer, true)
        .map_err(|e| format!("error enabling accelerometer: {}", e))?;
    controller
        .sensor_set_enabled(SensorType::Gyroscope, true)
        .map_err(|e| format!("error enabling gyroscope: {}", e))?;
    let mut now = Instant::now();
    for event in sdl_context.event_pump()?.wait_iter() {
        if false && now.elapsed() > Duration::from_secs(1) {
            now = Instant::now();

            let mut gyro_data = [0f32; 3];
            let mut accel_data = [0f32; 3];

            controller
                .sensor_get_data(SensorType::Gyroscope, &mut gyro_data)
                .map_err(|e| format!("error getting gyro data: {}", e))?;
            controller
                .sensor_get_data(SensorType::Accelerometer, &mut accel_data)
                .map_err(|e| format!("error getting accel data: {}", e))?;

            println!("gyro: {:?}, accel: {:?}", gyro_data, accel_data);
        }

        if let Event::ControllerSensorUpdated { .. } = event {
            println!("{:?}", event);
        }

        if let Event::Quit { .. } = event {
            break;
        }
    }

    Ok(())
}
