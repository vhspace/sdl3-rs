// https://en.wikipedia.org/wiki/Rotation_matrix
// fork demos https://www.desmos.com/calculator/lhsycydnsk
// orginal code https://www.desmos.com/calculator/vp7yjxkq9h
// orginal C code https://github.com/servetgulnaroglu/cube.c

extern crate sdl3;

use sdl3::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
};
use std::time::Duration;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

const CUBE_SIZE: f32 = 1.0;
const VIEWER_DISTANCE: f32 = 5.0;

#[derive(Clone, Copy)]

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    fn rotate(&self, angle_x: f32, angle_y: f32, angle_z: f32) -> Vertex {
        let (sin_x, cos_x) = angle_x.sin_cos();
        let (sin_y, cos_y) = angle_y.sin_cos();
        let (sin_z, cos_z) = angle_z.sin_cos();

        // Rotation around X axis
        let y1 = self.y * cos_x - self.z * sin_x;
        let z1 = self.y * sin_x + self.z * cos_x;

        // Rotation around Y axis
        let x2 = self.x * cos_y + z1 * sin_y;
        let z2 = -self.x * sin_y + z1 * cos_y;

        // Rotation around Z axis
        let x3 = x2 * cos_z - y1 * sin_z;
        let y3 = x2 * sin_z + y1 * cos_z;

        Vertex {
            x: x3,
            y: y3,
            z: z2,
        }
    }

    fn project(&self, width: u32, height: u32, fov: f32, viewer_distance: f32) -> (i32, i32) {
        let factor = fov / (viewer_distance + self.z);
        let x = self.x * factor * width as f32 / 2.0 + width as f32 / 2.0;
        let y = -self.y * factor * height as f32 / 2.0 + height as f32 / 2.0;
        (x as i32, y as i32)
    }
}

fn draw_line(canvas: &mut Canvas<Window>, p1: (i32, i32), p2: (i32, i32), color: Color) {
    canvas.set_draw_color(color);
    canvas
        .draw_line(Point::new(p1.0, p1.1), Point::new(p2.0, p2.1))
        .unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("SDL3 Rotating Cube", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump()?;

    #[rustfmt::skip]
    let vertices = [
        // vertex 0
        Vertex {
            x: -CUBE_SIZE, y: -CUBE_SIZE, z: -CUBE_SIZE,
        },
        // vertex 1
        Vertex {
            x: CUBE_SIZE, y: -CUBE_SIZE, z: -CUBE_SIZE,
        },
        // vertex 2
        Vertex {
            x: CUBE_SIZE, y: CUBE_SIZE, z: -CUBE_SIZE,
        },
        // vertex 3
        Vertex {
            x: -CUBE_SIZE, y: CUBE_SIZE, z: -CUBE_SIZE,
        },
        // vertex 4
        Vertex {
            x: -CUBE_SIZE, y: -CUBE_SIZE, z: CUBE_SIZE,
        },
        // vertex 5
        Vertex {
            x: CUBE_SIZE, y: -CUBE_SIZE, z: CUBE_SIZE,
        },
        // vertex 6
        Vertex {
            x: CUBE_SIZE, y: CUBE_SIZE, z: CUBE_SIZE,
        },
        // vertex 7
        Vertex {
            x: -CUBE_SIZE, y: CUBE_SIZE, z: CUBE_SIZE,
        },
    ];

    #[rustfmt::skip]
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    let mut angle_x = 0.0;
    let mut angle_y = 0.0;
    let mut angle_z = 0.0;

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
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let transformed_vertices: Vec<Vertex> = vertices
            .iter()
            .map(|&v| v.rotate(angle_x, angle_y, angle_z))
            .collect();

        for &(start, end) in &edges {
            let p1 = transformed_vertices[start].project(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                1.0,
                VIEWER_DISTANCE,
            );
            let p2 = transformed_vertices[end].project(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                1.0,
                VIEWER_DISTANCE,
            );
            draw_line(&mut canvas, p1, p2, Color::RGB(255, 255, 255));
        }

        canvas.present();

        angle_x += 0.02;
        angle_y += 0.03;
        angle_z += 0.01;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
