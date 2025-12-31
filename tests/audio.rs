extern crate sdl3;

fn init_audio_subsystem() -> Option<(sdl3::Sdl, sdl3::AudioSubsystem)> {
    std::env::set_var("SDL_AUDIODRIVER", "dummy");
    let sdl = match sdl3::init() {
        Ok(sdl) => sdl,
        Err(err) => {
            eprintln!("Skipping audio test: failed to init SDL: {err}");
            return None;
        }
    };
    let audio = match sdl.audio() {
        Ok(audio) => audio,
        Err(err) => {
            eprintln!("Skipping audio test: failed to init audio subsystem: {err}");
            return None;
        }
    };
    Some((sdl, audio))
}

#[test]
fn audio_spec_wav() {
    let wav = sdl3::audio::AudioSpecWAV::load_wav("./assets/sine.wav").unwrap();

    assert_eq!(wav.freq, 22_050);
    assert_eq!(wav.format, sdl3::audio::AudioFormat::S16LE);
    assert_eq!(wav.channels, 1);

    let buffer = wav.buffer();
    assert_eq!(buffer.len(), 4_410);
}

#[test]
fn audio_stream_device_paused_state_transitions() {
    let Some((_sdl, audio)) = init_audio_subsystem() else {
        return;
    };

    let device = match audio.open_playback_device(&sdl3::audio::AudioSpec::default()) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Skipping stream test: failed to open dummy playback device: {err}");
            return;
        }
    };

    let stream = match device.open_device_stream(None) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Skipping stream test: failed to open device stream: {err}");
            return;
        }
    };

    // Newly created device streams start paused.
    assert!(stream.device_paused().expect("device_paused failed"));

    stream.resume().expect("resume failed");
    assert!(
        !stream.device_paused().expect("device_paused failed after resume"),
        "Device should report unpaused after resuming"
    );

    stream.pause().expect("pause failed");
    assert!(
        stream.device_paused().expect("device_paused failed after pause"),
        "Device should report paused after pausing"
    );
}

#[test]
fn audio_stream_reports_format_and_queue() {
    let Some((_sdl, audio)) = init_audio_subsystem() else {
        return;
    };

    let device = match audio.open_playback_device(&sdl3::audio::AudioSpec::default()) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Skipping format test: failed to open dummy playback device: {err}");
            return;
        }
    };

    let stream = match device.open_device_stream(None) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Skipping format test: failed to open device stream: {err}");
            return;
        }
    };

    let (src, dst) = stream.get_format().expect("get_format failed");
    assert!(
        src.is_some() || dst.is_some(),
        "Expected dummy driver to report at least one side of the stream format"
    );

    let avail = stream.available_bytes().expect("available_bytes failed");
    assert!(avail >= 0, "available bytes should be non-negative");
    let queued = stream.queued_bytes().expect("queued_bytes failed");
    assert!(queued >= 0, "queued bytes should be non-negative");

    stream.clear().expect("clear failed");
    stream.pause().expect("pause failed");
    stream.resume().expect("resume failed");
}
