extern crate sdl3;

use sdl3::audio::{AudioCallback, AudioFormat, AudioRecordingCallback, AudioSpec, AudioStream};
use sdl3::AudioSubsystem;
use std::i16;
use std::sync::mpsc;
use std::time::Duration;

const RECORDING_LENGTH_SECONDS: usize = 3;

struct Recording {
    record_buffer: Vec<i16>,
    pos: usize,
    done_sender: mpsc::Sender<Vec<i16>>,
    done: bool,
}

// Append the input of the callback to the record_buffer.
// When the record_buffer is full, send it to the main thread via done_sender.
impl AudioRecordingCallback<i16> for Recording {
    fn callback(&mut self, stream: &mut AudioStream, _available: i32) {
        if self.done {
            return;
        }

        self.pos += stream
            .read_i16_samples(&mut self.record_buffer[self.pos..])
            .unwrap();

        if self.pos >= self.record_buffer.len() {
            self.done = true;
            self.done_sender
                .send(self.record_buffer.clone())
                .expect("could not send record buffer");
        }
    }
}

fn record(
    audio_subsystem: &AudioSubsystem,
    desired_spec: &AudioSpec,
) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    println!("Capturing {RECORDING_LENGTH_SECONDS:} seconds... Please rock!");

    let (done_sender, done_receiver) = mpsc::channel();

    let capture_device = audio_subsystem.open_recording_stream(
        desired_spec,
        Recording {
            record_buffer: vec![
                0;
                desired_spec.freq.unwrap() as usize
                    * RECORDING_LENGTH_SECONDS
                    * desired_spec.channels.unwrap() as usize
            ],
            pos: 0,
            done_sender,
            done: false,
        },
    )?;

    println!("AudioDriver: {:?}", audio_subsystem.current_audio_driver());
    capture_device.resume()?;

    // Wait until the recording is done.
    let recorded_vec = done_receiver.recv().map_err(|e| e.to_string())?;

    capture_device.pause()?;

    // Device is automatically closed when dropped.
    // Depending on your system it might be even important that the capture_device is dropped
    // before the playback starts.

    Ok(recorded_vec)
}

/// Returns a percent value
fn calculate_average_volume(recorded_vec: &[i16]) -> f32 {
    let sum: i64 = recorded_vec.iter().map(|&x| (x as i64).abs()).sum();
    (sum as f32) / (recorded_vec.len() as f32) / (i16::MAX as f32) * 100.0
}

/// Returns a percent value
fn calculate_max_volume(recorded_vec: &[i16]) -> f32 {
    let max: i64 = recorded_vec
        .iter()
        .map(|&x| (x as i64).abs())
        .max()
        .expect("expected at least one value in recorded_vec");
    (max as f32) / (i16::MAX as f32) * 100.0
}

struct SoundPlayback {
    data: Vec<i16>,
}

impl AudioCallback<i16> for SoundPlayback {
    fn callback(&mut self, stream: &mut AudioStream, _requested: i32) {
        stream.put_data_i16(&self.data).unwrap();
    }
}

fn replay_recorded_vec(
    audio_subsystem: &AudioSubsystem,
    desired_spec: &AudioSpec,
    recorded_vec: Vec<i16>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Playing...");

    let playback_device =
        audio_subsystem.open_playback_stream(desired_spec, SoundPlayback { data: recorded_vec })?;

    // Start playback
    playback_device.resume()?;

    std::thread::sleep(Duration::from_secs(RECORDING_LENGTH_SECONDS as u64));
    // Device is automatically closed when dropped

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpec {
        freq: Some(48000),
        channels: Some(2),
        format: Some(AudioFormat::s16_sys()),
    };

    let recorded_vec = record(&audio_subsystem, &desired_spec)?;

    println!(
        "Average Volume of your Recording = {:?}%",
        calculate_average_volume(&recorded_vec)
    );
    println!(
        "Max Volume of your Recording = {:?}%",
        calculate_max_volume(&recorded_vec)
    );

    replay_recorded_vec(&audio_subsystem, &desired_spec, recorded_vec)?;

    Ok(())
}
