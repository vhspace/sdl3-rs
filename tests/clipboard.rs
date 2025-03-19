extern crate sdl3;

#[test]
fn test_clipboard() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let clipboard = video_subsystem.clipboard();
    let text = "Hello World!";

    // set some text
    assert!(clipboard.set_clipboard_text(text).is_ok());
    assert!(clipboard.has_clipboard_text());
    // get it back
    assert_eq!(clipboard.clipboard_text(), Ok(text.to_string()));
}
