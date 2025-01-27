use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::f64::consts::PI;
use std::time::{Duration, Instant};

const MAP_BOX_SCALE: i32 = 16;
const MAP_BOX_EDGES_LEN: usize = 12 + (MAP_BOX_SCALE * 2) as usize;
const MAX_PLAYER_COUNT: usize = 4;
const CIRCLE_DRAW_SIDES: usize = 32;
const CIRCLE_DRAW_SIDES_LEN: usize = CIRCLE_DRAW_SIDES + 1;

#[derive(Clone, Copy)]
struct Player {
    mouse: u32,
    keyboard: u32,
    pos: [f64; 3],
    vel: [f64; 3],
    yaw: u32,
    pitch: i32,
    radius: f32,
    height: f32,
    color: [u8; 3],
    wasd: u8,
}

struct AppState {
    canvas: Canvas<Window>,
    player_count: usize,
    players: [Player; MAX_PLAYER_COUNT],
    edges: [[f32; 6]; MAP_BOX_EDGES_LEN],
}

fn whose_mouse(mouse: u32, players: &[Player], players_len: usize) -> Option<usize> {
    players.iter().position(|p| p.mouse == mouse)
}

fn whose_keyboard(keyboard: u32, players: &[Player], players_len: usize) -> Option<usize> {
    players.iter().position(|p| p.keyboard == keyboard)
}

fn shoot(shooter: usize, players: &mut [Player], players_len: usize) {
    let x0 = players[shooter].pos[0];
    let y0 = players[shooter].pos[1];
    let z0 = players[shooter].pos[2];
    let bin_rad = PI / 2147483648.0;
    let yaw_rad = (bin_rad as f64) * (players[shooter].yaw) as f64;
    let pitch_rad = (bin_rad as f64) * players[shooter].pitch as f64;
    let cos_yaw = yaw_rad.cos();
    let sin_yaw = yaw_rad.sin();
    let cos_pitch = pitch_rad.cos();
    let sin_pitch = pitch_rad.sin();
    let vx = -sin_yaw * cos_pitch;
    let vy = sin_pitch;
    let vz = -cos_yaw * cos_pitch;

    for i in 0..players_len {
        if i == shooter {
            continue;
        }
        let target = &mut players[i];
        let mut hit = 0;
        for j in 0..2 {
            let r = target.radius as f64;
            let h = target.height as f64;
            let dx = target.pos[0] - x0;
            let dy = target.pos[1] - y0 + if j == 0 { 0.0 } else { r - h };
            let dz = target.pos[2] - z0;
            let vd = vx * dx as f64 + vy * dy as f64 + vz * dz as f64;
            let dd = dx * dx + dy * dy + dz * dz;
            let vv = vx * vx + vy * vy + vz * vz;
            let rr = r * r;
            if vd < 0.0 {
                continue;
            }
            if vd * vd >= vv * (dd as f64 - rr as f64) {
                hit += 1;
            }
        }
        if hit > 0 {
            target.pos[0] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
            target.pos[1] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
            target.pos[2] = (MAP_BOX_SCALE as f64 * (rand::random::<u8>() as f64 - 128.0)) / 256.0;
        }
    }
}

fn update(players: &mut [Player], players_len: usize, dt_ns: u64) {
    let time = dt_ns as f64 * 1e-9;
    for player in players.iter_mut().take(players_len) {
        let rate = 6.0;
        let drag = (-time * rate).exp();
        let diff = 1.0 - drag;
        let mult = 60.0;
        let grav = 25.0;
        let yaw = player.yaw as f64;
        let rad = yaw * PI / 2147483648.0;
        let cos = rad.cos();
        let sin = rad.sin();
        let wasd = player.wasd;
        let dir_x = if wasd & 8 != 0 { 1.0 } else { 0.0 } - if wasd & 2 != 0 { 1.0 } else { 0.0 };
        let dir_z = if wasd & 4 != 0 { 1.0 } else { 0.0 } - if wasd & 1 != 0 { 1.0 } else { 0.0 };
        let norm = dir_x * dir_x + dir_z * dir_z;
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
        let vel_x = player.vel[0];
        let vel_y = player.vel[1];
        let vel_z = player.vel[2];
        player.vel[0] -= vel_x * diff;
        player.vel[1] -= grav * time;
        player.vel[2] -= vel_z * diff;
        player.vel[0] += diff * acc_x / rate;
        player.vel[2] += diff * acc_z / rate;
        player.pos[0] += (time - diff / rate) * acc_x / rate + diff * vel_x / rate;
        player.pos[1] += -0.5 * grav * time * time + vel_y * time;
        player.pos[2] += (time - diff / rate) * acc_z / rate + diff * vel_z / rate;
        let scale = MAP_BOX_SCALE as f64;
        let bound = scale - player.radius as f64;
        let pos_x = player.pos[0].max(-bound).min(bound);
        let pos_y = player.pos[1].max(player.height as f64 - scale).min(bound);
        let pos_z = player.pos[2].max(-bound).min(bound);
        if player.pos[0] != pos_x {
            player.vel[0] = 0.0;
        }
        if player.pos[1] != pos_y {
            player.vel[1] = if wasd & 16 != 0 { 8.4375 } else { 0.0 };
        }
        if player.pos[2] != pos_z {
            player.vel[2] = 0.0;
        }
        player.pos[0] = pos_x;
        player.pos[1] = pos_y;
        player.pos[2] = pos_z;
    }
}

fn draw_circle(canvas: &mut Canvas<Window>, r: f32, x: f32, y: f32) {
    let mut points = Vec::with_capacity(CIRCLE_DRAW_SIDES_LEN);
    for i in 0..CIRCLE_DRAW_SIDES_LEN {
        let ang = 2.0 * PI * i as f64 / CIRCLE_DRAW_SIDES as f64;
        points.push(sdl3::render::FPoint::new(
            x + r * (ang.cos() as f32),
            y + r * (ang.sin() as f32),
        ));
    }
    canvas.draw_lines(points.as_slice()).unwrap();
}

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
    if az >= -w && bz >= -w {
        return;
    }
    let dx = ax - bx;
    let dy = ay - by;
    let (mut ax, mut ay, mut az) = if az > -w {
        let t = (-w - bz) / (az - bz);
        (bx + dx * t, by + dy * t, -w)
    } else {
        (ax, ay, az)
    };
    let (mut bx, mut by, mut bz) = if bz > -w {
        let t = (-w - az) / (bz - az);
        (ax - dx * t, ay - dy * t, -w)
    } else {
        (bx, by, bz)
    };
    ax = -z * ax / az;
    ay = -z * ay / az;
    bx = -z * bx / bz;
    by = -z * by / bz;
    canvas
        .draw_line(
            sdl3::rect::Point::new((x + ax) as i32, (y - ay) as i32),
            sdl3::rect::Point::new((x + bx) as i32, (y - by) as i32),
        )
        .unwrap();
}

fn draw(canvas: &mut Canvas<Window>, edges: &[[f32; 6]], players: &[Player], players_len: usize) {
    let (w, h) = canvas.output_size().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    if players_len > 0 {
        let wf = w as f32;
        let hf = h as f32;
        let part_hor = if players_len > 2 { 2 } else { 1 };
        let part_ver = if players_len > 1 { 2 } else { 1 };
        let size_hor = wf / part_hor as f32;
        let size_ver = hf / part_ver as f32;
        for i in 0..players_len {
            let player = &players[i];
            let mod_x = (i % part_hor) as f32;
            let mod_y = (i / part_hor) as f32;
            let hor_origin = (mod_x + 0.5) * size_hor;
            let ver_origin = (mod_y + 0.5) * size_ver;
            let cam_origin = (0.5 * (size_hor * size_hor + size_ver * size_ver).sqrt()) as f32;
            let hor_offset = mod_x * size_hor;
            let ver_offset = mod_y * size_ver;
            let rect = Rect::new(
                hor_offset as i32,
                ver_offset as i32,
                size_hor as u32,
                size_ver as u32,
            );
            canvas.set_clip_rect(rect);
            let x0 = player.pos[0];
            let y0 = player.pos[1];
            let z0 = player.pos[2];
            let bin_rad = PI / 2147483648.0;
            let yaw_rad = bin_rad * player.yaw as f64;
            let pitch_rad = bin_rad * player.pitch as f64;
            let cos_yaw = yaw_rad.cos();
            let sin_yaw = yaw_rad.sin();
            let cos_pitch = pitch_rad.cos();
            let sin_pitch = pitch_rad.sin();
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
            canvas.set_draw_color(Color::RGB(64, 64, 64));
            for line in edges.iter() {
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
                draw_clipped_segment(
                    canvas, ax, ay, az, bx, by, bz, hor_origin, ver_origin, cam_origin, 1.0,
                );
            }
            for j in 0..players_len {
                if i == j {
                    continue;
                }
                let target = &players[j];
                canvas.set_draw_color(Color::RGB(
                    target.color[0],
                    target.color[1],
                    target.color[2],
                ));
                for k in 0..2 {
                    let rx = target.pos[0] - player.pos[0];
                    let ry = target.pos[1] - player.pos[1]
                        + (target.radius as f64 - target.height as f64) * k as f64;
                    let rz = target.pos[2] - player.pos[2];
                    let dx = mat[0] as f64 * rx + mat[1] as f64 * ry + mat[2] as f64 * rz;
                    let dy = mat[3] as f64 * rx + mat[4] as f64 * ry + mat[5] as f64 * rz;
                    let dz = mat[6] as f64 * rx + mat[7] as f64 * ry + mat[8] as f64 * rz;
                    let r_eff = target.radius as f64 * cam_origin as f64 / dz;
                    if dz >= 0.0 {
                        continue;
                    }
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
    for i in 0..len {
        players[i].pos[0] = 8.0 * if i & 1 != 0 { -1.0 } else { 1.0 };
        players[i].pos[1] = 0.0;
        players[i].pos[2] =
            8.0 * if i & 1 != 0 { -1.0 } else { 1.0 } * if i & 2 != 0 { -1.0 } else { 1.0 };
        players[i].vel[0] = 0.0;
        players[i].vel[1] = 0.0;
        players[i].vel[2] = 0.0;
        players[i].yaw = 0x20000000
            + if i & 1 != 0 { 0x80000000 } else { 0 }
            + if i & 2 != 0 { 0x40000000 } else { 0 };
        players[i].pitch = -0x08000000;
        players[i].radius = 0.5;
        players[i].height = 1.5;
        players[i].wasd = 0;
        players[i].mouse = 0;
        players[i].keyboard = 0;
        players[i].color[0] = if (1 << (i / 2)) & 2 != 0 { 0 } else { 0xff };
        players[i].color[1] = if (1 << (i / 2)) & 1 != 0 { 0 } else { 0xff };
        players[i].color[2] = if (1 << (i / 2)) & 4 != 0 { 0 } else { 0xff };
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
    let r = scale as f32;
    let map = [
        0, 1, 1, 3, 3, 2, 2, 0, 7, 6, 6, 4, 4, 5, 5, 7, 6, 2, 3, 7, 0, 4, 5, 1,
    ];
    for i in 0..12 {
        for j in 0..3 {
            edges[i][j] = if map[i * 2] & (1 << j) != 0 { r } else { -r };
            edges[i][j + 3] = if map[i * 2 + 1] & (1 << j) != 0 {
                r
            } else {
                -r
            };
        }
    }
    for i in 0..scale as usize {
        let d = (i * 2) as f32;
        for j in 0..2 {
            edges[i + 12][3 * j] = if j != 0 { r } else { -r };
            edges[i + 12][3 * j + 1] = -r;
            edges[i + 12][3 * j + 2] = d - r;
            edges[i + 12 + scale as usize][3 * j] = d - r;
            edges[i + 12 + scale as usize][3 * j + 1] = -r;
            edges[i + 12 + scale as usize][3 * j + 2] = if j != 0 { r } else { -r };
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
                            .wrapping_add((-xrel as i32 * 0x00400000) as u32); // Move the mouse quickly

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
