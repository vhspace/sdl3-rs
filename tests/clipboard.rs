extern crate sdl3;

#[test]
fn test_clipboard() {
    let sdl_context = match sdl3::init() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Skipping clipboard test: failed to init SDL: {err}");
            return;
        }
    };
    let video_subsystem = match sdl_context.video() {
        Ok(video) => video,
        Err(err) => {
            eprintln!("Skipping clipboard test: no video device available: {err}");
            return;
        }
    };
    let clipboard = video_subsystem.clipboard();
    let text = "Hello World!";

    // set some text
    assert!(clipboard.set_clipboard_text(text).is_ok());
    assert!(clipboard.has_clipboard_text());
    // get it back
    assert_eq!(clipboard.clipboard_text(), Ok(text.to_string()));
}
