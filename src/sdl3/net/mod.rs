//!
//! Bindings for the `SDL3_net` extension library.
//!
//! SDL3_net is a thin cross-platform networking wrapper over BSD sockets /
//! WinSock. It provides reliable stream sockets (TCP) and unreliable datagram
//! sockets (UDP) along with asynchronous hostname resolution.
//!
//! Note that you need to build with the `net` feature for this module to be
//! enabled:
//!
//! ```bash
//! $ cargo build --features "net"
//! ```
//!
//! The upstream SDL3_net C library is still in prerelease, so this binding
//! tracks the `sdl3-net-sys` prerelease crate and the API may shift before the
//! first stable release.
//!
//! # Quick start
//!
//! ```no_run
//! use sdl3::net;
//!
//! let _ctx = net::init().unwrap();
//! let mut addr = net::Address::resolve("example.com").unwrap();
//! addr.wait_until_resolved(-1).unwrap();
//! let mut client = net::StreamSocket::connect(&addr, 80).unwrap();
//! client.wait_until_connected(-1).unwrap();
//! client.write(b"GET / HTTP/1.0\r\n\r\n").unwrap();
//! ```

use crate::{get_error, Error};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr;

pub use sdl3_net_sys as sys;

use sys::net::{
    NET_AcceptClient, NET_Address, NET_CompareAddresses, NET_CreateClient,
    NET_CreateDatagramSocket, NET_CreateServer, NET_Datagram, NET_DatagramSocket,
    NET_DestroyDatagram, NET_DestroyDatagramSocket, NET_DestroyServer, NET_DestroyStreamSocket,
    NET_FreeLocalAddresses, NET_GetAddressStatus, NET_GetAddressString, NET_GetConnectionStatus,
    NET_GetLocalAddresses, NET_GetStreamSocketAddress, NET_GetStreamSocketPendingWrites, NET_Init,
    NET_Quit, NET_ReadFromStreamSocket, NET_ReceiveDatagram, NET_RefAddress, NET_ResolveHostname,
    NET_SendDatagram, NET_Server, NET_SimulateAddressResolutionLoss,
    NET_SimulateDatagramPacketLoss, NET_SimulateStreamPacketLoss, NET_Status, NET_StreamSocket,
    NET_UnrefAddress, NET_Version, NET_WaitUntilConnected, NET_WaitUntilInputAvailable,
    NET_WaitUntilResolved, NET_WaitUntilStreamSocketDrained, NET_WriteToStreamSocket,
};

/// Tri-state status returned by asynchronous SDL3_net operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The operation completed successfully.
    Success,
    /// The operation is still in progress.
    Waiting,
    /// The operation completed with an error.
    Failure,
}

impl Status {
    fn from_raw(s: NET_Status) -> Self {
        match s {
            NET_Status::SUCCESS => Status::Success,
            NET_Status::FAILURE => Status::Failure,
            _ => Status::Waiting,
        }
    }

    fn into_result(self) -> Result<Status, Error> {
        match self {
            Status::Failure => Err(get_error()),
            other => Ok(other),
        }
    }
}

/// Context manager for `SDL3_net`. Initializes on construction and quits on drop.
///
/// `NET_Init`/`NET_Quit` are internally reference counted, so multiple contexts
/// can coexist. The library shuts down once the last context is dropped.
#[must_use]
pub struct NetContext {
    _marker: PhantomData<*mut ()>, // !Send + !Sync
}

impl NetContext {
    fn new() -> Result<Self, Error> {
        let ok = unsafe { NET_Init() };
        if !ok {
            return Err(get_error());
        }
        Ok(NetContext {
            _marker: PhantomData,
        })
    }
}

impl Clone for NetContext {
    fn clone(&self) -> Self {
        let ok = unsafe { NET_Init() };
        assert!(ok, "NET_Init failed during clone");
        NetContext {
            _marker: PhantomData,
        }
    }
}

impl Drop for NetContext {
    fn drop(&mut self) {
        unsafe { NET_Quit() };
    }
}

/// Initialize SDL3_net and return a context guard.
#[doc(alias = "NET_Init")]
pub fn init() -> Result<NetContext, Error> {
    NetContext::new()
}

/// Returns the linked SDL3_net library version (encoded as `SDL_VERSIONNUM`).
#[doc(alias = "NET_Version")]
pub fn linked_version() -> i32 {
    NET_Version()
}

/// Pretend a given percentage of hostname resolutions will fail. Intended for
/// testing. Pass 0 to disable.
#[doc(alias = "NET_SimulateAddressResolutionLoss")]
pub fn simulate_address_resolution_loss(percent_loss: i32) {
    unsafe { NET_SimulateAddressResolutionLoss(percent_loss as _) };
}

/// A network address (hostname/IP). Reference counted internally by SDL3_net.
#[doc(alias = "NET_Address")]
pub struct Address {
    raw: *mut NET_Address,
}

impl Address {
    /// Begin resolving a hostname (or IP address string). Returns immediately
    /// with an unresolved [`Address`]; call [`Address::wait_until_resolved`] or
    /// [`Address::status`] to check progress.
    #[doc(alias = "NET_ResolveHostname")]
    pub fn resolve(host: &str) -> Result<Address, Error> {
        let cstr = CString::new(host).map_err(|e| Error(e.to_string()))?;
        let raw = unsafe { NET_ResolveHostname(cstr.as_ptr()) };
        if raw.is_null() {
            return Err(get_error());
        }
        Ok(Address { raw })
    }

    /// Block until this address is resolved, fails, or the timeout elapses.
    /// Pass `-1` to wait forever, `0` for a non-blocking poll.
    #[doc(alias = "NET_WaitUntilResolved")]
    pub fn wait_until_resolved(&mut self, timeout_ms: i32) -> Result<Status, Error> {
        let s = unsafe { NET_WaitUntilResolved(self.raw, timeout_ms) };
        Status::from_raw(s).into_result()
    }

    /// Non-blocking check on resolution progress.
    #[doc(alias = "NET_GetAddressStatus")]
    pub fn status(&self) -> Result<Status, Error> {
        let s = unsafe { NET_GetAddressStatus(self.raw) };
        Status::from_raw(s).into_result()
    }

    /// Returns the address as a human-readable string (e.g. `"159.203.69.7"`)
    /// once resolution has completed successfully. Returns `None` while
    /// pending or on failure.
    #[doc(alias = "NET_GetAddressString")]
    pub fn to_string_lossy(&self) -> Option<String> {
        let raw = unsafe { NET_GetAddressString(self.raw) };
        if raw.is_null() {
            return None;
        }
        Some(
            unsafe { CStr::from_ptr(raw) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    /// Returns the raw `*mut NET_Address` pointer. Caller must not free it.
    pub fn raw(&self) -> *mut NET_Address {
        self.raw
    }

    /// Wrap an existing raw `*mut NET_Address`. The new [`Address`] takes
    /// ownership of one reference (does **not** call `NET_RefAddress`).
    ///
    /// # Safety
    /// The caller must own a reference to `raw` that they are transferring
    /// to this [`Address`].
    pub unsafe fn from_raw(raw: *mut NET_Address) -> Address {
        Address { raw }
    }
}

impl Clone for Address {
    fn clone(&self) -> Self {
        let raw = unsafe { NET_RefAddress(self.raw) };
        Address { raw }
    }
}

impl Drop for Address {
    fn drop(&mut self) {
        unsafe { NET_UnrefAddress(self.raw) };
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        unsafe { NET_CompareAddresses(self.raw, other.raw) == 0 }
    }
}

impl Eq for Address {}

/// Get all local addresses that can be bound to.
#[doc(alias = "NET_GetLocalAddresses")]
pub fn local_addresses() -> Result<Vec<Address>, Error> {
    let mut count: std::os::raw::c_int = 0;
    let raw = unsafe { NET_GetLocalAddresses(&mut count) };
    if raw.is_null() {
        return Err(get_error());
    }
    if count <= 0 {
        unsafe { NET_FreeLocalAddresses(raw) };
        return Ok(Vec::new());
    }
    // Each entry holds its own reference; take ownership by ref-ing then freeing the array.
    let slice = unsafe { std::slice::from_raw_parts(raw, count as usize) };
    let out = slice
        .iter()
        .map(|&p| Address {
            raw: unsafe { NET_RefAddress(p) },
        })
        .collect();
    unsafe { NET_FreeLocalAddresses(raw) };
    Ok(out)
}

/// A TCP-style reliable byte-stream socket.
#[doc(alias = "NET_StreamSocket")]
pub struct StreamSocket {
    raw: *mut NET_StreamSocket,
}

impl StreamSocket {
    /// Begin connecting as a client to `address:port`. The connection
    /// completes asynchronously; use [`StreamSocket::wait_until_connected`] or
    /// [`StreamSocket::connection_status`].
    #[doc(alias = "NET_CreateClient")]
    pub fn connect(address: &Address, port: u16) -> Result<StreamSocket, Error> {
        let raw = unsafe {
            NET_CreateClient(address.raw, port, sdl3_sys::properties::SDL_PropertiesID(0))
        };
        if raw.is_null() {
            return Err(get_error());
        }
        Ok(StreamSocket { raw })
    }

    /// Block until the socket has connected, failed, or `timeout_ms` elapses.
    #[doc(alias = "NET_WaitUntilConnected")]
    pub fn wait_until_connected(&mut self, timeout_ms: i32) -> Result<Status, Error> {
        let s = unsafe { NET_WaitUntilConnected(self.raw, timeout_ms) };
        Status::from_raw(s).into_result()
    }

    /// Non-blocking connection status check.
    #[doc(alias = "NET_GetConnectionStatus")]
    pub fn connection_status(&self) -> Result<Status, Error> {
        let s = unsafe { NET_GetConnectionStatus(self.raw) };
        Status::from_raw(s).into_result()
    }

    /// Returns the remote peer's address.
    #[doc(alias = "NET_GetStreamSocketAddress")]
    pub fn peer_address(&self) -> Result<Address, Error> {
        let raw = unsafe { NET_GetStreamSocketAddress(self.raw) };
        if raw.is_null() {
            return Err(get_error());
        }
        Ok(Address { raw })
    }

    /// Send bytes. Never blocks; data may be queued internally.
    #[doc(alias = "NET_WriteToStreamSocket")]
    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let ok =
            unsafe { NET_WriteToStreamSocket(self.raw, buf.as_ptr() as *const _, buf.len() as _) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Number of bytes still queued for transmission. Returns `-1` on
    /// unrecoverable socket failure.
    #[doc(alias = "NET_GetStreamSocketPendingWrites")]
    pub fn pending_writes(&self) -> i32 {
        unsafe { NET_GetStreamSocketPendingWrites(self.raw) as i32 }
    }

    /// Block until all queued bytes have been flushed (or `timeout_ms`).
    /// Returns the number of bytes still pending, or `-1` on failure.
    #[doc(alias = "NET_WaitUntilStreamSocketDrained")]
    pub fn wait_until_drained(&mut self, timeout_ms: i32) -> i32 {
        unsafe { NET_WaitUntilStreamSocketDrained(self.raw, timeout_ms) as i32 }
    }

    /// Read up to `buf.len()` bytes. Returns the number of bytes actually
    /// read (0 means no data is currently available; this never blocks).
    #[doc(alias = "NET_ReadFromStreamSocket")]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let n = unsafe {
            NET_ReadFromStreamSocket(self.raw, buf.as_mut_ptr() as *mut _, buf.len() as _)
        };
        if n < 0 {
            Err(get_error())
        } else {
            Ok(n as usize)
        }
    }

    /// Inject artificial packet loss for testing.
    #[doc(alias = "NET_SimulateStreamPacketLoss")]
    pub fn simulate_packet_loss(&mut self, percent_loss: i32) {
        unsafe { NET_SimulateStreamPacketLoss(self.raw, percent_loss as _) };
    }

    /// Returns the raw pointer.
    pub fn raw(&self) -> *mut NET_StreamSocket {
        self.raw
    }
}

impl Drop for StreamSocket {
    fn drop(&mut self) {
        unsafe { NET_DestroyStreamSocket(self.raw) };
    }
}

/// A listening server. Hand out [`StreamSocket`]s via [`Server::accept`].
#[doc(alias = "NET_Server")]
pub struct Server {
    raw: *mut NET_Server,
}

impl Server {
    /// Create a server listening on `port`. `addr` may be `None` to bind to
    /// all available interfaces (the typical case).
    #[doc(alias = "NET_CreateServer")]
    pub fn bind(addr: Option<&Address>, port: u16) -> Result<Server, Error> {
        let addr_raw = addr.map(|a| a.raw).unwrap_or(ptr::null_mut());
        let raw =
            unsafe { NET_CreateServer(addr_raw, port, sdl3_sys::properties::SDL_PropertiesID(0)) };
        if raw.is_null() {
            return Err(get_error());
        }
        Ok(Server { raw })
    }

    /// Non-blocking accept. Returns `Ok(None)` if no client is pending.
    #[doc(alias = "NET_AcceptClient")]
    pub fn accept(&mut self) -> Result<Option<StreamSocket>, Error> {
        let mut client: *mut NET_StreamSocket = ptr::null_mut();
        let ok = unsafe { NET_AcceptClient(self.raw, &mut client) };
        if !ok {
            return Err(get_error());
        }
        if client.is_null() {
            Ok(None)
        } else {
            Ok(Some(StreamSocket { raw: client }))
        }
    }

    /// Returns the raw pointer.
    pub fn raw(&self) -> *mut NET_Server {
        self.raw
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        unsafe { NET_DestroyServer(self.raw) };
    }
}

/// A UDP-style unreliable, packet-oriented socket.
#[doc(alias = "NET_DatagramSocket")]
pub struct DatagramSocket {
    raw: *mut NET_DatagramSocket,
}

/// A received datagram, paired with the sender's address and port.
#[doc(alias = "NET_Datagram")]
pub struct Datagram {
    raw: *mut NET_Datagram,
}

impl Datagram {
    /// Sender address as an owned, ref-counted handle. The returned
    /// [`Address`] is independent of this [`Datagram`] and remains valid
    /// after the datagram is dropped.
    pub fn address(&self) -> Address {
        let raw = unsafe { (*self.raw).addr };
        let reffed = unsafe { NET_RefAddress(raw) };
        Address { raw: reffed }
    }

    /// Sender port (host byte order).
    pub fn port(&self) -> u16 {
        unsafe { (*self.raw).port }
    }

    /// Payload bytes.
    pub fn payload(&self) -> &[u8] {
        unsafe {
            let p = (*self.raw).buf;
            let len = (*self.raw).buflen as usize;
            if p.is_null() || len == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(p, len)
            }
        }
    }
}

impl Drop for Datagram {
    fn drop(&mut self) {
        unsafe { NET_DestroyDatagram(self.raw) };
    }
}

impl DatagramSocket {
    /// Create and bind a datagram socket. `addr` may be `None` to bind to all
    /// interfaces. Pass `port = 0` to let the OS pick an ephemeral port.
    #[doc(alias = "NET_CreateDatagramSocket")]
    pub fn bind(addr: Option<&Address>, port: u16) -> Result<DatagramSocket, Error> {
        let addr_raw = addr.map(|a| a.raw).unwrap_or(ptr::null_mut());
        let raw = unsafe {
            NET_CreateDatagramSocket(addr_raw, port, sdl3_sys::properties::SDL_PropertiesID(0))
        };
        if raw.is_null() {
            return Err(get_error());
        }
        Ok(DatagramSocket { raw })
    }

    /// Send a packet to `address:port`.
    #[doc(alias = "NET_SendDatagram")]
    pub fn send(&mut self, address: &Address, port: u16, buf: &[u8]) -> Result<(), Error> {
        let ok = unsafe {
            NET_SendDatagram(
                self.raw,
                address.raw,
                port,
                buf.as_ptr() as *const _,
                buf.len() as _,
            )
        };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// Non-blocking receive. Returns `Ok(None)` when nothing is pending.
    #[doc(alias = "NET_ReceiveDatagram")]
    pub fn recv(&mut self) -> Result<Option<Datagram>, Error> {
        let mut dgram: *mut NET_Datagram = ptr::null_mut();
        let ok = unsafe { NET_ReceiveDatagram(self.raw, &mut dgram) };
        if !ok {
            return Err(get_error());
        }
        if dgram.is_null() {
            Ok(None)
        } else {
            Ok(Some(Datagram { raw: dgram }))
        }
    }

    /// Inject artificial packet loss for testing.
    #[doc(alias = "NET_SimulateDatagramPacketLoss")]
    pub fn simulate_packet_loss(&mut self, percent_loss: i32) {
        unsafe { NET_SimulateDatagramPacketLoss(self.raw, percent_loss as _) };
    }

    /// Returns the raw pointer.
    pub fn raw(&self) -> *mut NET_DatagramSocket {
        self.raw
    }
}

impl Drop for DatagramSocket {
    fn drop(&mut self) {
        unsafe { NET_DestroyDatagramSocket(self.raw) };
    }
}

/// Something that can be passed to [`wait_until_input_available`].
///
/// # Safety
/// Implementors must return a pointer to a live SDL3_net object that
/// `NET_WaitUntilInputAvailable` recognizes (currently `NET_Server`,
/// `NET_StreamSocket`, or `NET_DatagramSocket`). Returning anything else,
/// including dangling or arbitrary pointers, will trigger undefined
/// behavior inside the FFI call.
pub unsafe trait Waitable {
    /// Pointer cast to the void pointer SDL3_net expects.
    fn as_void_ptr(&self) -> *mut std::ffi::c_void;
}

unsafe impl Waitable for StreamSocket {
    fn as_void_ptr(&self) -> *mut std::ffi::c_void {
        self.raw as *mut _
    }
}

unsafe impl Waitable for Server {
    fn as_void_ptr(&self) -> *mut std::ffi::c_void {
        self.raw as *mut _
    }
}

unsafe impl Waitable for DatagramSocket {
    fn as_void_ptr(&self) -> *mut std::ffi::c_void {
        self.raw as *mut _
    }
}

/// Block until one of the given sockets has input available, or `timeout_ms`
/// elapses. Returns the number of sockets reporting input.
#[doc(alias = "NET_WaitUntilInputAvailable")]
pub fn wait_until_input_available(
    sockets: &[&dyn Waitable],
    timeout_ms: i32,
) -> Result<i32, Error> {
    let mut ptrs: Vec<*mut std::ffi::c_void> = sockets.iter().map(|s| s.as_void_ptr()).collect();
    let n = unsafe { NET_WaitUntilInputAvailable(ptrs.as_mut_ptr(), ptrs.len() as _, timeout_ms) };
    if n < 0 {
        Err(get_error())
    } else {
        Ok(n as i32)
    }
}
