use std::marker::PhantomData;

use crate::gpu::Buffer;



// a typed pseudo-reference to an object located in a `Buffer` on the GPU 
pub struct Ref<'a, T> {
    pub(crate) buf: &'a Buffer,
    pub(crate) offset: u32,
    pub(crate) marker: PhantomData<&'a T>,
}