extern crate sdl3;

use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::{event::Event, gpu::GraphicsPipelineTargetInfo};
use sdl3_sys::render;
use std::time::Duration;

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
    let gpu = sdl3::gpu::Device::new(
        sdl3::gpu::ShaderFormat::SpirV
            | sdl3::gpu::ShaderFormat::Dxil
            | sdl3::gpu::ShaderFormat::Dxbc
            | sdl3::gpu::ShaderFormat::MetalLib,
        true,
    )
    .with_window(&window)?;

    let fs_source = include_bytes!("shaders/triangle.frag.spv");
    let vs_source = include_bytes!("shaders/triangle.vert.spv");

    // Our shaders, require to be precompiled by a SPIR-V compiler beforehand
    let vs_shader = gpu
        .create_shader()
        .with_code(
            sdl3::gpu::ShaderFormat::SpirV,
            vs_source,
            sdl3::gpu::ShaderStage::Vertex,
        )
        .with_entrypoint("main")
        .build()?;

    let fs_shader = gpu
        .create_shader()
        .with_code(
            sdl3::gpu::ShaderFormat::SpirV,
            fs_source,
            sdl3::gpu::ShaderStage::Fragment,
        )
        .with_entrypoint("main")
        .build()?;

    let swapchain_format = gpu.get_swapchain_texture_format(&window);

    // Create a pipeline, we specify that we want our target format in the one of the swapchain
    // since we are rendering directly unto the swapchain, however, we could specify one that
    // is different from the swapchain (i.e offscreen rendering)
    let pipeline = gpu
        .create_graphics_pipeline()
        .with_fragment_shader(&fs_shader)
        .with_vertex_shader(&vs_shader)
        .with_primitive_type(sdl3::gpu::PrimitiveType::TriangleList)
        .with_fill_mode(sdl3::gpu::FillMode::Fill)
        .with_target_info(
            GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&[
                sdl3::gpu::ColorTargetDescriptionBuilder::new()
                    .with_format(swapchain_format)
                    .build(),
            ]),
        )
        .build()?;

    vs_shader.release(&gpu);
    fs_shader.release(&gpu);

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
        let mut command_buffer = gpu.acquire_command_buffer();
        if let Ok(swapchain) = gpu.wait_and_acquire_swapchain_texture(&window, &mut command_buffer)
        {
            // Again, like in gpu-clear.rs, we'd want to define basic operations for our triangle
            let color_targets = [
                sdl3::gpu::ColorTargetInfo::default()
                    .with_texture(swapchain)
                    .with_load_op(sdl3::gpu::LoadOp::Clear)
                    .with_store_op(sdl3::gpu::StoreOp::Store)
                    .with_clear_color(sdl3::pixels::Color::RGB(5, 3, 255)), //blue with small RG bias
            ];
            let render_pass = gpu.begin_render_pass(&command_buffer, &color_targets, None)?;
            render_pass.bind_graphics_pipeline(&pipeline);
            // Screen is cleared here due to the color target info
            // Now we'll draw the triangle primitives
            render_pass.draw_primitives(3, 1, 0, 0);
            gpu.end_render_pass(render_pass);
            command_buffer.submit()?;
        } else {
            // Swapchain unavailable, cancel work
            command_buffer.cancel();
        }
    }
    Ok(())
}
