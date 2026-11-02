// SDL_GetPrefPath returns NULL when it can't work out a data directory, which
// on Linux happens with neither XDG_DATA_HOME nor HOME set. Kept in its own
// test binary since it clears process-wide env vars.
#[cfg(target_os = "linux")]
#[test]
fn pref_path_null_is_an_error() {
    std::env::remove_var("XDG_DATA_HOME");
    std::env::remove_var("HOME");

    let err = sdl3::filesystem::get_pref_path("org", "app").unwrap_err();
    assert!(matches!(err, sdl3::filesystem::PrefPathError::SdlError(_)));
}
