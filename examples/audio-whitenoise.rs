extern crate rand;
extern crate sdl3;

use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStreamInner};
use std::time::Duration;

struct MyCallback {
    volume: f32,
    buffer: Vec<f32>
}
impl AudioCallback<f32> for MyCallback {
    fn callback(&mut self, stream: &mut AudioStreamInner, requested: i32) {
        use self::rand::{thread_rng, Rng};
        let mut rng = thread_rng();

        self.buffer.resize(requested as usize, 0.0);

        // Generate white noise
        for x in self.buffer.iter_mut() {
            *x = (rng.gen_range(0.0..=2.0) - 1.0) * self.volume;
        }

        stream.put_data_f32(&self.buffer);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpec {
        freq: Some(48000),
        channels: Some(1), // mono
        format: Some(AudioFormat::f32_sys()),
    };
    let device = audio_subsystem.open_playback_device(&desired_spec)?;

    // None: use default device
    let mut device = audio_subsystem.open_playback_stream_with_callback(
        &device,
        &desired_spec,
        MyCallback {
            volume: 0.0,
            buffer: Vec::new()
        },
    )?;

    // Start playback
    device.resume()?;

    // Play for 1 second
    std::thread::sleep(Duration::from_millis(1_000));

    if let Some(mut context) = device.lock() {
        context.volume = 0.25;
    }

    // Play for another second
    std::thread::sleep(Duration::from_millis(1_000));

    // Device is automatically closed when dropped

    Ok(())
}
