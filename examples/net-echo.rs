//! Tiny TCP echo demo using the `net` feature.
//!
//! Spawns a server on 127.0.0.1:43215 in a background thread, connects a
//! client, sends a line, and prints whatever comes back.

use sdl3::net;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const PORT: u16 = 43215;

fn server_thread(ready: mpsc::Sender<()>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ctx = net::init()?;
    let mut server = net::Server::bind(None, PORT)?;
    let _ = ready.send(());

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut client = loop {
        if let Some(c) = server.accept()? {
            break c;
        }
        if Instant::now() > deadline {
            return Err("server: client never connected".into());
        }
        thread::sleep(Duration::from_millis(10));
    };

    let mut buf = [0u8; 256];
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let n = client.read(&mut buf)?;
        if n > 0 {
            client.write(&buf[..n])?;
            if client.wait_until_drained(1000) < 0 {
                return Err("server: failed to drain socket".into());
            }
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err("server: never received data".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let ctx = net::init()?;

    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = thread::spawn(move || server_thread(ready_tx));
    ready_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "server never came up")?;

    let mut addr = net::Address::resolve("127.0.0.1")?;
    addr.wait_until_resolved(-1)?;

    let mut client = net::StreamSocket::connect(&addr, PORT)?;
    client.wait_until_connected(-1)?;
    println!(
        "connected to {}",
        addr.to_string_lossy().unwrap_or_default()
    );

    client.write(b"hello, sdl3-net\n")?;
    if client.wait_until_drained(1000) < 0 {
        return Err("client: failed to drain socket".into());
    }

    let mut buf = [0u8; 256];
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let n = client.read(&mut buf)?;
        if n > 0 {
            print!("echo: {}", String::from_utf8_lossy(&buf[..n]));
            break;
        }
        if Instant::now() > deadline {
            return Err("client: never got echo".into());
        }
        thread::sleep(Duration::from_millis(10));
    }

    handle
        .join()
        .map_err(|_| "server panicked")?
        .map_err(|e| -> Box<dyn Error + Send + Sync> { e.to_string().into() })?;
    drop(ctx);
    Ok(())
}
