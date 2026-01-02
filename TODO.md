# SDL3-RS Maintenance TODO List

This document tracks maintenance tasks, technical debt, and areas needing improvement in the sdl3-rs codebase.

**Last Updated:** 2026-01-01
**Status:** Actively maintained

---

## 🔴 High Priority (Critical Issues)

### 1. Replace `panic!` with Result Types
**Impact:** Improves error handling and prevents crashes
**Effort:** Medium-High (requires API changes)

| File | Panic Count | Priority |
|------|-------------|----------|
| `src/sdl3/render.rs` | 15+ | High |
| `src/sdl3/surface.rs` | 8 | High |
| `src/sdl3/event.rs` | 8 | High |
| Other modules | 10+ | Medium |

**Action Items:**
- [ ] Convert panics in render.rs to Result<T, RenderError>
- [ ] Convert panics in surface.rs to Result<T, SurfaceError>
- [ ] Convert panics in event.rs to Result<T, EventError>
- [ ] Update examples to handle new error types

---

### 2. Compiler Warnings (31 total, 12 fixed, 19 remaining)

**Fixed ✅:**
- [x] 2x unnecessary unsafe blocks (sdl.rs, version.rs)
- [x] 2x PathBuf vs Path (build.rs)
- [x] 2x dead code (filesystem.rs, render.rs)
- [x] 6x lifetime annotations (audio.rs, event.rs)

**Remaining ⚠️:**
- [ ] 19x lifetime annotation warnings (add `'_` to return types)
  - filesystem.rs:394
  - gpu/device.rs:119, 124, 129
  - keyboard/mod.rs:71, 101
  - mouse/mod.rs:311, 334
  - mouse/relative.rs:120, 143
  - render.rs:982, 1004, 1018, 1032, 1073, 1090, 1625
  - video.rs:224, 385

**Quick Fix Script:**
```bash
# These can be fixed with pattern: `Type` -> `Type<'_>`
# Example: SomeIterator -> SomeIterator<'_>
```

---

### 3. Implement SysWM Events
**Impact:** Critical for platform-specific window handling
**Effort:** High (requires FFI bindings)

**Location:** `src/sdl3/event.rs`
- Line 317: `// TODO: SysWM = sys::events::SDL_EVENT_SYSWM`
- Line 694: `// TODO: SysWMEvent`
- Line 1816: `// TODO: SysWMEventType`

**Blocked by:** Need sdl3-sys to expose SysWM types properly

---

## 🟡 Medium Priority (Important Improvements)

### 4. Code TODOs and FIXMEs (17 items)

#### Audio Module
- [ ] **audio.rs:1345, 1367** - Cache specs to avoid repeated get_format() calls
  - **Benefit:** Performance improvement
  - **Effort:** Low

#### Surface Module
- [ ] **surface.rs:549** - Replace unsafe transmute with safe alternative
  - **Current:** `mem::transmute(rect.as_ref())`
  - **Need:** Safe Rect conversion method

#### Mouse Module
- [ ] **mouse/mod.rs:74** - Fix Surface parameter handling for cursor creation
  - **Issue:** Unclear how to pass Surface correctly

#### Video Module
- [ ] **video.rs:1137, 1255** - Use OsStr::to_cstring() once stabilized in Rust stdlib
  - **Blocked by:** Rust stdlib stabilization
- [ ] **video.rs:2018** - Clarify SDL_SetWindowData API usage
  - **Issue:** Undocumented in SDL3

#### Pixels Module
- [ ] **pixels.rs:25** - Improve validate_int to remove workaround

#### GFX Module
- [ ] **gfx/primitives.rs:517** - Improve polygon drawing API design
  - **Current:** FIXME about pointer tuple usage

#### Mixer Module (Blocked on Upstream)
- [ ] **mixer/mod.rs:956** - Mix_HookMusic
- [ ] **mixer/mod.rs:957** - Mix_GetMusicHookData
- [ ] **mixer/mod.rs:1009** - Mix_RegisterEffect
- [ ] **mixer/mod.rs:1010** - Mix_UnregisterEffect
- [ ] **mixer/mod.rs:1011** - Mix_SetPostMix
- **Blocked by:** Awaiting stable SDL3_mixer C library release

---

### 5. Add Missing Tests

**Current Coverage:** 7 test files, 18 tests total

**Missing Test Coverage:**
- [ ] GPU modules (device, buffer, texture, pass)
- [ ] Input handling (gamepad, joystick, haptic)
- [ ] Platform-specific (dialog, pen, sensor, touch)
- [ ] Extensions (TTF, image, mixer)
- [ ] Properties and hints
- [ ] Surface transformations
- [ ] Render state management

**Recommended Priority:**
1. GPU module tests (new major SDL3 feature)
2. Input handling tests
3. Platform-specific tests

---

### 6. Documentation Gaps

**Modules Needing Documentation:**

| Module | Status | Action |
|--------|--------|--------|
| surface.rs | ⚠️ Poor | Add function-level docs and examples |
| gfx/primitives.rs | ⚠️ Minimal | Document drawing functions |
| gpu/buffer.rs | ⚠️ Minimal | Document builder patterns |
| mixer/mod.rs | ⚠️ Blocked | Awaiting C library |

**Positive:**
- ✅ 577 `#[doc(alias)]` annotations for SDL3 C API mapping

**Action Items:**
- [ ] Add usage examples to public functions in surface.rs
- [ ] Document GFX primitives drawing functions
- [ ] Add GPU buffer builder pattern documentation
- [ ] Create tutorial-style docs for common use cases

---

## 🟢 Low Priority (Nice to Have)

### 7. SDL3 Missing Features

**Not Yet Implemented:**
- [ ] **Camera API** - New SDL3 camera handling
- [ ] **Main Callback API** - SDL3's new event loop callback system
- [ ] **Enhanced Property System** - Expand basic properties implementation

**Blocked Extension Libraries:**
- 🟨 **SDL_mixer** - Awaiting stable C release
- 🟨 **SDL_sound** - Awaiting stable C release
- 🟨 **SDL_net** - Awaiting stable C release
- 🟨 **SDL_gfx** - Waiting on C library improvements
- ❌ **Dear ImGUI** - Not started
- ❌ **RmlUI** - Not started
- ❌ **SDL_shadercross** - Not started

---

### 8. Missing Examples

**Well-Covered:**
- ✅ Audio (6 examples)
- ✅ GPU (4 examples)
- ✅ Rendering (10+ examples)
- ✅ Input (5+ examples)

**Missing:**
- [ ] Complete TTF text rendering workflow
- [ ] SDL_image advanced operations
- [ ] SDL_mixer music/effects management
- [ ] GPU compute shaders
- [ ] GPU multi-pass rendering
- [ ] Properties/hints management
- [ ] Custom event handling
- [ ] Clipboard operations

---

## 📊 Maintenance Statistics

- **Lines of Code:** ~15,000+ in src/sdl3/
- **Modules:** 40+ Rust modules
- **Examples:** 52 working examples
- **Tests:** 7 test files, 18 total tests
- **TODOs/FIXMEs:** 17 identified
- **Compiler Warnings:** 31 total (12 fixed, 19 remaining)
- **Panic Calls:** 30+ (should be Result types)

---

## 🛠️ Build Requirements

### Important: Use Build-from-Source Features

**Do NOT use system SDL3.** Instead, use one of these features when building:

```bash
# Option 1: Build SDL3 from source
cargo build --features build-from-source

# Option 2: Build SDL3 from source as static library
cargo build --features build-from-source-static

# For testing
cargo test --features build-from-source
```

**Reason:** System SDL3 versions may not match the sdl3-sys bindings version, causing compatibility issues.

---

## 🔄 Recent Changes (from CHANGELOG.md)

**v0.17.0:**
- Added lifetime to TextureSamplerBinding
- Aligned doctest examples with current API
- Updated dependencies (libc, sdl3-sys)

**v0.16.4:**
- Added SDL_SetWindowHitTest wrapper
- Added SDL_SetAudioStreamGain
- Fixed filesystem errors
- Fixed keycode handling

---

## 📝 How to Contribute

1. Pick a task from this list
2. Create an issue on GitHub referencing the TODO item
3. Submit a PR with your fix
4. Update this TODO.md to mark the item as complete

**Priority Order:**
1. 🔴 High Priority items (critical issues)
2. 🟡 Medium Priority items (improvements)
3. 🟢 Low Priority items (nice to have)

---

## 📞 Questions or Discussion

Join the #rust channel on [Discord](https://discord.gg/qMyEpKVnCD) to discuss:
- Implementation questions
- API design decisions
- Maintenance priorities
- Feature requests

---

*This TODO list was generated by automated codebase analysis on 2026-01-01.*
