use crate::sys;

pub const CACHELINESIZE: u8 = 128;

#[doc(alias = "SDL_GetNumLogicalCPUCores")]
pub fn cpu_count() -> i32 {
    unsafe { sys::cpuinfo::SDL_GetNumLogicalCPUCores() }
}

#[doc(alias = "SDL_GetCPUCacheLineSize")]
pub fn cpu_cache_line_size() -> i32 {
    unsafe { sys::cpuinfo::SDL_GetCPUCacheLineSize() }
}

#[doc(alias = "SDL_HasAltiVec")]
pub fn has_alti_vec() -> bool {
    unsafe { sys::cpuinfo::SDL_HasAltiVec() == true }
}

#[doc(alias = "SDL_HasMMX")]
pub fn has_mmx() -> bool {
    unsafe { sys::cpuinfo::SDL_HasMMX() == true }
}

#[doc(alias = "SDL_HasSSE")]
pub fn has_sse() -> bool {
    unsafe { sys::cpuinfo::SDL_HasSSE() == true }
}

#[doc(alias = "SDL_HasSSE2")]
pub fn has_sse2() -> bool {
    unsafe { sys::cpuinfo::SDL_HasSSE2() == true }
}

#[doc(alias = "SDL_HasSSE3")]
pub fn has_sse3() -> bool {
    unsafe { sys::cpuinfo::SDL_HasSSE3() == true }
}

#[doc(alias = "SDL_HasSSE41")]
pub fn has_sse41() -> bool {
    unsafe { sys::cpuinfo::SDL_HasSSE41() == true }
}

#[doc(alias = "SDL_HasSSE42")]
pub fn has_sse42() -> bool {
    unsafe { sys::cpuinfo::SDL_HasSSE42() == true }
}

#[doc(alias = "SDL_HasAVX")]
pub fn has_avx() -> bool {
    unsafe { sys::cpuinfo::SDL_HasAVX() == true }
}

#[doc(alias = "SDL_HasAVX2")]
pub fn has_avx2() -> bool {
    unsafe { sys::cpuinfo::SDL_HasAVX2() == true }
}

#[doc(alias = "SDL_HasAVX512F")]
pub fn has_avx512f() -> bool {
    unsafe { sys::cpuinfo::SDL_HasAVX512F() == true }
}

#[doc(alias = "SDL_GetSystemRAM")]
pub fn system_ram() -> i32 {
    unsafe { sys::cpuinfo::SDL_GetSystemRAM() }
}
