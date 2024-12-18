// https://github.com/tsoding/midpoint-circle-visualization

extern crate sdl3;

use sdl3::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
};

use std::time::Duration;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

// Function to fill a triangle

fn fill_triangle(
    canvas: &mut Canvas<Window>,
    p1: (i32, i32),
    p2: (i32, i32),
    p3: (i32, i32),
    color: Color,
) {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let (x3, y3) = p3;

    // Sort the vertices by y-coordinate ascending (y1 <= y2 <= y3)
    let (x1, y1, x2, y2, x3, y3) = if y1 > y2 {
        if y2 > y3 {
            (x3, y3, x2, y2, x1, y1)
        } else if y1 > y3 {
            (x2, y2, x3, y3, x1, y1)
        } else {
            (x2, y2, x1, y1, x3, y3)
        }
    } else {
        if y1 > y3 {
            (x3, y3, x1, y1, x2, y2)
        } else if y2 > y3 {
            (x1, y1, x3, y3, x2, y2)
        } else {
            (x1, y1, x2, y2, x3, y3)
        }
    };

    // Helper function to draw a horizontal line
    let mut draw_horizontal_line = |y: i32, x_min: i32, x_max: i32| {
        for x in x_min..=x_max {
            canvas.draw_point(Point::new(x, y)).unwrap();
        }
    };

    // Fill bottom flat triangle
    if y2 != y1 {
        for y in y1..=y2 {
            let alpha = (y - y1) as f32 / (y2 - y1) as f32;
            let beta = (y - y1) as f32 / (y3 - y1) as f32;

            let x_start = x1 + ((x2 - x1) as f32 * alpha) as i32;
            let x_end = x1 + ((x3 - x1) as f32 * beta) as i32;

            draw_horizontal_line(y, x_start.min(x_end), x_start.max(x_end));
        }
    }

    // Fill top flat triangle
    if y3 != y2 {
        for y in y2..=y3 {
            let alpha = (y - y2) as f32 / (y3 - y2) as f32;
            let beta = (y - y1) as f32 / (y3 - y1) as f32;

            let x_start = x2 + ((x3 - x2) as f32 * alpha) as i32;
            let x_end = x1 + ((x3 - x1) as f32 * beta) as i32;

            draw_horizontal_line(y, x_start.min(x_end), x_start.max(x_end));
        }
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo: draw Triangle", WINDOW_WIDTH, WINDOW_HEIGHT)
        .resizable()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas();
    //map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    fill_triangle(
        &mut canvas,
        (100, 100),
        (200, 200),
        (300, 100),
        Color::RGB(255, 0, 0),
    );

    canvas.present();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
