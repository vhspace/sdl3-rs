use sdl3::gpu::{Device, ShaderFormat};

/// `Device::new` takes a `VideoSubsystem` because SDL refuses to create a GPU
/// device without one. The device holds on to the subsystem, so dropping the
/// caller's handles first must not shut video down underneath it.
#[test]
fn device_keeps_video_subsystem_alive() {
    let sdl_context = match sdl3::init() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Skipping gpu test: failed to init SDL: {err}");
            return;
        }
    };
    let video_subsystem = match sdl_context.video() {
        Ok(video) => video,
        Err(err) => {
            eprintln!("Skipping gpu test: no video device available: {err}");
            return;
        }
    };

    let device = match Device::new(
        &video_subsystem,
        ShaderFormat::SPIRV | ShaderFormat::DXIL | ShaderFormat::DXBC | ShaderFormat::METALLIB,
        false,
    ) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Skipping gpu test: no GPU device available: {err}");
            return;
        }
    };

    drop(video_subsystem);
    assert!(device.acquire_command_buffer().is_ok());
}
