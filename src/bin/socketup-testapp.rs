use std::io;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let ports = [8080, 9901];
    let mut listeners = Vec::new();

    for port in ports {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        listener.set_nonblocking(true)?;
        listeners.push(listener);
    }

    loop {
        for listener in &listeners {
            match listener.accept() {
                Ok((_stream, _addr)) => {}
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                Err(err) => return Err(err),
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
}
