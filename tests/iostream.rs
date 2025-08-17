extern crate sdl3;
use std::io::Read;

#[test]
fn iostream_from_vec() {
    let logo = std::fs::read("./assets/SDL_logo.bmp").unwrap();

    let mut ios = sdl3::iostream::IOStream::from_vec(logo.clone()).unwrap();

    let mut output = Vec::new();
    ios.read_to_end(&mut output).unwrap();

    assert_eq!(output, logo);
}
