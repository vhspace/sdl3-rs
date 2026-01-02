/*!
Querying SDL Version
 */

use std::fmt;

use crate::sys;

/// A structure that contains information about the version of SDL in use.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Version {
    /// major version
    pub major: u8,
    /// minor version
    pub minor: u8,
    /// update version (patchlevel)
    pub patch: u8,
}

impl Version {
    /// Convert a raw sdl version number to Version.
    pub fn from_ll(v: i32) -> Version {
        // pub const SDL_VERSION: i32 = _; // 3_001_003i32
        Version {
            major: (v / 1_000_000) as u8,
            minor: (v % 1_000_000 / 1_000) as u8,
            patch: (v % 1_000) as u8,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Get the version of SDL that is linked against your program.
#[doc(alias = "SDL_GetVersion")]
pub fn version() -> Version {
    let version = sys::version::SDL_GetVersion();
    Version::from_ll(version)
}
