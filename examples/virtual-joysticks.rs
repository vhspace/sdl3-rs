use sdl3::{
    gamepad::{Axis, Button},
    joystick::{HatState, VirtualJoystickDescription},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n<===> VIRTUAL JOYSTICKS <===>\n");
    let sdl_context = sdl3::init()?;
    let joystick_subsystem = sdl_context.joystick()?;

    let axes: Vec<Axis> = vec![Axis::LeftX, Axis::LeftY];
    let buttons: Vec<Button> = vec![Button::North, Button::South, Button::East, Button::West];

    {
        let desc = VirtualJoystickDescription::new()
            .num_hats(1)
            .with_axes(axes.clone())
            .with_buttons(buttons.clone())
            .name("My Virtual Joystick");

        println!(
            "Current number of joysticks: {}\n",
            joystick_subsystem.joysticks().unwrap().len()
        );

        // `connection` is of type VirtualJoystickConnection, which behaves like a physical connection
        // between a physical device and your PC. If it leaves scope and is dropped, it is analogous to
        // you unplugging your mouse from your PC while a program is currently using it.
        let connection = joystick_subsystem.attach_virtual_joystick(desc)?;
        println!("Successfully built a new virtual joystick.");

        println!(
            "New number of joysticks: {}\n",
            joystick_subsystem.joysticks().unwrap().len()
        );

        let id = connection.id();
        let joystick = joystick_subsystem.open(id)?;

        println!(
            "Joystick:\n   Name: {}\n   Number of Axes: {}\n   Number of Buttons: {}\n   Number of Hats: {}\n",
            joystick.name(),
            joystick.num_axes(),
            joystick.num_buttons(),
            joystick.num_hats()
        );

        let axes_iter = axes.iter();
        let buttons_iter = buttons.iter();

        let axis_codes: Vec<u32> = axes_iter.map(|axis| axis.to_ll().0 as u32).collect();
        let button_codes: Vec<u32> = buttons_iter.map(|button| button.to_ll().0 as u32).collect();

        println!("Values before write:");
        println!(
            "{:<3}Axes (LX/LY):      {}/{}",
            "",
            joystick.axis(axis_codes[0]).unwrap(), // Axis::LeftX
            joystick.axis(axis_codes[1]).unwrap()  // Axis::LeftY
        );

        println!(
            "{:<3}Buttons (N/S/E/W): {}/{}/{}/{}",
            "",
            joystick.button(button_codes[0]).unwrap(), // Button::North
            joystick.button(button_codes[1]).unwrap(), // Button::South
            joystick.button(button_codes[2]).unwrap(), // Button::East
            joystick.button(button_codes[3]).unwrap()  // Button::West
        );
        println!(
            "{:<3}Hat 0:             {:?}\n",
            "",
            joystick.hat(0).unwrap()
        );

        println!("Writing new states to the virtual joystick...\n");
        joystick.set_virtual_axis(axis_codes[0], 100)?; // Axis::LeftX
        joystick.set_virtual_axis(axis_codes[1], 200)?; // Axis::LeftY
        joystick.set_virtual_button(button_codes[0], true)?; // Button::North
        joystick.set_virtual_button(button_codes[1], true)?; // Button::South
        joystick.set_virtual_button(button_codes[2], true)?; // Button::East
        joystick.set_virtual_button(button_codes[3], true)?; // Button::West
        joystick.set_virtual_hat(0, HatState::Up)?;

        // update() must be called for the new states to be flushed
        joystick_subsystem.update();

        println!("Values after write:");
        println!(
            "{:<3}Axes (LX/LY):      {}/{}",
            "",
            joystick.axis(axis_codes[0]).unwrap(), // Axis::LeftX
            joystick.axis(axis_codes[1]).unwrap()  // Axis::LeftY
        );

        println!(
            "{:<3}Buttons (N/S/E/W): {}/{}/{}/{}",
            "",
            joystick.button(button_codes[0]).unwrap(), // Button::North
            joystick.button(button_codes[1]).unwrap(), // Button::South
            joystick.button(button_codes[2]).unwrap(), // Button::East
            joystick.button(button_codes[3]).unwrap()  // Button::West
        );
        println!(
            "{:<3}Hat 0:             {:?}\n",
            "",
            joystick.hat(0).unwrap()
        );

        println!("Lifetime of Virtual Joystick Connection ends here.")
    }

    // By this point, the the Virtual Joystick Connection has been dropped, so it should no longer
    // be counted in joystick_subsystem().joysticks()
    println!(
        "Final number of joysticks: {}",
        joystick_subsystem.joysticks().unwrap().len()
    );
    Ok(())
}
