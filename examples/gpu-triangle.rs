extern crate sdl3;

use sdl3::{
    event::Event,
    gpu::{
        ColorTargetDescription, ColorTargetInfo, Device, FillMode, GraphicsPipelineTargetInfo,
        LoadOp, PrimitiveType, ShaderFormat, ShaderStage, StoreOp,
    },
    keyboard::Keycode,
    pixels::Color,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-sdl3 demo: GPU (triangle)", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    // Need to create the GPU device first, we'll let SDL3 use the most optimal driver
    // by default, and we specify that our shaders will be SPIR-V ones (even through we
    // aren't using any shaders)
    // We'll also turn on debug mode to true, so we get debug stuff
    let gpu = Device::new(
        ShaderFormat::SPIRV | ShaderFormat::DXIL | ShaderFormat::DXBC | ShaderFormat::METALLIB,
        true,
    )?;
    gpu.claim_window(&window)?;

    let fs_source = include_bytes!("shaders/triangle.frag.spv");
    let vs_source = include_bytes!("shaders/triangle.vert.spv");

    // Our shaders, require to be precompiled by a SPIR-V compiler beforehand
    let vs_shader = gpu
        .create_shader()
        .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
        .with_entrypoint(c"main")
        .build()?;

    let fs_shader = gpu
        .create_shader()
        .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
        .with_entrypoint(c"main")
        .build()?;

    let swapchain_format = gpu.get_swapchain_texture_format(&window);

    // Create a pipeline, we specify that we want our target format in the one of the swapchain
    // since we are rendering directly unto the swapchain, however, we could specify one that
    // is different from the swapchain (i.e offscreen rendering)
    let pipeline = gpu
        .create_graphics_pipeline()
        .with_fragment_shader(&fs_shader)
        .with_vertex_shader(&vs_shader)
        .with_primitive_type(PrimitiveType::TriangleList)
        .with_fill_mode(FillMode::Fill)
        .with_target_info(
            GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&[
                ColorTargetDescription::new().with_format(swapchain_format),
            ]),
        )
        .build()?;

    // The pipeline now holds copies of our shaders, so we can release them
    drop(vs_shader);
    drop(fs_shader);

    let mut event_pump = sdl_context.event_pump()?;
    println!(
        "This example demonstrates that the GPU is working, if it isn't - you should be worried."
    );
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

        // The swapchain texture is basically the framebuffer corresponding to the drawable
        // area of a given window - note how we "wait" for it to come up
        //
        // This is because a swapchain needs to be "allocated", and it can quickly run out
        // if we don't properly time the rendering process.
        let mut command_buffer = gpu.acquire_command_buffer()?;
        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&window) {
            // Again, like in gpu-clear.rs, we'd want to define basic operations for our triangle
            let color_targets = [
                ColorTargetInfo::default()
                    .with_texture(&swapchain)
                    .with_load_op(LoadOp::CLEAR)
                    .with_store_op(StoreOp::STORE)
                    .with_clear_color(Color::RGB(5, 3, 255)), //blue with small RG bias
            ];
            
            command_buffer.render_pass(&color_targets, None, |_cmd, render_pass| {
                render_pass.bind_graphics_pipeline(&pipeline);
                // Screen is cleared here due to the color target info
                // Now we'll draw the triangle primitives
                render_pass.draw_primitives(3, 1, 0, 0);
            })?;

            command_buffer.submit()?;
        } else {
            // Swapchain unavailable, cancel work
            command_buffer.cancel();
        }
    }

    Ok(())
}
