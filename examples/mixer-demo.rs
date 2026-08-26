/// Demonstrates audio mixing with SDL3_mixer.
///
/// Usage: mixer-demo <music_file> [sound_effect_file]
use sdl3::mixer;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./mixer-demo music.[mp3|wav|ogg] [sound-effect.[mp3|wav|ogg]]");
        return Ok(());
    }

    println!("SDL_mixer version: {}", mixer::get_linked_version());

    let sdl = sdl3::init()?;
    let _audio = sdl.audio()?;

    // Initialize SDL_mixer
    let _ctx = mixer::init()?;

    // List available decoders
    let n = mixer::get_num_audio_decoders();
    println!("available audio decoders: {n}");
    for i in 0..n {
        if let Some(name) = mixer::get_audio_decoder(i) {
            println!("  decoder {i} => {name}");
        }
    }

    // Create mixer device with default format
    let mixer_dev = mixer::Mixer::open_device(None)?;
    let fmt = mixer_dev.format()?;
    println!("mixer format: {}Hz, {} channels", fmt.freq, fmt.channels);

    // Load music from file
    let music = mixer_dev.load_audio(Path::new(&args[1]), false)?;
    println!("music duration: {} frames", music.duration());

    // Create a track and play the music
    let music_track = mixer_dev.create_track()?;
    music_track.set_audio(&music)?;
    music_track.set_loops(-1)?; // loop forever
    music_track.tag("music")?;
    music_track.play()?;
    println!("playing music...");

    // Optionally play a sound effect
    if let Some(sound_path) = args.get(2) {
        let sfx = mixer_dev.load_audio(Path::new(sound_path), true)?;
        let sfx_track = mixer_dev.create_track()?;
        sfx_track.set_audio(&sfx)?;
        sfx_track.tag("sfx")?;
        sfx_track.play()?;
        println!("playing sound effect...");
    } else {
        // Generate a test sine wave
        let sine = mixer_dev.create_sine_wave(440, 0.25, 2000)?;
        let sine_track = mixer_dev.create_track()?;
        sine_track.set_audio(&sine)?;
        sine_track.tag("sfx")?;
        sine_track.play()?;
        println!("playing 440Hz sine wave...");
    }

    // Play for a bit
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Lower the music volume
    println!("lowering music volume to 50%...");
    music_track.set_gain(0.5)?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Fade out with a 2-second fade
    let fade_frames = music_track.ms_to_frames(2000);
    println!("fading out music...");
    music_track.stop(fade_frames)?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Stop everything
    mixer_dev.stop_all(0)?;
    println!("done.");

    Ok(())
}
