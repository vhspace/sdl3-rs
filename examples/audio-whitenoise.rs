extern crate rand;
extern crate sdl3;

use sdl3::audio::{AudioCallback, AudioSpec};
use std::{sync::{Arc, Mutex}, time::Duration};

struct MyCallback {
    volume: Arc<Mutex<f32>>, // it seems like AudioStreamWithCallback<> is supposed to have a lock() method which allows us to modify MyCallback, but that function no longer seems to exist
}
impl AudioCallback<f32> for MyCallback {
    fn callback(&mut self, out: &mut [f32]) {
        use self::rand::{thread_rng, Rng};
        let mut rng = thread_rng();

        let Ok(volume) = self.volume.lock() else {return};
        // Generate white noise
        for x in out.iter_mut() {
            *x = (rng.gen_range(0.0 ..= 2.0) - 1.0) * *volume;
        }
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpec {
        freq: Some(44_100),
        channels: Some(1), // mono
        format: None,
    };
    let device = audio_subsystem.open_playback_device(&desired_spec)?;

    // None: use default device
    let volume = Arc::new(Mutex::new(0.5));
    let device = audio_subsystem.open_playback_stream_with_callback(&device, &desired_spec, MyCallback { volume: volume.clone() })?;

    // Start playback
    device.resume()?;

    // Play for 1 second
    std::thread::sleep(Duration::from_millis(1_000));

    if let Ok(mut volume) = volume.lock() {
        *volume = 0.25;
    }

    // Play for another second
    std::thread::sleep(Duration::from_millis(1_000));

    // Device is automatically closed when dropped

    Ok(())
}
