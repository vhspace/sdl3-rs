extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-sdl3 demo: GPU (clear)", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    // Need to create the GPU device first, we'll let SDL3 use the most optimal driver
    // by default, and we specify that our shaders will be SPIR-V ones (even through we
    // aren't using any shaders)
    // We'll also turn on debug mode to true, so we get debug stuff
    let gpu = sdl3::gpu::Device::new(sdl3::gpu::ShaderFormat::SPIRV, true)?;
    gpu.claim_window(&window)?;

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
            let color_targets = [
                sdl3::gpu::ColorTargetInfo::default()
                    .with_texture(&swapchain) // Use swapchain texture
                    .with_load_op(sdl3::gpu::LoadOp::CLEAR) // Clear when load
                    .with_store_op(sdl3::gpu::StoreOp::STORE) // Store back
                    .with_clear_color(sdl3::pixels::Color::RGB(5, 3, 255)), //blue with small RG bias
            ];
            // Here we do all (none) of our drawing (clearing the screen)
            command_buffer.render_pass(
                &color_targets,
                None,
                |_cmd, _pass| {
                    // Do absolutely nothing -- this clears the screen because of the defined operations above
                    // which are ALWAYS done even through we just created and ended a render pass
                }
            )?;
            command_buffer.submit()?;
        } else {
            // Swapchain unavailable, cancel work
            command_buffer.cancel();
        }
    }
    Ok(())
}
