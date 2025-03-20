extern crate sdl3;

use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream};
use std::time::Duration;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    buffer: Vec<f32>,
}

impl AudioCallback<f32> for SquareWave {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        self.buffer.resize(requested as usize, 0.0);

        // Generate a square wave
        for x in self.buffer.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }

        stream.put_data_f32(&self.buffer).unwrap();
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

    let device = audio_subsystem.open_playback_stream(
        &desired_spec,
        SquareWave {
            phase_inc: 440.0 / desired_spec.freq.unwrap() as f32,
            phase: 0.0,
            volume: 0.25,
            buffer: Vec::new(),
        },
    )?;

    // Start playback
    device.resume()?;

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(2_000));

    // Device is automatically closed when dropped

    Ok(())
}
