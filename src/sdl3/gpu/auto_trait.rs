// manually checking auto-traits for a type
type _X = u32;
const _: () = _copy::<_X>();
const _: () = _send::<_X>();
const _: () = _sync::<_X>();
const _: () = _unpin::<_X>();

const fn _copy<T: Copy>() {}
const fn _send<T: Send>() {}
const fn _sync<T: Sync>() {}
const fn _unpin<T: Unpin>() {}

#[cfg(test)]
mod assertions {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    macro_rules! thread_safe {
        ($t:ty) => {
            assert_impl_all!($t: Sync);
            assert_not_impl_any!($t: Copy, Send, Unpin);
        };
    }
    macro_rules! thread_unsafe {
        ($t:ty) => {
            assert_not_impl_any!($t: Copy, Sync, Send, Unpin);
        };
    }

    use crate::gpu::*;

    // definitely not thread-safe
    thread_unsafe!(CommandBuffer);

    // at least SDL_GetGPUDeviceProperties is documented as
    // "safe to call from any thread", which implies that the device can be shared
    thread_safe!(Device);

    // these are ambiguous
    thread_safe!(Buffer);
    thread_safe!(Texture);

    // possibly thread-safe, but haven't checked yet
    thread_unsafe!(Sampler);
    thread_unsafe!(TransferBuffer);
    thread_unsafe!(GraphicsPipeline);
    thread_unsafe!(ComputePipeline);
    thread_unsafe!(Shader);
}
