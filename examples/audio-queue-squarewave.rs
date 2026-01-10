extern crate sdl3;

use sdl3::audio::{AudioFormat, AudioSpec};
use std::time::Duration;

fn gen_wave(sample_count: i32) -> Vec<i16> {
    // Generate a square wave
    let tone_volume = 1_000i16;
    let period = 48_000 / 256;
    let mut result = Vec::with_capacity(sample_count as usize);

    for x in 0..sample_count {
        result.push(if (x / period) % 2 == 0 {
            tone_volume
        } else {
            -tone_volume
        });
    }
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpec {
        freq: Some(48_000),
        channels: Some(2),
        format: Some(AudioFormat::s16_sys()),
    };

    // Open a playback device and create a stream for it
    let device = audio_subsystem.open_playback_device(&desired_spec)?;
    let stream = device.open_device_stream(Some(&desired_spec))?;

    // Generate 2 seconds of audio (48000 samples/sec * 2 channels * 2 seconds)
    let target_samples = 48_000 * 2 * 2;
    let wave = gen_wave(target_samples);

    // Queue the audio data
    stream.put_data_i16(&wave)?;

    // Start playback
    stream.resume()?;

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(2_000));

    // Stream and device are automatically closed when dropped

    Ok(())
}
