# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### <!-- 0 -->Added
- Add lifetime to TextureSamplerBinding ([#273](https://github.com/vhspace/sdl3-rs/pull/273))

### <!-- 2 -->Fixed
- Align doctest examples with current API ([#274](https://github.com/vhspace/sdl3-rs/pull/274))

### <!-- 4 -->Dependencies
- Bump peter-evans/create-pull-request from 7 to 8 ([#269](https://github.com/vhspace/sdl3-rs/pull/269))
- Bump libc from 0.2.177 to 0.2.178 ([#264](https://github.com/vhspace/sdl3-rs/pull/264))
- Bump sdl3-sys from 0.5.10+SDL3-3.2.26 to 0.5.11+SDL3-3.2.28 ([#266](https://github.com/vhspace/sdl3-rs/pull/266))

## [0.16.4] - 2025-12-28

### <!-- 0 -->Added
- Derive Default for Color ([#272](https://github.com/vhspace/sdl3-rs/pull/272))
- Added SDL_SetWindowHitTest wrapper ([#268](https://github.com/vhspace/sdl3-rs/pull/268))
- Add SDL_SetAudioStreamGain ([#267](https://github.com/vhspace/sdl3-rs/pull/267))

### <!-- 2 -->Fixed
- Implemented Error for FileSystemError ([#265](https://github.com/vhspace/sdl3-rs/pull/265))
- Don't unwrap keycode and pass directly the Option. ([#263](https://github.com/vhspace/sdl3-rs/pull/263))

### <!-- 4 -->Dependencies
- Bump actions/cache from 4 to 5 ([#270](https://github.com/vhspace/sdl3-rs/pull/270))
- Bump actions/checkout from 5 to 6 ([#261](https://github.com/vhspace/sdl3-rs/pull/261))

## [0.16.2] - 2025-11-16

### <!-- 0 -->Added
- Add bitflags type (`WindowFlags`) for SDL_WindowFlags ([#252](https://github.com/vhspace/sdl3-rs/pull/252))
- Make draw_rect consistent with fill_rect generic parameter ([#253](https://github.com/vhspace/sdl3-rs/pull/253))

### <!-- 1 -->Changed
- 41:18.325143356Z INFO  sanzu::client_sdl2] Map video memory ([#257](https://github.com/vhspace/sdl3-rs/pull/257))

### <!-- 2 -->Fixed
- Correct previous attempt to fix EventWatch soundness hole ([#259](https://github.com/vhspace/sdl3-rs/pull/259))
- Add Send + Sync bounds for EventWatch to avoid soundness hole ([#256](https://github.com/vhspace/sdl3-rs/pull/256))
- Avoid segfault when device is drop after window ([#255](https://github.com/vhspace/sdl3-rs/pull/255))

### <!-- 4 -->Dependencies
- Bump bitflags from 2.9.4 to 2.10.0 ([#249](https://github.com/vhspace/sdl3-rs/pull/249))
- Bump sdl3-sys from 0.5.7+SDL3-3.2.24 to 0.5.10+SDL3-3.2.26 ([#251](https://github.com/vhspace/sdl3-rs/pull/251))

## [0.16.1] - 2025-10-17

### <!-- 0 -->Added
- Added EventSubsystem::set_event_enabled and EventSubsystem::event_enable. ([#237](https://github.com/vhspace/sdl3-rs/pull/237))

## [0.16.0] - 2025-10-17

### <!-- 0 -->Added
- Add draw_debug_text() ([#233](https://github.com/vhspace/sdl3-rs/pull/233))

### <!-- 1 -->Changed
- Update deps
- Remove rust cache ([#245](https://github.com/vhspace/sdl3-rs/pull/245))
- Automate changelog generation ([#231](https://github.com/vhspace/sdl3-rs/pull/231))

### <!-- 2 -->Fixed
- Change acquire_swapchain_texture to return an Option<Texture> ([#240](https://github.com/vhspace/sdl3-rs/pull/240))

## [0.15.1] - 2025-09-14

### <!-- 0 -->Added
- Add basic TTF_TextEngine wrapper ([#221](https://github.com/vhspace/sdl3-rs/pull/221))
- Add set_scale_mode and scale_mode to unsafe Texture ([#224](https://github.com/vhspace/sdl3-rs/pull/224))

### <!-- 1 -->Changed
- Bump minor version, sort of breaking changes
- Update log enums ([#222](https://github.com/vhspace/sdl3-rs/pull/222))
- Add lint pr title workflow ([#229](https://github.com/vhspace/sdl3-rs/pull/229))
- Remove build cache ([#228](https://github.com/vhspace/sdl3-rs/pull/228))

### <!-- 2 -->Fixed
- Expose viewport wrapper publicly ([#227](https://github.com/vhspace/sdl3-rs/pull/227))

### <!-- 3 -->Removed
- Remove PixelFormatEnum ([#223](https://github.com/vhspace/sdl3-rs/pull/223))

### <!-- 4 -->Dependencies
- Bump actions/checkout from 4 to 5 ([#211](https://github.com/vhspace/sdl3-rs/pull/211))
- Bump bitflags from 2.9.1 to 2.9.4 ([#225](https://github.com/vhspace/sdl3-rs/pull/225))
- Bump amannn/action-semantic-pull-request from 5 to 6 ([#230](https://github.com/vhspace/sdl3-rs/pull/230))

## [0.14.42] - 2025-09-04

### <!-- 4 -->Dependencies
- Bump sdl3-sys from 0.5.0+SDL3-3.2.12 to 0.5.5+SDL3-3.2.22 ([#226](https://github.com/vhspace/sdl3-rs/pull/226))

## [0.14.41] - 2025-08-24

### <!-- 0 -->Added
- Add Viewport wrapper and RenderPass::set_scissor ([#220](https://github.com/vhspace/sdl3-rs/pull/220))
- Add fence and blit operations ([#218](https://github.com/vhspace/sdl3-rs/pull/218))

## [0.14.40] - 2025-08-18

### <!-- 0 -->Added
- Added IOStream::from_vec.

### <!-- 1 -->Changed
- Rework checks workflows ([#203](https://github.com/vhspace/sdl3-rs/pull/203))
- Replace unsound gpu enums with wrapper structs ([#200](https://github.com/vhspace/sdl3-rs/pull/200))
- Fix GPU compute pass binding count and expose storage bindings ([#199](https://github.com/vhspace/sdl3-rs/pull/199))

### <!-- 2 -->Fixed
- Use new enum format ([#202](https://github.com/vhspace/sdl3-rs/pull/202))
- Allow to bind samplers for compute pass ([#201](https://github.com/vhspace/sdl3-rs/pull/201))

### <!-- 4 -->Dependencies
- Bump libc from 0.2.172 to 0.2.174 ([#210](https://github.com/vhspace/sdl3-rs/pull/210))
- Bump bitflags from 2.9.0 to 2.9.1 ([#207](https://github.com/vhspace/sdl3-rs/pull/207))
- Bump actions/cache from 3 to 4 ([#208](https://github.com/vhspace/sdl3-rs/pull/208))

[unreleased]: https://github.com/vhspace/sdl3-rs/compare/v0.16.4...HEAD
[0.16.4]: https://github.com/vhspace/sdl3-rs/compare/v0.16.2...v0.16.4
[0.16.2]: https://github.com/vhspace/sdl3-rs/compare/v0.16.1...v0.16.2
[0.16.1]: https://github.com/vhspace/sdl3-rs/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/vhspace/sdl3-rs/compare/v0.15.1...v0.16.0
[0.15.1]: https://github.com/vhspace/sdl3-rs/compare/v0.14.42...v0.15.1
[0.14.42]: https://github.com/vhspace/sdl3-rs/compare/v0.14.41...v0.14.42
[0.14.41]: https://github.com/vhspace/sdl3-rs/compare/v0.14.40...v0.14.41
[0.14.40]: https://github.com/vhspace/sdl3-rs/compare/v0.14.36...v0.14.40

<!-- generated by git-cliff -->
