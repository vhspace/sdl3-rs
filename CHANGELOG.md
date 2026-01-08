# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.17.2] - 2026-01-08

### <!-- 1 -->Changed
- Make Event::to_ll public ([#304](https://github.com/vhspace/sdl3-rs/pull/304))

### <!-- 2 -->Fixed
- Add compile error for non-functional gfx feature ([#303](https://github.com/vhspace/sdl3-rs/pull/303))
- Add explicit lifetimes to Texture return types ([#302](https://github.com/vhspace/sdl3-rs/pull/302))

## [0.17.1] - 2026-01-07

### <!-- 0 -->Added
- Add support for sdl3-main ([#298](https://github.com/vhspace/sdl3-rs/pull/298))
- Added support for window progress value API ([#294](https://github.com/vhspace/sdl3-rs/pull/294))

### <!-- 1 -->Changed
- Add CLAUDE.local.md to gitignore

### <!-- 2 -->Fixed
- Align Keycode with SDL3's u32 representation ([#296](https://github.com/vhspace/sdl3-rs/pull/296))
- Add explicit lifetime annotations ([#293](https://github.com/vhspace/sdl3-rs/pull/293))

### <!-- 4 -->Dependencies
- Bump libc from 0.2.178 to 0.2.179 ([#299](https://github.com/vhspace/sdl3-rs/pull/299))

## [0.17.0] - 2026-01-01

### <!-- 0 -->Added
- Add AudioStream::device_paused tests
- Expand AudioStream tests
- Add AudioStream::device_paused
- Surface SDL audio device info
- Expose SDL3 audio stream controls
- Expose SDL_GetCurrentDirectory ([#278](https://github.com/vhspace/sdl3-rs/pull/278))
- Add lifetime to TextureSamplerBinding ([#273](https://github.com/vhspace/sdl3-rs/pull/273))

### <!-- 1 -->Changed
- Ignores
- Update to sdl3-sys 0.6 ([#286](https://github.com/vhspace/sdl3-rs/pull/286))
- Merge pull request #284 from vhspace/test/audio-device-dummy
- Merge remote-tracking branch 'origin/master' into test/audio-device-dummy
- Merge
- Run tests under xvfb x11 driver
- Fix tests job indentation
- Run tests under xvfb with dummy video
- Ensure tests use bundled SDL
- Merge remote-tracking branch 'origin/master' into ci-test-matrix
- Add feature-specific tests
- Revert "ci: build image feature"
- Build image feature
- Merge pull request #283 from vhspace/feature/audio-stream-tests
- Merge pull request #282 from vhspace/doc/fix-metal-view-doc
- Merge pull request #281 from vhspace/feature/audio-stream-device-paused
- Merge pull request #280 from vhspace/feature/audio-device-query
- Merge pull request #279 from vhspace/feature/audio-stream-extras
- Run tests under xvfb ([#276](https://github.com/vhspace/sdl3-rs/pull/276))
- CI tests ([#275](https://github.com/vhspace/sdl3-rs/pull/275))

### <!-- 2 -->Fixed
- Allow window test to skip in headless env
- Skip renderer test without video device
- Skip clipboard when video device missing
- Align doctest examples with current API
- Align doctest examples with current API ([#274](https://github.com/vhspace/sdl3-rs/pull/274))

### <!-- 4 -->Dependencies
- Bump peter-evans/create-pull-request from 7 to 8 ([#269](https://github.com/vhspace/sdl3-rs/pull/269))
- Bump libc from 0.2.177 to 0.2.178 ([#264](https://github.com/vhspace/sdl3-rs/pull/264))
- Bump sdl3-sys from 0.5.10+SDL3-3.2.26 to 0.5.11+SDL3-3.2.28 ([#266](https://github.com/vhspace/sdl3-rs/pull/266))

### <!-- 5 -->Documentation
- Fix repository link in README ([#290](https://github.com/vhspace/sdl3-rs/pull/290))
- Clarify feature flags ([#285](https://github.com/vhspace/sdl3-rs/pull/285))
- Fix metal_view documentation typo

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

[0.17.2]: https://github.com/vhspace/sdl3-rs/compare/v0.17.1...v0.17.2
[0.17.1]: https://github.com/vhspace/sdl3-rs/compare/v0.17.0...v0.17.1
[0.17.0]: https://github.com/vhspace/sdl3-rs/compare/v0.16.4...v0.17.0
[0.16.4]: https://github.com/vhspace/sdl3-rs/compare/v0.16.2...v0.16.4
[0.16.2]: https://github.com/vhspace/sdl3-rs/compare/v0.16.1...v0.16.2
[0.16.1]: https://github.com/vhspace/sdl3-rs/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/vhspace/sdl3-rs/compare/v0.15.1...v0.16.0
[0.15.1]: https://github.com/vhspace/sdl3-rs/compare/v0.14.42...v0.15.1
[0.14.42]: https://github.com/vhspace/sdl3-rs/compare/v0.14.41...v0.14.42
[0.14.41]: https://github.com/vhspace/sdl3-rs/compare/v0.14.40...v0.14.41
[0.14.40]: https://github.com/vhspace/sdl3-rs/compare/v0.14.36...v0.14.40

<!-- generated by git-cliff -->
