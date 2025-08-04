use rand::Rng;
use sdl3::gpu::*;
use sdl3::pixels::Color;
use sdl3::{event::Event, keyboard::Keycode};
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Copy)]
struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
}

const PARTICLE_COUNT: u32 = 1000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;

    let window = video
        .window("SDL3 Compute Particles", 800, 600)
        .position_centered()
        .build()?;

    let device: Device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

    // === Create particle buffer ===
    let particles: Vec<Particle> = (0..PARTICLE_COUNT)
        .map(|_| Particle {
            pos: [rand01() * 2.0 - 1.0, rand01() * 2.0 - 1.0],
            vel: [0.0, -0.01],
            color: [rand01(), rand01(), rand01(), 1.0],
        })
        .collect();

    let buffer_size = (PARTICLE_COUNT as usize * size_of::<Particle>()) as u32;
    let particle_buffer = device
        .create_buffer()
        .with_size(buffer_size)
        .with_usage(BufferUsageFlags::COMPUTE_STORAGE_WRITE)
        .build()?;

    // === Upload particles to GPU ===
    let upload = device
        .create_transfer_buffer()
        .with_size(buffer_size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    {
        let mut map = upload.map::<Particle>(&device, true);
        map.mem_mut().copy_from_slice(&particles);
        map.unmap();

        let copy_cmd = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_cmd)?;
        copy_pass.upload_to_gpu_buffer(
            TransferBufferLocation::new()
                .with_offset(0)
                .with_transfer_buffer(&upload),
            BufferRegion::new()
                .with_offset(0)
                .with_size(buffer_size)
                .with_buffer(&particle_buffer),
            true,
        );
        device.end_copy_pass(copy_pass);
        copy_cmd.submit()?;
    }

    // === Load shaders ===
    let pipeline = device
        .create_graphics_pipeline()
        .with_primitive_type(PrimitiveType::PointList)
        .with_vertex_shader(
            &device
                .create_shader()
                .with_code(
                    ShaderFormat::SPIRV,
                    include_bytes!("shaders/particles.vert.spv"),
                    ShaderStage::Vertex,
                )
                .with_storage_buffers(1)
                .with_entrypoint(c"main")
                .build()?,
        )
        .with_fragment_shader(
            &device
                .create_shader()
                .with_code(
                    ShaderFormat::SPIRV,
                    include_bytes!("shaders/particles.frag.spv"),
                    ShaderStage::Fragment,
                )
                .with_entrypoint(c"main")
                .build()?,
        )
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(&[ColorTargetDescription::new()
                    .with_format(device.get_swapchain_texture_format(&window))]),
        )
        .build()?;

    let compute_pipeline = device
        .create_compute_pipeline()
        .with_code(
            ShaderFormat::SPIRV,
            include_bytes!("shaders/particles.comp.spv"),
        )
        .with_entrypoint(c"main")
        .with_readwrite_storage_buffers(1)
        .with_thread_count(64, 1, 1)
        .build()?;

    // === Event loop ===
    let mut events = sdl.event_pump()?;
    'running: loop {
        for e in events.poll_iter() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // === Run compute shader ===
        let compute_cmd = device.acquire_command_buffer()?;
        let binding = StorageBufferReadWriteBinding::new()
            .with_buffer(&particle_buffer)
            .with_cycle(false);
        let compute_pass = device.begin_compute_pass(&compute_cmd, &[], &[binding])?;
        compute_pass.bind_compute_pipeline(&compute_pipeline);
        compute_pass.dispatch(PARTICLE_COUNT.div_ceil(64), 1, 1);
        device.end_compute_pass(compute_pass);
        compute_cmd.submit()?;

        // === Draw ===
        let mut draw_cmd = device.acquire_command_buffer()?;
        if let Ok(swapchain) = draw_cmd.wait_and_acquire_swapchain_texture(&window) {
            let color_target = ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(10, 10, 30));
            let pass = device.begin_render_pass(&draw_cmd, &[color_target], None)?;

            pass.bind_graphics_pipeline(&pipeline);
            pass.bind_vertex_storage_buffers(0, &[particle_buffer.clone()]);
            pass.draw_primitives(PARTICLE_COUNT as usize, 1, 0, 0);

            device.end_render_pass(pass);
            draw_cmd.submit()?;
        } else {
            draw_cmd.cancel();
        }
    }

    Ok(())
}

fn rand01() -> f32 {
    rand::thread_rng().gen::<f32>()
}
