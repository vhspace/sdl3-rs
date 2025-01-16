extern crate sdl3;

use sdl3::dialog::{
    show_open_file_dialog, show_open_folder_dialog, show_save_file_dialog, DialogFileFilter,
};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::path::PathBuf;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo: Dialog", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let filters = [
        DialogFileFilter {
            name: "Text",
            pattern: "txt",
        },
        DialogFileFilter {
            name: "Videos",
            pattern: "mp4;mkv",
        },
        DialogFileFilter {
            name: "All",
            pattern: "*",
        },
    ];

    let default_path_path = PathBuf::from("/");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::O),
                    ..
                } => {
                    show_open_file_dialog(
                        &filters,
                        None::<PathBuf>,
                        true,
                        canvas.window(),
                        Box::new(|result, filter| {
                            match result {
                                Ok(result) => {
                                    println!("Files: {result:?} Filter: {filter:?}");
                                }
                                Err(error) => {
                                    eprintln!("File dialog error {error}");
                                }
                            };
                        }),
                    )
                    .unwrap_or_else(|e| panic!("Failed to show open file dialog: {e}"));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    show_open_folder_dialog(
                        Some(&default_path_path),
                        false,
                        canvas.window(),
                        Box::new(|result, _| {
                            match result {
                                Ok(result) => {
                                    println!("Folder: {result:?}");
                                }
                                Err(error) => {
                                    eprintln!("Folder dialog error {error}");
                                }
                            };
                        }),
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    show_save_file_dialog(
                        &filters,
                        Some("/home"),
                        canvas.window(),
                        Box::new(|result, filter| {
                            match result {
                                Ok(result) => {
                                    println!("Save File: {result:?} Filter: {filter:?}");
                                }
                                Err(error) => {
                                    eprintln!("Save dialog error {error}");
                                }
                            };
                        }),
                    )
                    .unwrap_or_else(|e| panic!("Failed to show save file dialog: {e}"));
                }
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
