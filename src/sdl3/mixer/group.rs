use std::marker::PhantomData;

use crate::{get_error, Error};

use super::device::Mixer;
use super::sys;
use super::track::Track;

/// A group for organizing tracks.
///
/// Groups allow you to receive post-mix callbacks for a subset of tracks,
/// which is useful for applying effects or level metering to specific
/// categories of sound (e.g., music vs. sound effects).
///
/// A group must not outlive its parent mixer (enforced by the lifetime
/// parameter).
pub struct Group<'mixer> {
    raw: *mut sys::MIX_Group,
    _marker: PhantomData<&'mixer Mixer>,
}

impl<'mixer> Group<'mixer> {
    pub(crate) fn from_raw(raw: *mut sys::MIX_Group) -> Self {
        Group {
            raw,
            _marker: PhantomData,
        }
    }

    /// Get a pointer to the underlying `MIX_Group`.
    #[inline]
    pub fn raw(&self) -> *mut sys::MIX_Group {
        self.raw
    }

    /// Assign a track to this group.
    ///
    /// A track can only belong to one group at a time. Assigning to a new group
    /// removes it from its previous group.
    #[doc(alias = "MIX_SetTrackGroup")]
    pub fn assign_track(&self, track: &Track) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_SetTrackGroup(track.raw(), self.raw) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Remove a track from this group (return it to the default group).
    #[doc(alias = "MIX_SetTrackGroup")]
    pub fn remove_track(&self, track: &Track) -> Result<(), Error> {
        let ok = unsafe { sys::MIX_SetTrackGroup(track.raw(), std::ptr::null_mut()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

impl Drop for Group<'_> {
    fn drop(&mut self) {
        unsafe { sys::MIX_DestroyGroup(self.raw) };
    }
}
