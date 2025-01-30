// original code : https://github.com/libsdl-org/SDL/tree/main/examples/demo/02-woodeneye-008

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::f64::consts::PI;
use std::time::{Duration, Instant};

// Constants defining map size, player count, and drawing precision
const MAP_BOX_SCALE: i32 = 16; // Size of the map box
const MAP_BOX_EDGES_LEN: usize = 12 + (MAP_BOX_SCALE * 2) as usize; // Number of map edges
const MAX_PLAYER_COUNT: usize = 4; // Maximum number of players
const CIRCLE_DRAW_SIDES: usize = 32; // Number of sides for drawing circles
const CIRCLE_DRAW_SIDES_LEN: usize = CIRCLE_DRAW_SIDES + 1; // Number of points for drawing circles

// Structure representing a player
#[derive(Clone, Copy)]
struct Player {
    mouse: u32,     // ID of the mouse associated with the player
    keyboard: u32,  // ID of the keyboard associated with the player
    pos: [f64; 3],  // 3D position of the player (x, y, z)
    vel: [f64; 3],  // 3D velocity of the player (x, y, z)
    yaw: u32,       // Horizontal rotation of the player (angle)
    pitch: i32,     // Vertical rotation of the player (angle)
    radius: f32,    // Radius of the player's collision circle
    height: f32,    // Height of the player
    color: [u8; 3], // RGB color of the player
    wasd: u8,       // Bitmask representing WASD key presses (Up, Left, Down, Right)
}

// Structure holding the application state
struct AppState {
    canvas: Canvas<Window>,               // SDL canvas for rendering
    player_count: usize,                  // Current number of players in the game
    players: [Player; MAX_PLAYER_COUNT],  // Array of players
    edges: [[f32; 6]; MAP_BOX_EDGES_LEN], // Array of map edges (start and end points)
}

// Function to find a player by their mouse ID
fn whose_mouse(mouse: u32, players: &[Player], players_len: usize) -> Option<usize> {
    players.iter().position(|p| p.mouse == mouse)
}

// Function to find a player by their keyboard ID
fn whose_keyboard(keyboard: u32, players: &[Player], players_len: usize) -> Option<usize> {
    players.iter().position(|p| p.keyboard == keyboard)
}

// Function to handle shooting (simplified hit detection)
fn shoot(shooter: usize, players: &mut [Player], players_len: usize) {
    let x0 = players[shooter].pos[0]; // Shooter's x position
    let y0 = players[shooter].pos[1]; // Shooter's y position
    let z0 = players[shooter].pos[2]; // Shooter's z position

    // Convert yaw and pitch to radians
    let bin_rad = PI / 2147483648.0;
    let yaw_rad = (bin_rad as f64) * (players[shooter].yaw) as f64;
    let pitch_rad = (bin_rad as f64) * players[shooter].pitch as f64;

    // Calculate shooting direction vector
    let cos_yaw = yaw_rad.cos();
    let sin_yaw = yaw_rad.sin();
    let cos_pitch = pitch_rad.cos();
    let sin_pitch = pitch_rad.sin();
    let vx = -sin_yaw * cos_pitch;
    let vy = sin_pitch;
    let vz = -cos_yaw * cos_pitch;

    // Iterate through other players to check for hits
    for i in 0..players_len {
        if i == shooter {
            continue; // Skip the shooter themselves
        }
        let target = &mut players[i]; // Get a mutable reference to the target player
        let mut hit = 0; // Initialize hit counter for head and feet check
        for j in 0..2 {
            // Check head and feet
            let r = target.radius as f64; // Target's radius
            let h = target.height as f64; // Target's height
            let dx = target.pos[0] - x0; // Difference in x position
            let dy = target.pos[1] - y0 + if j == 0 { 0.0 } else { r - h }; // Difference in y position (adjust for head/feet)
            let dz = target.pos[2] - z0; // Difference in z position
            let vd = vx * dx as f64 + vy * dy as f64 + vz * dz as f64; // Dot product of velocity and distance vectors
            let dd = dx * dx + dy * dy + dz * dz; // Squared distance between shooter and target
            let vv = vx * vx + vy * vy + vz * vz; // Squared magnitude of velocity vector
            let rr = r * r; // Squared radius

            // Simplified hit detection (cone intersection with player's bounding sphere)
            if vd < 0.0 {
                continue; // If the target is moving away, skip
            }
            if vd * vd >= vv * (dd as f64 - rr as f64) {
                hit += 1; // Increment hit counter if the shot hits the target
            }
        }
        if hit > 0 {
            // If hit, reset the target's position to a random location
            target.pos[0] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
            target.pos[1] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
            target.pos[2] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
        }
    }
}

/// 1. update Function - Physics and Movement:

///   Time Delta: The dt_ns (delta time in nanoseconds) is crucial for frame-rate independent physics. It's converted to seconds (time).
///   Drag: drag = (-time * rate).exp(); calculates an exponential drag factor. This simulates air resistance or friction, slowing the player down over time. The higher rate is, the stronger the drag.
///   Acceleration: The code calculates the player's acceleration based on the WASD keys pressed and the player's current yaw (horizontal rotation). It uses trigonometry (cos and sin) to determine the direction of movement in 2D (x and z axes). norm normalizes the direction vector.
///   Velocity Update: The player's velocity is updated by:
///      Applying drag: player.vel[0] -= vel_x * diff; (and similarly for z).
///      Applying gravity: player.vel[1] -= grav * time;.
///      Applying acceleration: player.vel[0] += diff * acc_x / rate; (and similarly for z).
///   Position Update: The player's position is updated using a combination of the current velocity and the calculated acceleration. The formula used is a discrete approximation of the equations of motion.
///   Boundary Collision: The code now clamps the player's position to the map boundaries (-bound to bound). If a player hits a boundary, their velocity in that direction is set to 0.
///   Jumping: The spacebar (keycode 16) now allows the player to jump. If the player is on the ground (y position is at the boundary), pressing space sets a vertical velocity.
///
/// 2. Mathematical Principles (Simplified):
///
///   Drag: The drag force is proportional to the player's velocity, acting in the opposite direction. The exponential form is common for simulating drag.
///   Gravity: A constant downward acceleration is applied to the player's y-velocity.
///   Equations of Motion (Simplified): The position updates are based on simplified versions of the following:
///       position = initial_position + velocity * time + 0.5 * acceleration * time^2
///       velocity = initial_velocity + acceleration * time
///   The code uses a slightly different form to account for the changing acceleration due to drag.
///
/// 3. Frame Rate Independence: By using dt_ns, the physics calculations are adjusted based on the time elapsed between frames. This makes the game's physics behave more consistently regardless of the frame rate.

// Function to update player positions and velocities based on input and physics
fn update(players: &mut [Player], players_len: usize, dt_ns: u64) {
    let time = dt_ns as f64 * 1e-9; // Convert time difference to seconds
    for player in players.iter_mut().take(players_len) {
        let rate = 6.0; // Rate of drag
        let drag = (-time * rate).exp(); // Calculate drag factor
        let diff = 1.0 - drag; // Calculate difference factor
        let mult = 60.0; // Movement multiplier
        let grav = 25.0; // Gravity acceleration

        // Calculate player's direction based on yaw and WASD input
        let yaw = player.yaw as f64;
        let rad = yaw * PI / 2147483648.0; // Convert yaw to radians
        let cos = rad.cos(); // Cosine of yaw
        let sin = rad.sin(); // Sine of yaw
        let wasd = player.wasd; // Get WASD input

        // Determine direction of movement based on WASD keys
        let dir_x = if wasd & 8 != 0 { 1.0 } else { 0.0 } - if wasd & 2 != 0 { 1.0 } else { 0.0 };
        let dir_z = if wasd & 4 != 0 { 1.0 } else { 0.0 } - if wasd & 1 != 0 { 1.0 } else { 0.0 };
        let norm = dir_x * dir_x + dir_z * dir_z; // Calculate normalization factor

        // Calculate acceleration based on direction and multiplier
        let acc_x = mult
            * if norm == 0.0 {
                0.0
            } else {
                (cos * dir_x + sin * dir_z) / norm.sqrt()
            };
        let acc_z = mult
            * if norm == 0.0 {
                0.0
            } else {
                (-sin * dir_x + cos * dir_z) / norm.sqrt()
            };

        // Update player's velocity with drag and acceleration
        let vel_x = player.vel[0];
        let vel_y = player.vel[1];
        let vel_z = player.vel[2];

        player.vel[0] -= vel_x * diff; // Apply drag to x velocity
        player.vel[1] -= grav * time; // Apply gravity to y velocity
        player.vel[2] -= vel_z * diff; // Apply drag to z velocity

        player.vel[0] += diff * acc_x / rate; // Apply acceleration to x velocity
        player.vel[2] += diff * acc_z / rate; // Apply acceleration to z velocity

        // Update player's position based on velocity and acceleration
        player.pos[0] += (time - diff / rate) * acc_x / rate + diff * vel_x / rate;
        player.pos[1] += -0.5 * grav * time * time + vel_y * time;
        player.pos[2] += (time - diff / rate) * acc_z / rate + diff * vel_z / rate;

        // Keep player within map bounds
        let scale = MAP_BOX_SCALE as f64;
        let bound = scale - player.radius as f64;
        let pos_x = player.pos[0].max(-bound).min(bound);
        let pos_y = player.pos[1].max(player.height as f64 - scale).min(bound);
        let pos_z = player.pos[2].max(-bound).min(bound);

        // Handle collisions with map boundaries
        if player.pos[0] != pos_x {
            player.vel[0] = 0.0; // Stop x movement
        }
        if player.pos[1] != pos_y {
            player.vel[1] = if wasd & 16 != 0 { 8.4375 } else { 0.0 }; // Set y velocity if spacebar is pressed (jumping)
        }
        if player.pos[2] != pos_z {
            player.vel[2] = 0.0; // Stop z movement
        }
        player.pos[0] = pos_x;
        player.pos[1] = pos_y;
        player.pos[2] = pos_z;
    }
}

fn draw_circle(canvas: &mut Canvas<Window>, r: f32, x: f32, y: f32) {
    let mut points = Vec::with_capacity(CIRCLE_DRAW_SIDES_LEN); // Pre-allocate vector for efficiency

    for i in 0..CIRCLE_DRAW_SIDES_LEN {
        let ang = 2.0 * PI * i as f64 / CIRCLE_DRAW_SIDES as f64; // Calculate angle for each point

        // Create and add the point to the vector
        points.push(sdl3::render::FPoint::new(
            x + r * (ang.cos() as f32),
            y + r * (ang.sin() as f32),
        ));
    }
    // Draw the circle by connecting the points with lines
    canvas.draw_lines(points.as_slice()).unwrap();
}

/// fn draw_clipped_segment()
///
/// 1. Clipping: The function implements a simple form of clipping against a plane defined by z = -w. This is a common technique in 3D graphics to prevent drawing objects that are behind the "camera" or outside the viewing frustum.
///   The if az >= -w && bz >= -w check efficiently handles the case where both points of the line segment are behind the clipping plane.  No drawing is needed in this case.
///   The code then checks each point individually (if az > -w and if bz > -w). If a point is behind the plane, the code calculates the intersection point of the line segment with the plane using linear interpolation.  The parameter t determines how far along the line segment the intersection occurs.
///
/// 2. Perspective Projection: After clipping, the code performs perspective projection.  This is what makes objects appear smaller as they are further away.
///   ax = -z * ax / az; (and similarly for ay, bx, by): This is the perspective divide. Dividing by az (and bz) makes the coordinates scale inversely with distance, creating the perspective effect. -z is used because the code assumes the camera is looking along the negative z-axis.
///
/// 3 .Screen Coordinates:  The projected coordinates (ax, ay, bx, by) are then added to x and y respectively. These x and y values likely represent the origin or offset for the current viewport or camera. The y-coordinate is also negated (y - ay) because in most screen coordinate systems, the y-axis points downwards, while in typical Cartesian coordinate systems, it points upwards.
///
/// 4. Drawing: Finally, the canvas.draw_line function is used to draw the clipped and projected line segment.  The coordinates are converted to integers (as i32) before being passed to draw_line, as screen coordinates are typically integers.
fn draw_clipped_segment(
    canvas: &mut Canvas<Window>,
    ax: f32,
    ay: f32,
    az: f32,
    bx: f32,
    by: f32,
    bz: f32,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    // Check if both points are behind the clipping plane
    if az >= -w && bz >= -w {
        return; // If so, don't draw anything
    }

    // Calculate the difference vector between the two points
    let dx = ax - bx;
    let dy = ay - by;

    // Clip the first point (A) if it's behind the clipping plane
    let (mut ax, mut ay, mut az) = if az > -w {
        let t = (-w - bz) / (az - bz); // Calculate intersection parameter
        (bx + dx * t, by + dy * t, -w) // Interpolate to the clipping plane
    } else {
        (ax, ay, az) // Point A is already in front, no clipping needed
    };

    // Clip the second point (B) if it's behind the clipping plane
    let (mut bx, mut by, mut bz) = if bz > -w {
        let t = (-w - az) / (bz - az); // Calculate intersection parameter
        (ax - dx * t, ay - dy * t, -w) // Interpolate to the clipping plane
    } else {
        (bx, by, bz) // Point B is already in front, no clipping needed
    };

    // Perspective projection:  Project the 3D points to 2D
    ax = -z * ax / az;
    ay = -z * ay / az;
    bx = -z * bx / bz;
    by = -z * by / bz;

    // Draw the line segment
    canvas
        .draw_line(
            sdl3::rect::Point::new((x + ax) as i32, (y - ay) as i32), // Convert to screen coordinates
            sdl3::rect::Point::new((x + bx) as i32, (y - by) as i32), // Convert to screen coordinates
        )
        .unwrap(); // Handle potential drawing errors
}

fn draw(canvas: &mut Canvas<Window>, edges: &[[f32; 6]], players: &[Player], players_len: usize) {
    let (w, h) = canvas.output_size().unwrap(); // Get window width and height
    canvas.set_draw_color(Color::RGB(0, 0, 0)); // Set background color to black
    canvas.clear(); // Clear the canvas

    if players_len > 0 {
        // Only draw if there are players
        let wf = w as f32; // Window width as float
        let hf = h as f32; // Window height as float

        // Calculate how to split the screen based on the number of players
        let part_hor = if players_len > 2 { 2 } else { 1 }; // Number of horizontal splits
        let part_ver = if players_len > 1 { 2 } else { 1 }; // Number of vertical splits
        let size_hor = wf / part_hor as f32; // Width of each split screen
        let size_ver = hf / part_ver as f32; // Height of each split screen

        // Iterate through each player
        for i in 0..players_len {
            // Get the current player
            let player = &players[i];

            // Calculate the position of the current player's viewport
            let mod_x = (i % part_hor) as f32; // x-coordinate of the viewport in the grid
            let mod_y = (i / part_hor) as f32; // y-coordinate of the viewport in the grid
            let hor_origin = (mod_x + 0.5) * size_hor; // x-coordinate of the center of the viewport
            let ver_origin = (mod_y + 0.5) * size_ver; // y-coordinate of the center of the viewport
            let cam_origin = (0.5 * (size_hor * size_hor + size_ver * size_ver).sqrt()) as f32; // Distance to the "camera"
            let hor_offset = mod_x * size_hor; // x-offset of the viewport
            let ver_offset = mod_y * size_ver; // y-offset of the viewport

            // Set the clipping rectangle for the current player's viewport
            let rect = Rect::new(
                hor_offset as i32,
                ver_offset as i32,
                size_hor as u32,
                size_ver as u32,
            );
            canvas.set_clip_rect(rect); // Anything drawn outside this rectangle won't be visible

            let x0 = player.pos[0]; // Player's x position
            let y0 = player.pos[1]; // Player's y position
            let z0 = player.pos[2]; // Player's z position

            // Pre-calculate trigonometric values for player's view direction
            let bin_rad = PI / 2147483648.0; // Angle conversion factor
            let yaw_rad = bin_rad * player.yaw as f64; // Player's yaw in radians
            let pitch_rad = bin_rad * player.pitch as f64; // Player's pitch in radians
            let cos_yaw = yaw_rad.cos();
            let sin_yaw = yaw_rad.sin();
            let cos_pitch = pitch_rad.cos();
            let sin_pitch = pitch_rad.sin();

            // Create the view matrix (combining rotation)
            let mat = [
                cos_yaw as f32,
                0.0,
                -sin_yaw as f32,
                sin_yaw as f32 * sin_pitch as f32,
                cos_pitch as f32,
                cos_yaw as f32 * sin_pitch as f32,
                sin_yaw as f32 * cos_pitch as f32,
                -sin_pitch as f32,
                cos_yaw as f32 * cos_pitch as f32,
            ];
            canvas.set_draw_color(Color::RGB(64, 64, 64)); // Set color for the map edges

            // Draw each edge of the map
            for line in edges.iter() {
                // Transform the edge points by the player's view matrix
                let ax = mat[0] * (line[0] as f64 - x0) as f32
                    + mat[1] * (line[1] as f64 - y0) as f32
                    + mat[2] * (line[2] as f64 - z0) as f32;
                let ay = mat[3] * (line[0] as f64 - x0) as f32
                    + mat[4] * (line[1] as f64 - y0) as f32
                    + mat[5] * (line[2] as f64 - z0) as f32;
                let az = mat[6] * (line[0] as f64 - x0) as f32
                    + mat[7] * (line[1] as f64 - y0) as f32
                    + mat[8] * (line[2] as f64 - z0) as f32;
                let bx = mat[0] * (line[3] as f64 - x0) as f32
                    + mat[1] * (line[4] as f64 - y0) as f32
                    + mat[2] * (line[5] as f64 - z0) as f32;
                let by = mat[3] * (line[3] as f64 - x0) as f32
                    + mat[4] * (line[4] as f64 - y0) as f32
                    + mat[5] * (line[5] as f64 - z0) as f32;
                let bz = mat[6] * (line[3] as f64 - x0) as f32
                    + mat[7] * (line[4] as f64 - y0) as f32
                    + mat[8] * (line[5] as f64 - z0) as f32;

                // Draw the clipped line segment
                draw_clipped_segment(
                    canvas, ax, ay, az, bx, by, bz, hor_origin, ver_origin, cam_origin, 1.0,
                );
            }

            // Draw other players
            for j in 0..players_len {
                if i == j {
                    continue; // Don't draw the current player
                }
                let target = &players[j]; // Get the target player
                canvas.set_draw_color(Color::RGB(
                    // Set the target player's color
                    target.color[0],
                    target.color[1],
                    target.color[2],
                ));

                // Draw the target player's top and bottom circles
                for k in 0..2 {
                    let rx = target.pos[0] - player.pos[0]; // Relative x position
                    let ry = target.pos[1] - player.pos[1] // Relative y position
                        + (target.radius as f64 - target.height as f64) * k as f64; // Adjust for top/bottom
                    let rz = target.pos[2] - player.pos[2]; // Relative z position

                    // Transform the relative position by the player's view matrix
                    let dx = mat[0] as f64 * rx + mat[1] as f64 * ry + mat[2] as f64 * rz;
                    let dy = mat[3] as f64 * rx + mat[4] as f64 * ry + mat[5] as f64 * rz;
                    let dz = mat[6] as f64 * rx + mat[7] as f64 * ry + mat[8] as f64 * rz;

                    // Calculate the projected radius
                    let r_eff = target.radius as f64 * cam_origin as f64 / dz;

                    // If the target is behind the player, don't draw it
                    if dz >= 0.0 {
                        continue;
                    }
                    //
                    // Draw the target player's circle
                    draw_circle(
                        canvas,
                        r_eff as f32,
                        (hor_origin - cam_origin * dx as f32 / dz as f32) as f32,
                        (ver_origin + cam_origin * dy as f32 / dz as f32) as f32,
                    );
                }
            }
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas
                .draw_line(
                    sdl3::rect::Point::new(hor_origin as i32, (ver_origin - 10.0) as i32),
                    sdl3::rect::Point::new(hor_origin as i32, (ver_origin + 10.0) as i32),
                )
                .unwrap();
            canvas
                .draw_line(
                    sdl3::rect::Point::new((hor_origin - 10.0) as i32, ver_origin as i32),
                    sdl3::rect::Point::new((hor_origin + 10.0) as i32, ver_origin as i32),
                )
                .unwrap();
        }
    }
    canvas.set_clip_rect(None);
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.present();
}

fn init_players(players: &mut [Player], len: usize) {
    // Initialize player positions. Players are placed in a grid-like pattern.
    for i in 0..len {
        players[i].pos[0] = 8.0 * if i & 1 != 0 { -1.0 } else { 1.0 }; // x-position: +/- 8.0
        players[i].pos[1] = 0.0; // y-position: 0.0
        players[i].pos[2] =
            8.0 * if i & 1 != 0 { -1.0 } else { 1.0 } * if i & 2 != 0 { -1.0 } else { 1.0 }; // z-position: +/- 8.0

        // Initialize player velocities to zero.
        players[i].vel[0] = 0.0;
        players[i].vel[1] = 0.0;
        players[i].vel[2] = 0.0;

        // Initialize player yaw (horizontal rotation).  The initial yaw is set based on player index.
        // The bitwise operations distribute the players around the origin.
        players[i].yaw = 0x20000000
            + if i & 1 != 0 { 0x80000000 } else { 0 } // Adds 0x80000000 if the 0th bit is set (player 1 and 3)
            + if i & 2 != 0 { 0x40000000 } else { 0 }; // Adds 0x40000000 if the 1st bit is set (player 2 and 3)

        // Initialize player pitch (vertical rotation). All players start with the same pitch.
        players[i].pitch = -0x08000000;

        // Set player radius and height.
        players[i].radius = 0.5;
        players[i].height = 1.5;

        // Initialize WASD key states to 0 (not pressed).
        players[i].wasd = 0;

        // Initialize mouse and keyboard IDs to 0 (not assigned).
        players[i].mouse = 0;
        players[i].keyboard = 0;

        // Initialize player color based on player index.
        // This code uses bitwise operations to generate a variety of colors.
        players[i].color[0] = if (1 << (i / 2)) & 2 != 0 { 0 } else { 0xff };
        players[i].color[1] = if (1 << (i / 2)) & 1 != 0 { 0 } else { 0xff };
        players[i].color[2] = if (1 << (i / 2)) & 4 != 0 { 0 } else { 0xff };

        // This part inverts the color components based on the player index for more variation.
        players[i].color[0] = if i & 1 != 0 {
            players[i].color[0]
        } else {
            !players[i].color[0]
        };
        players[i].color[1] = if i & 1 != 0 {
            players[i].color[1]
        } else {
            !players[i].color[1]
        };
        players[i].color[2] = if i & 1 != 0 {
            players[i].color[2]
        } else {
            !players[i].color[2]
        };
    }
}

fn init_edges(scale: i32, edges: &mut [[f32; 6]], _edges_len: usize) {
    // Radius of the map cube (half the side length)
    let r = scale as f32;

    // Define the edges of the initial cube (12 edges).
    // Each number in `map` represents a vertex of the cube.
    // The bits in the number correspond to the x, y, and z coordinates (+r or -r)

    #[rustfmt::skip]
    let map = [
        0, 1, 1, 3, 3, 2, 2, 0, // First 4 edges (bottom face)
        7, 6, 6, 4, 4, 5, 5, 7, // Next 4 edges (top face)
        6, 2, 3, 7, 0, 4, 5, 1, // Last 4 edges (connecting top and bottom)
    ];

    // Initialize the first 12 edges (the cube's edges).
    for i in 0..12 {
        // Iterate over x, y, z coordinates
        for j in 0..3 {
            // The bitwise AND checks if the j-th bit is set in map[i*2] or map[i*2+1].
            // If the bit is set, the coordinate is +r; otherwise, it's -r.
            edges[i][j] = if map[i * 2] & (1 << j) != 0 { r } else { -r };
            edges[i][j + 3] = if map[i * 2 + 1] & (1 << j) != 0 {
                r
            } else {
                -r
            };
        }
    }

    // Initialize the remaining edges (the "walls" extending outwards).
    for i in 0..scale as usize {
        let d = (i * 2) as f32; // Distance of the wall from the center

        // For each wall (we're building two walls at a time)
        for j in 0..2 {
            edges[i + 12][3 * j] = if j != 0 { r } else { -r }; // x coordinate of the wall, alternate signs
            edges[i + 12][3 * j + 1] = -r; // y coordinate, always -r
            edges[i + 12][3 * j + 2] = d - r; // z coordinate, increasing with i

            edges[i + 12 + scale as usize][3 * j] = d - r; // x coordinate of the opposite wall
            edges[i + 12 + scale as usize][3 * j + 1] = -r; // y coordinate of the opposite wall
            edges[i + 12 + scale as usize][3 * j + 2] = if j != 0 { r } else { -r };
            // z coordinate of the opposite wall, alternate signs
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Example splitscreen shooter game", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut players = [Player {
        mouse: 0,
        keyboard: 0,
        pos: [0.0; 3],
        vel: [0.0; 3],
        yaw: 0,
        pitch: 0,
        radius: 0.0,
        height: 0.0,
        color: [0; 3],
        wasd: 0,
    }; MAX_PLAYER_COUNT];
    let mut edges = [[0.0; 6]; MAP_BOX_EDGES_LEN];

    init_players(&mut players, MAX_PLAYER_COUNT);
    init_edges(MAP_BOX_SCALE, &mut edges, MAP_BOX_EDGES_LEN);

    let mut app_state = AppState {
        canvas,
        player_count: 1,
        players,
        edges,
    };

    let mut last_time = Instant::now();
    let mut accumulator = 0u64;
    let mut past_time = Instant::now();

    'running: loop {
        let now = Instant::now();
        let dt_ns = now.duration_since(past_time).as_nanos() as u64;
        past_time = now;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseMotion {
                    which, xrel, yrel, ..
                } => {
                    if let Some(index) =
                        whose_mouse(which, &app_state.players, app_state.player_count)
                    {
                        // Invert the xrel for correct left/right rotation
                        app_state.players[index].yaw = app_state.players[index]
                            .yaw
                            .wrapping_add((-xrel as i32 * 0x00400000) as u32); // Adjust mouse movement quickly/slowly

                        // Invert yrel for correct up/down looking
                        let new_pitch = app_state.players[index].pitch - (yrel as i32 * 0x00400000);

                        // Clamp pitch to prevent over-rotation
                        app_state.players[index].pitch = new_pitch.clamp(-0x42000000, 0x42000000);
                    } else if which != 0 {
                        for i in 0..MAX_PLAYER_COUNT {
                            if app_state.players[i].mouse == 0 {
                                app_state.players[i].mouse = which;
                                app_state.player_count = app_state.player_count.max(i + 1);
                                break;
                            }
                        }
                    }
                }

                Event::MouseButtonDown { which, .. } => {
                    if let Some(index) =
                        whose_mouse(which, &app_state.players, app_state.player_count)
                    {
                        shoot(index, &mut app_state.players, app_state.player_count);
                    }
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    which,
                    ..
                } => {
                    if let Some(index) =
                        whose_keyboard(which, &app_state.players, app_state.player_count)
                    {
                        match keycode {
                            Keycode::W => app_state.players[index].wasd |= 1,
                            Keycode::A => app_state.players[index].wasd |= 2,
                            Keycode::S => app_state.players[index].wasd |= 4,
                            Keycode::D => app_state.players[index].wasd |= 8,
                            Keycode::Space => app_state.players[index].wasd |= 16,
                            _ => {}
                        }
                    } else if which != 0 {
                        for i in 0..MAX_PLAYER_COUNT {
                            if app_state.players[i].keyboard == 0 {
                                app_state.players[i].keyboard = which;
                                app_state.player_count = app_state.player_count.max(i + 1);
                                break;
                            }
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    which,
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                    if let Some(index) =
                        whose_keyboard(which, &app_state.players, app_state.player_count)
                    {
                        match keycode {
                            Keycode::W => app_state.players[index].wasd &= 30,
                            Keycode::A => app_state.players[index].wasd &= 29,
                            Keycode::S => app_state.players[index].wasd &= 27,
                            Keycode::D => app_state.players[index].wasd &= 23,
                            Keycode::Space => app_state.players[index].wasd &= 15,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        update(&mut app_state.players, app_state.player_count, dt_ns);
        draw(
            &mut app_state.canvas,
            &app_state.edges,
            &app_state.players,
            app_state.player_count,
        );
        draw(
            &mut app_state.canvas,
            &app_state.edges,
            &app_state.players,
            app_state.player_count,
        );

        accumulator += 1;
        if now.duration_since(last_time) > Duration::from_secs(1) {
            last_time = now;
            accumulator = 0;
        }

        let elapsed = Instant::now().duration_since(now).as_nanos() as u64;
        if elapsed < 999999 {
            std::thread::sleep(Duration::from_nanos(999999 - elapsed));
        }
    }

    Ok(())
}
