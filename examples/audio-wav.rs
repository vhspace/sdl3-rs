extern crate sdl3;

use sdl3::audio::{AudioSpec, AudioSpecWAV};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wav_file: Cow<'static, Path> = match std::env::args().nth(1) {
        None => Cow::from(Path::new("./assets/sine.wav")),
        Some(s) => Cow::from(PathBuf::from(s)),
    };

    let sdl_context = sdl3::init()?;
    let audio_subsystem = sdl_context.audio()?;

    // Load the WAV file
    let wav = AudioSpecWAV::load_wav(&wav_file)?;
    println!(
        "Loaded WAV: {} Hz, {} channels, {:?}",
        wav.freq, wav.channels, wav.format
    );

    // Create an audio spec matching the WAV file
    let wav_spec = AudioSpec {
        freq: Some(wav.freq),
        channels: Some(wav.channels as i32),
        format: Some(wav.format),
    };

    // Open a playback device and create a stream for it
    // SDL will handle any necessary format conversion
    let device = audio_subsystem.open_playback_device(&wav_spec)?;
    let stream = device.open_device_stream(Some(&wav_spec))?;

    // Queue the WAV data
    stream.put_data(wav.buffer())?;

    // Start playback
    stream.resume()?;

    // Play for the duration of the audio (estimate based on buffer size)
    // For a more accurate duration, calculate from sample rate and buffer length
    let bytes_per_sample = match wav.format {
        sdl3::audio::AudioFormat::U8 | sdl3::audio::AudioFormat::S8 => 1,
        sdl3::audio::AudioFormat::S16LE | sdl3::audio::AudioFormat::S16BE => 2,
        sdl3::audio::AudioFormat::S32LE
        | sdl3::audio::AudioFormat::S32BE
        | sdl3::audio::AudioFormat::F32LE
        | sdl3::audio::AudioFormat::F32BE => 4,
        _ => 2, // default assumption
    };
    let total_samples = wav.buffer().len() / (bytes_per_sample * wav.channels as usize);
    let duration_ms = (total_samples as u64 * 1000) / wav.freq as u64;

    println!("Playing for {} ms...", duration_ms);
    std::thread::sleep(Duration::from_millis(duration_ms + 100)); // add a small buffer

    // Stream and device are automatically closed when dropped

    Ok(())
}
