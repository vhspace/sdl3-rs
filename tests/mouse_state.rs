// SDL only allows reading the mouse state on the main thread, but EventPump is
// Sync, so the check has to happen at runtime.
#[test]
fn mouse_state_off_main_thread_panics() {
    let sdl = sdl3::init().unwrap();
    let pump = sdl.event_pump().unwrap();

    // The thread that called SDL_Init is SDL's main thread.
    let _ = pump.mouse_state();
    let _ = pump.relative_mouse_state();

    std::thread::scope(|s| {
        assert!(s.spawn(|| pump.mouse_state()).join().is_err());
        assert!(s.spawn(|| pump.relative_mouse_state()).join().is_err());
    });
}
