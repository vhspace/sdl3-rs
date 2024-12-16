use crate::sys;
use libc::c_void;
use std::ptr::NonNull;

/// Constructs a new timer using the boxed closure `callback`.
///
/// The timer is started immediately and will be canceled either:
///
/// * When the timer is dropped.
/// * When the callback returns a non-positive continuation interval.
///
/// The callback is run in a thread that is created and managed internally
/// by SDL3 from C. The callback **must not panic!**
#[must_use = "if unused the Timer will be dropped immediately"]
#[doc(alias = "SDL_AddTimer")]
pub fn add_timer(delay: u32, callback: TimerCallback) -> Timer {
    unsafe {
        // Allocate the callback on the heap and get a raw pointer.
        let callback_ptr = Box::into_raw(Box::new(callback));

        // Call SDL_AddTimer with the appropriate function signature.
        let timer_id =
            sys::timer::SDL_AddTimer(delay, Some(c_timer_callback), callback_ptr as *mut c_void);

        Timer {
            callback: Some(NonNull::new(callback_ptr).unwrap()),
            raw: timer_id,
        }
    }
}

/// Gets the number of milliseconds elapsed since the timer subsystem was initialized.
///
/// It's recommended to use another library for timekeeping, such as `time`.
#[doc(alias = "SDL_GetTicks")]
pub fn ticks() -> u64 {
    unsafe { sys::timer::SDL_GetTicks() }
}

/// Sleeps the current thread for the specified amount of milliseconds.
///
/// It's recommended to use `std::thread::sleep()` instead.
#[doc(alias = "SDL_Delay")]
pub fn delay(ms: u32) {
    unsafe { sys::timer::SDL_Delay(ms) }
}

#[doc(alias = "SDL_GetPerformanceCounter")]
pub fn performance_counter() -> u64 {
    unsafe { sys::timer::SDL_GetPerformanceCounter() }
}

#[doc(alias = "SDL_GetPerformanceFrequency")]
pub fn performance_frequency() -> u64 {
    unsafe { sys::timer::SDL_GetPerformanceFrequency() }
}

/// Type alias for the timer callback function.
pub type TimerCallback = Box<dyn FnMut() -> u32 + Send + 'static>;

pub struct Timer {
    callback: Option<NonNull<TimerCallback>>,
    raw: sys::timer::SDL_TimerID,
}

impl Timer {
    /// Returns the closure as a trait-object and cancels the timer
    /// by consuming it.
    pub fn into_inner(mut self) -> TimerCallback {
        unsafe {
            sys::timer::SDL_RemoveTimer(self.raw);
            if let Some(callback_ptr) = self.callback.take() {
                // Reconstruct the Box from the raw pointer.
                let callback = Box::from_raw(callback_ptr.as_ptr());
                callback
            } else {
                panic!("Timer callback already taken");
            }
        }
    }
}

impl Drop for Timer {
    #[inline]
    #[doc(alias = "SDL_RemoveTimer")]
    fn drop(&mut self) {
        unsafe {
            sys::timer::SDL_RemoveTimer(self.raw);
            if let Some(callback_ptr) = self.callback.take() {
                // Reclaim the Box and drop it.
                let _ = Box::from_raw(callback_ptr.as_ptr());
            }
        }
    }
}

extern "C" fn c_timer_callback(
    userdata: *mut c_void,
    _timerID: sys::timer::SDL_TimerID,
    _interval: u32,
) -> u32 {
    let callback_ptr = userdata as *mut TimerCallback;
    unsafe { (*callback_ptr)() }
}

#[cfg(not(target_os = "macos"))]
#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::timer::add_timer;

    #[test]
    fn test_timer_runs_multiple_times() {
        let _sdl_context = crate::sdl::init().unwrap();
        //let timer_subsystem = sdl_context.timer().unwrap();

        let local_num = Arc::new(Mutex::new(0));
        let timer_num = local_num.clone();

        let _timer = add_timer(
            20,
            Box::new(move || {
                let mut num = timer_num.lock().unwrap();
                if *num < 9 {
                    *num += 1;
                    20
                } else {
                    0
                }
            }),
        );

        std::thread::sleep(Duration::from_millis(250));
        let num = local_num.lock().unwrap();
        assert_eq!(*num, 9);
    }

    #[test]
    fn test_timer_runs_at_least_once() {
        let _sdl_context = crate::sdl::init().unwrap();
        //let timer_subsystem = sdl_context.timer().unwrap();

        let local_flag = Arc::new(Mutex::new(false));
        let timer_flag = local_flag.clone();

        let _timer = add_timer(
            20,
            Box::new(move || {
                let mut flag = timer_flag.lock().unwrap();
                *flag = true;
                0
            }),
        );

        std::thread::sleep(Duration::from_millis(50));
        let flag = local_flag.lock().unwrap();
        assert_eq!(*flag, true);
    }

    #[test]
    fn test_timer_can_be_recreated() {
        let sdl_context = crate::sdl::init().unwrap();
        //let timer_subsystem = sdl_context.timer().unwrap();

        let local_num = Arc::new(Mutex::new(0));
        let timer_num = local_num.clone();

        // Run the timer once and reclaim its closure.
        let timer_1 = add_timer(
            20,
            Box::new(move || {
                let mut num = timer_num.lock().unwrap();
                *num += 1;
                0
            }),
        );

        // Reclaim closure after timer runs.
        std::thread::sleep(Duration::from_millis(50));
        let closure = timer_1.into_inner();

        // Create a second timer and increment again.
        let _timer_2 = add_timer(20, closure);
        std::thread::sleep(Duration::from_millis(50));

        // Check that timer was incremented twice.
        let num = local_num.lock().unwrap();
        assert_eq!(*num, 2);
    }
}
