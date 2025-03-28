use sdl3::{
    event::Event,
    gpu::{
        Buffer, BufferBinding, BufferRegion, BufferUsageFlags, ColorTargetDescription,
        ColorTargetInfo, CompareOp, CopyPass, CullMode, DepthStencilState, DepthStencilTargetInfo,
        Device, FillMode, Filter, GraphicsPipelineTargetInfo, IndexElementSize, LoadOp,
        PrimitiveType, RasterizerState, SampleCount, SamplerAddressMode, SamplerCreateInfo,
        SamplerMipmapMode, ShaderFormat, ShaderStage, StoreOp, Texture, TextureCreateInfo,
        TextureFormat, TextureRegion, TextureSamplerBinding, TextureTransferInfo, TextureType,
        TextureUsage, TransferBuffer, TransferBufferLocation, TransferBufferUsage, VertexAttribute,
        VertexBufferDescription, VertexElementFormat, VertexInputRate, VertexInputState,
    },
    keyboard::Keycode,
    pixels::Color,
    surface::Surface,
    Error,
};
use std::path::Path;

extern crate sdl3;

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

// Below are the vertices and indices that make up the 3D mesh of the cube.
const CUBE_VERTICES: &'static [Vertex] = &[
    Vertex {
        x: -0.5,
        y: -0.5,
        z: -0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: 0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
];

const CUBE_INDICES: &'static [u16] = &[
    0, 1, 2, 0, 2, 3, // front
    4, 5, 6, 4, 6, 7, // back
    8, 9, 10, 10, 11, 8, // left
    12, 13, 14, 14, 15, 12, // right
    16, 17, 18, 18, 19, 17, // top
        // not bothering with bottom since it's not visible
];

const WINDOW_SIZE: u32 = 800;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-sdl3 demo: GPU (texture)", WINDOW_SIZE, WINDOW_SIZE)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let gpu = sdl3::gpu::Device::new(
        ShaderFormat::SPIRV | ShaderFormat::DXIL | ShaderFormat::DXBC | ShaderFormat::METALLIB,
        true,
    )?
    .with_window(&window)?;

    // Our shaders, require to be precompiled by a SPIR-V compiler beforehand
    let vert_shader = gpu
        .create_shader()
        .with_code(
            ShaderFormat::SPIRV,
            include_bytes!("shaders/cube-texture.vert.spv"),
            ShaderStage::Vertex,
        )
        .with_uniform_buffers(1)
        .with_entrypoint(c"main")
        .build()?;
    let frag_shader = gpu
        .create_shader()
        .with_code(
            ShaderFormat::SPIRV,
            include_bytes!("shaders/cube-texture.frag.spv"),
            ShaderStage::Fragment,
        )
        .with_samplers(1)
        .with_entrypoint(c"main")
        .build()?;

    // Create a pipeline, we specify that we want our target format in the swapchain
    // since we are rendering directly to the screen. However, we could specify a texture
    // buffer instead (e.g., for offscreen rendering).
    let swapchain_format = gpu.get_swapchain_texture_format(&window);
    let pipeline = gpu
        .create_graphics_pipeline()
        .with_primitive_type(PrimitiveType::TriangleList)
        .with_fragment_shader(&frag_shader)
        .with_vertex_shader(&vert_shader)
        .with_vertex_input_state(
            VertexInputState::new()
                .with_vertex_buffer_descriptions(&[VertexBufferDescription::new()
                    .with_slot(0)
                    .with_pitch(size_of::<Vertex>() as u32)
                    .with_input_rate(VertexInputRate::Vertex)
                    .with_instance_step_rate(0)])
                .with_vertex_attributes(&[
                    VertexAttribute::new()
                        .with_format(VertexElementFormat::Float3)
                        .with_location(0)
                        .with_buffer_slot(0)
                        .with_offset(0),
                    VertexAttribute::new()
                        .with_format(VertexElementFormat::Float2)
                        .with_location(1)
                        .with_buffer_slot(0)
                        .with_offset((3 * size_of::<f32>()) as u32),
                ]),
        )
        .with_rasterizer_state(
            RasterizerState::new()
                .with_fill_mode(FillMode::Fill)
                // Turn off culling so that I don't have to get my cube vertex order perfect
                .with_cull_mode(CullMode::None),
        )
        .with_depth_stencil_state(
            // Enable depth testing
            DepthStencilState::new()
                .with_enable_depth_test(true)
                .with_enable_depth_write(true)
                .with_compare_op(CompareOp::Less),
        )
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(&[
                    ColorTargetDescription::new().with_format(swapchain_format)
                ])
                .with_has_depth_stencil_target(true)
                .with_depth_stencil_format(TextureFormat::D16Unorm),
        )
        .build()?;

    // The pipeline now holds copies of our shaders, so we can release them
    drop(vert_shader);
    drop(frag_shader);

    // Next, we create a transfer buffer that is large enough to hold either
    // our vertices or indices since we will be transferring both with it.
    let vertices_len_bytes = CUBE_VERTICES.len() * size_of::<Vertex>();
    let indices_len_bytes = CUBE_INDICES.len() * size_of::<u16>();
    let transfer_buffer = gpu
        .create_transfer_buffer()
        .with_size(vertices_len_bytes.max(indices_len_bytes) as u32)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    // We need to start a copy pass in order to transfer data to the GPU
    let copy_commands = gpu.acquire_command_buffer()?;
    let copy_pass = gpu.begin_copy_pass(&copy_commands)?;

    // Create GPU buffers to hold our vertices and indices and transfer data to them
    let vertex_buffer = create_buffer_with_data(
        &gpu,
        &transfer_buffer,
        &copy_pass,
        BufferUsageFlags::VERTEX,
        &CUBE_VERTICES,
    )?;
    let index_buffer = create_buffer_with_data(
        &gpu,
        &transfer_buffer,
        &copy_pass,
        BufferUsageFlags::INDEX,
        &CUBE_INDICES,
    )?;

    // We're done with the transfer buffer now, so release it.
    drop(transfer_buffer);

    // Load up a texture to put on the cube
    let cube_texture = create_texture_from_image(&gpu, "./assets/texture.bmp", &copy_pass)?;

    // And configure a sampler for pulling pixels from that texture in the frag shader
    let cube_texture_sampler = gpu.create_sampler(
        SamplerCreateInfo::new()
            .with_min_filter(Filter::Nearest)
            .with_mag_filter(Filter::Nearest)
            .with_mipmap_mode(SamplerMipmapMode::Nearest)
            .with_address_mode_u(SamplerAddressMode::Repeat)
            .with_address_mode_v(SamplerAddressMode::Repeat)
            .with_address_mode_w(SamplerAddressMode::Repeat),
    )?;

    // Now complete and submit the copy pass commands to actually do the transfer work
    gpu.end_copy_pass(copy_pass);
    copy_commands.submit()?;

    // We'll need to allocate a texture buffer for our depth buffer for depth testing to work
    let mut depth_texture = gpu.create_texture(
        TextureCreateInfo::new()
            .with_type(TextureType::_2D)
            .with_width(WINDOW_SIZE)
            .with_height(WINDOW_SIZE)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_sample_count(SampleCount::NoMultiSampling)
            .with_format(TextureFormat::D16Unorm)
            .with_usage(TextureUsage::SAMPLER | TextureUsage::DEPTH_STENCIL_TARGET)
    )?;

    let mut rotation = 45.0f32;
    let mut event_pump = sdl_context.event_pump()?;
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
            // Again, like in gpu-clear.rs, we'd want to define basic operations for our cube
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];
            // This time, however, we want depth testing, so we need to also target a depth texture buffer
            let depth_target = DepthStencilTargetInfo::new()
                .with_texture(&mut depth_texture)
                .with_cycle(true)
                .with_clear_depth(1.0)
                .with_clear_stencil(0)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_stencil_load_op(LoadOp::CLEAR)
                .with_stencil_store_op(StoreOp::STORE);
            let render_pass =
                gpu.begin_render_pass(&command_buffer, &color_targets, Some(&depth_target))?;

            // Screen is cleared below due to the color target info
            render_pass.bind_graphics_pipeline(&pipeline);

            // Now we'll bind our buffers/sampler and draw the cube
            render_pass.bind_vertex_buffers(
                0,
                &[BufferBinding::new()
                    .with_buffer(&vertex_buffer)
                    .with_offset(0)],
            );
            render_pass.bind_index_buffer(
                &BufferBinding::new()
                    .with_buffer(&index_buffer)
                    .with_offset(0),
                IndexElementSize::_16BIT,
            );
            render_pass.bind_fragment_samplers(
                0,
                &[TextureSamplerBinding::new()
                    .with_texture(&cube_texture)
                    .with_sampler(&cube_texture_sampler)],
            );

            // Set the rotation uniform for our cube vert shader
            command_buffer.push_vertex_uniform_data(0, &rotation);
            rotation += 0.1f32;

            // Finally, draw the cube
            render_pass.draw_indexed_primitives(CUBE_INDICES.len() as u32, 1, 0, 0, 0);

            gpu.end_render_pass(render_pass);
            command_buffer.submit()?;
        } else {
            // Swapchain unavailable, cancel work
            command_buffer.cancel();
        }
    }

    Ok(())
}

fn create_texture_from_image(
    gpu: &Device,
    image_path: impl AsRef<Path>,
    copy_pass: &CopyPass,
) -> Result<Texture<'static>, Error> {
    let image = Surface::load_bmp(image_path.as_ref())?;
    let image_size = image.size();
    let size_bytes =
        image.pixel_format().byte_size_per_pixel() as u32 * image_size.0 * image_size.1;

    let texture = gpu.create_texture(
        TextureCreateInfo::new()
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_type(TextureType::_2D)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_usage(TextureUsage::SAMPLER),
    )?;

    let transfer_buffer = gpu
        .create_transfer_buffer()
        .with_size(size_bytes)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let mut buffer_mem = transfer_buffer.map::<u8>(gpu, false);
    image.with_lock(|image_bytes| {
        buffer_mem.mem_mut().copy_from_slice(image_bytes);
    });
    buffer_mem.unmap();

    copy_pass.upload_to_gpu_texture(
        TextureTransferInfo::new()
            .with_transfer_buffer(&transfer_buffer)
            .with_offset(0),
        TextureRegion::new()
            .with_texture(&texture)
            .with_layer(0)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_depth(1),
        false,
    );

    Ok(texture)
}

/// Creates a GPU buffer and uploads data to it using the given `copy_pass` and `transfer_buffer`.
fn create_buffer_with_data<T: Copy>(
    gpu: &Device,
    transfer_buffer: &TransferBuffer,
    copy_pass: &CopyPass,
    usage: BufferUsageFlags,
    data: &[T],
) -> Result<Buffer, Error> {
    // Figure out the length of the data in bytes
    let len_bytes = data.len() * std::mem::size_of::<T>();

    // Create the buffer with the size and usage we want
    let buffer = gpu
        .create_buffer()
        .with_size(len_bytes as u32)
        .with_usage(usage)
        .build()?;

    // Map the transfer buffer's memory into a place we can copy into, and copy the data
    //
    // Note: We set `cycle` to true since we're reusing the same transfer buffer to
    // initialize both the vertex and index buffer. This makes SDL synchronize the transfers
    // so that one doesn't interfere with the other.
    let mut map = transfer_buffer.map::<T>(gpu, true);
    let mem = map.mem_mut();
    for (index, &value) in data.iter().enumerate() {
        mem[index] = value;
    }

    // Now unmap the memory since we're done copying
    map.unmap();

    // Finally, add a command to the copy pass to upload this data to the GPU
    //
    // Note: We also set `cycle` to true here for the same reason.
    copy_pass.upload_to_gpu_buffer(
        TransferBufferLocation::new()
            .with_offset(0)
            .with_transfer_buffer(transfer_buffer),
        BufferRegion::new()
            .with_offset(0)
            .with_size(len_bytes as u32)
            .with_buffer(&buffer),
        true,
    );

    Ok(buffer)
}
