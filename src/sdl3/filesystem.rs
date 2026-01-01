use libc::{c_char, c_void};
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sys::filesystem::SDL_PathInfo;

use crate::get_error;
use crate::sys;
use crate::Error;

#[derive(Debug, Clone)]
pub enum FileSystemError {
    InvalidPathError(PathBuf),
    NulError(NulError),
    SdlError(Error),
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::FileSystemError::*;

        match *self {
            InvalidPathError(ref path) => write!(f, "Invalid path: {}", path.display()),
            NulError(ref e) => write!(f, "Nul: {e}"),
            SdlError(ref e) => write!(f, "SDL error: {e}"),
        }
    }
}

impl error::Error for FileSystemError {
    fn description(&self) -> &str {
        use self::FileSystemError::*;

        match *self {
            InvalidPathError(_) => "invalid path",
            NulError(_) => "nul",
            SdlError(ref e) => &e.0,
        }
    }
}

/// Turn a AsRef<Path> into a CString so it can be passed to C
macro_rules! path_cstring {
    ($pathref:ident) => {
        let Some($pathref) = $pathref.as_ref().to_str() else {
            return Err(FileSystemError::InvalidPathError(
                $pathref.as_ref().to_owned(),
            ));
        };

        let Ok($pathref) = CString::new($pathref) else {
            return Err(FileSystemError::InvalidPathError(PathBuf::from($pathref)));
        };
    };
}

// Turn a CString into a Path for ease of use
macro_rules! cstring_path {
    ($path:ident, $error:expr) => {
        let Ok($path) = CStr::from_ptr($path).to_str() else {
            $error
        };
        let $path = Path::new($path);
    };
}

#[doc(alias = "SDL_CopyFile")]
pub fn copy_file(
    old_path: impl AsRef<Path>,
    new_path: impl AsRef<Path>,
) -> Result<(), FileSystemError> {
    path_cstring!(old_path);
    path_cstring!(new_path);
    unsafe {
        if !sys::filesystem::SDL_CopyFile(old_path.as_ptr(), new_path.as_ptr()) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }
    Ok(())
}

#[doc(alias = "SDL_CreateDirectory")]
pub fn create_directory(path: impl AsRef<Path>) -> Result<(), FileSystemError> {
    path_cstring!(path);
    unsafe {
        if !sys::filesystem::SDL_CreateDirectory(path.as_ptr()) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }
    Ok(())
}

pub use sys::filesystem::SDL_EnumerationResult as EnumerationResult;

pub type EnumerateCallback = fn(&Path, &Path) -> EnumerationResult;

unsafe extern "C" fn c_enumerate_directory(
    userdata: *mut c_void,
    dirname: *const c_char,
    fname: *const c_char,
) -> EnumerationResult {
    let callback: EnumerateCallback = std::mem::transmute(userdata);

    cstring_path!(dirname, return EnumerationResult::FAILURE);
    cstring_path!(fname, return EnumerationResult::FAILURE);

    callback(dirname, fname)
}

#[doc(alias = "SDL_EnumerateDirectory")]
pub fn enumerate_directory(
    path: impl AsRef<Path>,
    callback: EnumerateCallback,
) -> Result<(), FileSystemError> {
    path_cstring!(path);
    unsafe {
        if !sys::filesystem::SDL_EnumerateDirectory(
            path.as_ptr(),
            Some(c_enumerate_directory),
            callback as *mut c_void,
        ) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }
    Ok(())
}

#[doc(alias = "SDL_GetBasePath")]
pub fn get_base_path() -> Result<&'static Path, FileSystemError> {
    unsafe {
        let path = sys::filesystem::SDL_GetBasePath();
        cstring_path!(path, return Err(FileSystemError::SdlError(get_error())));
        Ok(path)
    }
}

#[doc(alias = "SDL_GetCurrentDirectory")]
pub fn get_current_directory() -> Result<PathBuf, FileSystemError> {
    unsafe {
        let path_ptr = sys::filesystem::SDL_GetCurrentDirectory();
        if path_ptr.is_null() {
            return Err(FileSystemError::SdlError(get_error()));
        }

        let path = match CStr::from_ptr(path_ptr).to_str() {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                sys::stdinc::SDL_free(path_ptr as *mut c_void);
                return Err(FileSystemError::SdlError(get_error()));
            }
        };

        sys::stdinc::SDL_free(path_ptr as *mut c_void);
        Ok(path)
    }
}

pub use sys::filesystem::SDL_PathType as PathType;

pub struct PathInfo {
    internal: SDL_PathInfo,
}

impl PathInfo {
    fn path_type(&self) -> PathType {
        self.internal.r#type as PathType
    }

    fn size(&self) -> usize {
        self.internal.size as usize
    }

    fn create_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.internal.create_time as u64)
    }

    fn modify_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.internal.modify_time as u64)
    }

    fn access_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.internal.access_time as u64)
    }
}

impl fmt::Debug for PathInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathInfo")
            .field(
                "path_type",
                match self.path_type() {
                    PathType::DIRECTORY => &"Directory",
                    PathType::FILE => &"File",
                    PathType::NONE => &"None",
                    _ => &"Other",
                },
            )
            .field("size", &self.size())
            .field("create_time", &self.create_time())
            .field("modify_time", &self.modify_time())
            .field("access_time", &self.access_time())
            .finish()
    }
}

#[doc(alias = "SDL_GetPathInfo")]
pub fn get_path_info(path: impl AsRef<Path>) -> Result<PathInfo, FileSystemError> {
    let mut info = SDL_PathInfo {
        r#type: PathType::NONE,
        size: 0,
        create_time: 0,
        modify_time: 0,
        access_time: 0,
    };
    path_cstring!(path);

    unsafe {
        if !sys::filesystem::SDL_GetPathInfo(path.as_ptr(), &mut info as *mut SDL_PathInfo) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }

    Ok(PathInfo { internal: info })
}

#[derive(Debug, Clone)]
pub enum PrefPathError {
    InvalidOrganizationName(NulError),
    InvalidApplicationName(NulError),
    SdlError(Error),
}

impl fmt::Display for PrefPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PrefPathError::*;

        match *self {
            InvalidOrganizationName(ref e) => write!(f, "Invalid organization name: {e}"),
            InvalidApplicationName(ref e) => write!(f, "Invalid application name: {e}"),
            SdlError(ref e) => write!(f, "SDL error: {e}"),
        }
    }
}

impl error::Error for PrefPathError {
    fn description(&self) -> &str {
        use self::PrefPathError::*;

        match *self {
            InvalidOrganizationName(_) => "invalid organization name",
            InvalidApplicationName(_) => "invalid application name",
            SdlError(ref e) => &e.0,
        }
    }
}

/// Return the preferred directory for the application to write files on this
/// system, based on the given organization and application name.
#[doc(alias = "SDL_GetPrefPath")]
pub fn get_pref_path(org_name: &str, app_name: &str) -> Result<PathBuf, PrefPathError> {
    let org = match CString::new(org_name) {
        Ok(s) => s,
        Err(err) => return Err(PrefPathError::InvalidOrganizationName(err)),
    };
    let app = match CString::new(app_name) {
        Ok(s) => s,
        Err(err) => return Err(PrefPathError::InvalidApplicationName(err)),
    };

    let path = unsafe {
        let buf = sys::filesystem::SDL_GetPrefPath(
            org.as_ptr() as *const c_char,
            app.as_ptr() as *const c_char,
        );
        let path = PathBuf::from(CStr::from_ptr(buf).to_str().unwrap());
        sys::stdinc::SDL_free(buf as *mut c_void);
        path
    };

    if path.as_os_str().is_empty() {
        Err(PrefPathError::SdlError(get_error()))
    } else {
        Ok(path)
    }
}

pub use sys::filesystem::SDL_Folder as Folder;

#[doc(alias = "SDL_GetUserFolder")]
pub fn get_user_folder(folder: Folder) -> Result<&'static Path, FileSystemError> {
    unsafe {
        let path = sys::filesystem::SDL_GetUserFolder(folder);
        cstring_path!(path, return Err(FileSystemError::SdlError(get_error())));
        Ok(path)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct GlobFlags: u32 {
        const NONE = 0;
        const CASEINSENSITIVE = sys::filesystem::SDL_GLOB_CASEINSENSITIVE.0 as u32;
    }
}

impl From<GlobFlags> for sys::filesystem::SDL_GlobFlags {
    fn from(flags: GlobFlags) -> Self {
        sys::filesystem::SDL_GlobFlags(flags.bits() as sys::stdinc::Uint32)
    }
}

impl From<sys::filesystem::SDL_GlobFlags> for GlobFlags {
    fn from(flags: sys::filesystem::SDL_GlobFlags) -> Self {
        GlobFlags::from_bits_truncate(flags.0)
    }
}

pub struct GlobResultsIter<'a> {
    results: &'a GlobResults<'a>,
    index: isize,
}

impl<'a> Iterator for GlobResultsIter<'a> {
    type Item = &'a Path;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.results.get(self.index);
        self.index += 1;
        current
    }
}

pub struct GlobResults<'a> {
    internal: *mut *mut c_char,
    count: isize,
    phantom: PhantomData<&'a *mut *mut c_char>,
}

impl GlobResults<'_> {
    fn new(internal: *mut *mut c_char, count: isize) -> Self {
        Self {
            internal,
            count,
            phantom: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.count as usize
    }

    fn get<I>(&self, index: I) -> Option<&Path>
    where
        I: Into<isize>,
    {
        let index = index.into();
        if index >= self.count {
            return None;
        }
        unsafe {
            let path = *self.internal.offset(index);
            cstring_path!(path, return None);
            Some(path)
        }
    }
}

impl<'a> IntoIterator for &'a GlobResults<'a> {
    type Item = &'a Path;
    type IntoIter = GlobResultsIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            results: self,
            index: 0,
        }
    }
}

impl Drop for GlobResults<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::stdinc::SDL_free(self.internal as *mut c_void);
        }
    }
}

#[doc(alias = "SDL_GlobDirectory")]
pub fn glob_directory(
    path: impl AsRef<Path>,
    pattern: Option<&str>,
    flags: GlobFlags,
) -> Result<GlobResults, FileSystemError> {
    path_cstring!(path);
    let pattern = match pattern {
        Some(pattern) => match CString::new(pattern) {
            Ok(pattern) => Some(pattern),
            Err(error) => return Err(FileSystemError::NulError(error)),
        },
        None => None,
    };
    let pattern_ptr = pattern.as_ref().map_or(ptr::null(), |pat| pat.as_ptr());
    let mut count = 0;

    let results = unsafe {
        let sys_flags: sys::filesystem::SDL_GlobFlags = flags.into();
        let paths = sys::filesystem::SDL_GlobDirectory(
            path.as_ptr(),
            pattern_ptr,
            sys_flags,
            &mut count as *mut i32,
        );
        if paths.is_null() {
            return Err(FileSystemError::SdlError(get_error()));
        }
        GlobResults::new(paths, count as isize)
    };
    Ok(results)
}

#[doc(alias = "SDL_RemovePath")]
pub fn remove_path(path: impl AsRef<Path>) -> Result<(), FileSystemError> {
    path_cstring!(path);
    unsafe {
        if !sys::filesystem::SDL_RemovePath(path.as_ptr()) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }
    Ok(())
}

#[doc(alias = "SDL_RenamePath")]
pub fn rename_path(
    old_path: impl AsRef<Path>,
    new_path: impl AsRef<Path>,
) -> Result<(), FileSystemError> {
    path_cstring!(old_path);
    path_cstring!(new_path);

    unsafe {
        if !sys::filesystem::SDL_RenamePath(old_path.as_ptr(), new_path.as_ptr()) {
            return Err(FileSystemError::SdlError(get_error()));
        }
    }

    Ok(())
}
