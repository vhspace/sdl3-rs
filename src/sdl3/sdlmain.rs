//! This implements the necessary traits from the sdl3-main crate so that the app_event
//! callback can take the Event type from this crate directly

use crate::event::Event;
use crate::sys::events::SDL_Event;
use sdl3_main::app::{PassEventMut, PassEventRef, PassEventVal};

impl PassEventVal for Event {
    #[inline(always)]
    fn pass_event_val<R>(event: &mut SDL_Event, f: impl FnOnce(Self) -> R) -> R {
        f(Event::from_ll(*event))
    }
}

impl PassEventRef for Event {
    #[inline(always)]
    fn pass_event_ref<R>(event: &mut SDL_Event, f: impl FnOnce(&Self) -> R) -> R {
        f(&Event::from_ll(*event))
    }
}

impl PassEventMut for Event {
    #[inline(always)]
    fn pass_event_mut<R>(event: &mut SDL_Event, f: impl FnOnce(&mut Self) -> R) -> R {
        f(&mut Event::from_ll(*event))
    }
}
