use crate::get_error;
use libc::c_char;
use libc::c_void;
use std::ffi::CString;
use std::io;
use std::marker::PhantomData;
use std::mem::transmute;
use std::path::Path;

use crate::sys;

/// A structure that provides an abstract interface to stream I/O.
pub struct IOStream<'a> {
    raw: *mut sys::iostream::SDL_IOStream,
    _marker: PhantomData<&'a ()>,
}

impl<'a> IOStream<'a> {
    pub unsafe fn raw(&self) -> *mut sys::iostream::SDL_IOStream {
        self.raw
    }

    pub unsafe fn from_ll<'b>(raw: *mut sys::iostream::SDL_IOStream) -> IOStream<'b> {
        IOStream {
            raw,
            _marker: PhantomData,
        }
    }

    /// Creates an SDL file stream.
    #[doc(alias = "SDL_IOFromFile")]
    pub fn from_file<P: AsRef<Path>>(path: P, mode: &str) -> Result<IOStream<'static>, String> {
        let raw = unsafe {
            let path_c = CString::new(path.as_ref().to_str().unwrap()).unwrap();
            let mode_c = CString::new(mode).unwrap();
            sys::iostream::SDL_IOFromFile(
                path_c.as_ptr() as *const c_char,
                mode_c.as_ptr() as *const c_char,
            )
        };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(IOStream {
                raw,
                _marker: PhantomData,
            })
        }
    }

    /// Prepares a read-only memory buffer for use with `IOStream`.
    ///
    /// This method can only fail if the buffer size is zero.
    #[doc(alias = "SDL_IOFromConstMem")]
    pub fn from_bytes(buf: &'a [u8]) -> Result<IOStream<'a>, String> {
        let raw =
            unsafe { sys::iostream::SDL_IOFromConstMem(buf.as_ptr() as *const c_void, buf.len()) };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(IOStream {
                raw,
                _marker: PhantomData,
            })
        }
    }

    /// Reads a `Read` object into a buffer and then passes it to `IOStream.from_bytes`.
    ///
    /// The buffer must be provided to this function and must live as long as the
    /// `IOStream`, but the `IOStream` does not take ownership of it.
    pub fn from_read<T>(r: &mut T, buffer: &'a mut Vec<u8>) -> Result<IOStream<'a>, String>
    where
        T: io::Read + Sized,
    {
        match r.read_to_end(buffer) {
            Ok(_size) => IOStream::from_bytes(buffer),
            Err(ioerror) => {
                let msg = format!("IO error: {}", ioerror);
                Err(msg)
            }
        }
    }

    /// Prepares a read-write memory buffer for use with `IOStream`.
    ///
    /// This method can only fail if the buffer size is zero.
    #[doc(alias = "SDL_IOFromMem")]
    pub fn from_bytes_mut(buf: &'a mut [u8]) -> Result<IOStream<'a>, String> {
        let raw = unsafe { sys::iostream::SDL_IOFromMem(buf.as_ptr() as *mut c_void, buf.len()) };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(IOStream {
                raw,
                _marker: PhantomData,
            })
        }
    }

    /// Gets the stream's total size in bytes.
    ///
    /// Returns `None` if the stream size can't be determined
    /// (either because it doesn't make sense for the stream type, or there was an error).
    pub fn len(&self) -> Option<usize> {
        let result = unsafe { sys::iostream::SDL_GetIOSize(self.raw) };

        match result {
            -1 => None,
            v => Some(v as usize),
        }
    }

    // Tells if the stream is empty
    pub fn is_empty(&self) -> bool {
        match self.len() {
            Some(s) => s == 0,
            None => true,
        }
    }
}

impl<'a> Drop for IOStream<'a> {
    fn drop(&mut self) {
        let ret = unsafe { sys::iostream::SDL_CloseIO(self.raw) };
        if !ret {
            panic!("{}", get_error());
        }
    }
}

impl<'a> io::Read for IOStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let out_len = buf.len();
        let ret =
            unsafe { sys::iostream::SDL_ReadIO(self.raw, buf.as_ptr() as *mut c_void, out_len) };
        Ok(ret as usize)
    }
}

impl<'a> io::Write for IOStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let in_len = buf.len();
        let ret =
            unsafe { sys::iostream::SDL_WriteIO(self.raw, buf.as_ptr() as *const c_void, in_len) };
        Ok(ret as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> io::Seek for IOStream<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            io::SeekFrom::Start(pos) => (sys::iostream::SDL_IO_SEEK_SET, pos as i64),
            io::SeekFrom::End(pos) => (sys::iostream::SDL_IO_SEEK_END, pos),
            io::SeekFrom::Current(pos) => (sys::iostream::SDL_IO_SEEK_CUR, pos),
        };
        let ret = unsafe { sys::iostream::SDL_SeekIO(self.raw, offset, transmute(whence)) };
        if ret == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as u64)
        }
    }
}
