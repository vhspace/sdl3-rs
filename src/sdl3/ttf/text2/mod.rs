pub mod engine;
pub mod text;

const NULL_STRING: *const std::ffi::c_char = &raw const *&0;

struct Destructured {
    ptr: *const std::ffi::c_char,
    len: usize
}

// SDL TTF treats a string of zero length as a null terminated string. This allows empty rust strings to be used by simply replacing them with a string consisting only of a null-terminator, thus satisfying the null-terminated requirement.
trait StrDestructure {
    fn destructure(&self) -> Destructured;
}
impl StrDestructure for &str {
    fn destructure(&self) ->Destructured {
        match self.len() {
            0 => Destructured { ptr: NULL_STRING, len: 0 },
            len => Destructured { ptr: self.as_bytes().as_ptr().cast(), len },
        }
    }
}

#[allow(unused_imports)]
pub use engine::{
    Type as EngineType,
    TextEngine,
    OwnedTextEngine
};
#[allow(unused_imports)]
pub use text::{
    Text,
    OwnedText
};