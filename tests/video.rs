extern crate sdl3;

/*
#[test]
fn display_name_no_segfault() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video();
    if let Ok(video_subsystem) = video_subsystem {
        // hopefully no one has a 100 screen to see this test pass

        // With new display handling, this test is irrelevant
        let r = Display::from_ll(99).get_name();
                ^^^^^^^^^^^^^^^^^^^^ Errors

        assert!(r.is_err());
    } // in Err(), environment has no video device (for instance travis)
      // so ignore it
}
*/
