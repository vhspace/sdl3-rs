use crate::get_error;
use crate::Error;
use libc::c_void;
use std::ffi::CString;
use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use crate::sys;

/// A structure that provides an abstract interface to stream I/O.
pub struct IOStream<'a> {
    raw: NonNull<sys::iostream::SDL_IOStream>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> IOStream<'a> {
    pub fn raw(&self) -> *mut sys::iostream::SDL_IOStream {
        self.raw.as_ptr()
    }

    pub fn raw_non_null(&self) -> NonNull<sys::iostream::SDL_IOStream> {
        self.raw
    }

    pub unsafe fn from_ll<'b>(raw: *mut sys::iostream::SDL_IOStream) -> IOStream<'b> {
        IOStream {
            raw: NonNull::new_unchecked(raw),
            _marker: PhantomData,
        }
    }

    pub unsafe fn from_ll_or_error<'b>(
        raw: *mut sys::iostream::SDL_IOStream,
    ) -> Result<IOStream<'b>, Error> {
        match NonNull::new(raw) {
            Some(raw) => Ok(IOStream {
                raw,
                _marker: PhantomData,
            }),
            None => Err(get_error()),
        }
    }

    /// Creates an SDL file stream.
    #[doc(alias = "SDL_IOFromFile")]
    pub fn from_file<P: AsRef<Path>>(path: P, mode: &str) -> Result<IOStream<'static>, Error> {
        let path_c = CString::new(path.as_ref().to_str().unwrap()).unwrap();
        let mode_c = CString::new(mode).unwrap();

        unsafe {
            let raw = sys::iostream::SDL_IOFromFile(path_c.as_ptr(), mode_c.as_ptr());
            Self::from_ll_or_error(raw)
        }
    }

    /// Prepares a read-only memory buffer for use with `IOStream`.
    ///
    /// This method can only fail if the buffer size is zero.
    #[doc(alias = "SDL_IOFromConstMem")]
    pub fn from_bytes(buf: &'a [u8]) -> Result<IOStream<'a>, Error> {
        unsafe {
            let raw = sys::iostream::SDL_IOFromConstMem(buf.as_ptr() as *const c_void, buf.len());
            Self::from_ll_or_error(raw)
        }
    }

    /// Reads a `Read` object into a buffer and then passes it to `IOStream.from_bytes`.
    ///
    /// The buffer must be provided to this function and must live as long as the
    /// `IOStream`, but the `IOStream` does not take ownership of it.
    pub fn from_read<T>(r: &mut T, buffer: &'a mut Vec<u8>) -> Result<IOStream<'a>, Error>
    where
        T: io::Read + Sized,
    {
        match r.read_to_end(buffer) {
            Ok(_size) => IOStream::from_bytes(buffer),
            Err(ioerror) => {
                let msg = format!("IO error: {}", ioerror);
                Err(Error(msg))
            }
        }
    }

    /// Prepares a read-write memory buffer for use with `IOStream`.
    ///
    /// This method can only fail if the buffer size is zero.
    #[doc(alias = "SDL_IOFromMem")]
    pub fn from_bytes_mut(buf: &'a mut [u8]) -> Result<IOStream<'a>, Error> {
        unsafe {
            let raw = sys::iostream::SDL_IOFromMem(buf.as_mut_ptr() as *mut c_void, buf.len());
            Self::from_ll_or_error(raw)
        }
    }

    /// Gets the stream's total size in bytes.
    ///
    /// Returns `None` if the stream size can't be determined
    /// (either because it doesn't make sense for the stream type, or there was an error).
    pub fn len(&self) -> Option<usize> {
        let result = unsafe { sys::iostream::SDL_GetIOSize(self.raw()) };

        match result {
            -1 => None,
            v => Some(v as usize),
        }
    }

    /// Tells if the stream is empty.
    ///
    /// Returns `None` if the stream size can't be determined
    /// (either because it doesn't make sense for the stream type, or there was an error).
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|len| len == 0)
    }

    pub fn status(&self) -> IOStatus {
        match unsafe { sys::iostream::SDL_GetIOStatus(self.raw()) }.try_into() {
            Ok(status) => status,
            Err(()) => {
                panic!("SDL_GetIOStatus returned an invalid status");
            }
        }
    }
}

/// See [`SDL_IOStatus`](sys::iostream::SDL_IOStatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IOStatus {
    Ready,
    Error,
    EOF,
    NotReady,
    ReadOnly,
    WriteOnly,
}

impl TryFrom<sys::iostream::SDL_IOStatus> for IOStatus {
    type Error = ();

    fn try_from(value: sys::iostream::SDL_IOStatus) -> Result<Self, ()> {
        use sys::iostream::SDL_IOStatus;

        Ok(match value {
            SDL_IOStatus::READY => Self::Ready,
            SDL_IOStatus::ERROR => Self::Error,
            SDL_IOStatus::EOF => Self::EOF,
            SDL_IOStatus::NOT_READY => Self::NotReady,
            SDL_IOStatus::READONLY => Self::ReadOnly,
            SDL_IOStatus::WRITEONLY => Self::WriteOnly,
            _ => return Err(()),
        })
    }
}

impl Drop for IOStream<'_> {
    fn drop(&mut self) {
        let ret = unsafe { sys::iostream::SDL_CloseIO(self.raw()) };
        if !ret {
            panic!("{}", get_error());
        }
    }
}

impl io::Read for IOStream<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let out_len = buf.len();
        let ret =
            unsafe { sys::iostream::SDL_ReadIO(self.raw(), buf.as_ptr() as *mut c_void, out_len) };
        Ok(ret)
    }
}

impl io::Write for IOStream<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let in_len = buf.len();
        let ret = unsafe {
            sys::iostream::SDL_WriteIO(self.raw(), buf.as_ptr() as *const c_void, in_len)
        };
        if ret != in_len && self.status() == IOStatus::Error {
            Err(io::Error::other(get_error()))
        } else {
            Ok(ret)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let ret = unsafe { sys::iostream::SDL_FlushIO(self.raw()) };
        if ret {
            Ok(())
        } else {
            Err(io::Error::other(get_error()))
        }
    }
}

impl io::Seek for IOStream<'_> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            io::SeekFrom::Start(pos) => (sys::iostream::SDL_IO_SEEK_SET, pos as i64),
            io::SeekFrom::End(pos) => (sys::iostream::SDL_IO_SEEK_END, pos),
            io::SeekFrom::Current(pos) => (sys::iostream::SDL_IO_SEEK_CUR, pos),
        };
        let ret = unsafe { sys::iostream::SDL_SeekIO(self.raw(), offset, whence) };
        if ret == -1 {
            Err(io::Error::other(get_error()))
        } else {
            Ok(ret as u64)
        }
    }
}
