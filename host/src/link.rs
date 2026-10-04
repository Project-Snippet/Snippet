use std::fmt;
use std::io::{self, Read, Write};
use std::time::Duration;

use serialport::SerialPort;
use snippet_protocol::{decode, encode, Request, Response, MAX_FRAME};

#[derive(Debug)]
pub enum LinkError {
    Serial(serialport::Error),
    Io(io::Error),
    Protocol(snippet_protocol::Error),
    /// The device sent more bytes than a frame can hold without ending it.
    ReplyTooLong,
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkError::Serial(e) => write!(f, "serial port error: {e}"),
            LinkError::Io(e) => write!(f, "i/o error: {e}"),
            LinkError::Protocol(e) => write!(f, "bad message from device: {e}"),
            LinkError::ReplyTooLong => write!(f, "device reply was too long"),
        }
    }
}

impl std::error::Error for LinkError {}

impl From<serialport::Error> for LinkError {
    fn from(e: serialport::Error) -> Self {
        LinkError::Serial(e)
    }
}

impl From<io::Error> for LinkError {
    fn from(e: io::Error) -> Self {
        LinkError::Io(e)
    }
}

impl From<snippet_protocol::Error> for LinkError {
    fn from(e: snippet_protocol::Error) -> Self {
        LinkError::Protocol(e)
    }
}

/// A connection to the device. `P` is anything we can read bytes from and
/// write bytes to: a real serial port, or a fake one in tests.
pub struct Link<P: Read + Write> {
    port: P,
}

impl Link<Box<dyn SerialPort>> {
    pub fn open(path: &str) -> Result<Self, LinkError> {
        // The baud rate is ignored by a USB CDC device, but the API requires one.
        let port = serialport::new(path, 115_200)
            .timeout(Duration::from_millis(500))
            .open()?;
        Ok(Link { port })
    }
}

impl<P: Read + Write> Link<P> {
    pub fn new(port: P) -> Self {
        Link { port }
    }

    /// Send one request and wait for the device's reply.
    pub fn send(&mut self, request: Request) -> Result<Response, LinkError> {
        let mut out = [0u8; MAX_FRAME];
        let frame = encode(&request, &mut out)?;
        self.port.write_all(frame)?;

        // Read one byte at a time until the 0x00 that ends the frame.
        let mut buf = [0u8; MAX_FRAME];
        let mut len = 0;
        loop {
            let mut byte = [0u8; 1];
            self.port.read_exact(&mut byte)?;
            if len == buf.len() {
                return Err(LinkError::ReplyTooLong);
            }
            buf[len] = byte[0];
            len += 1;
            if byte[0] == 0 {
                break;
            }
        }
        Ok(decode(&mut buf[..len])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snippet_protocol::device::Device;
    use snippet_protocol::state::State;
    use snippet_protocol::Time;
    use std::collections::VecDeque;

    /// A pretend serial port with a real `Device` on the other end.
    struct FakeBoard {
        device: Device,
        incoming: Vec<u8>,
        outgoing: VecDeque<u8>,
    }

    impl FakeBoard {
        fn link() -> Link<FakeBoard> {
            Link::new(FakeBoard {
                device: Device::new(),
                incoming: Vec::new(),
                outgoing: VecDeque::new(),
            })
        }
    }

    impl Write for FakeBoard {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            for &byte in data {
                self.incoming.push(byte);
                if byte == 0 {
                    let mut frame = std::mem::take(&mut self.incoming);
                    let request: Request = decode(&mut frame).unwrap();
                    let reply = self.device.handle(request);
                    let mut buf = [0u8; MAX_FRAME];
                    self.outgoing.extend(encode(&reply, &mut buf).unwrap().iter());
                }
            }
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Read for FakeBoard {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            match self.outgoing.pop_front() {
                Some(byte) => {
                    out[0] = byte;
                    Ok(1)
                }
                None => Err(io::ErrorKind::TimedOut.into()),
            }
        }
    }

    #[test]
    fn ping_gets_pong() {
        assert_eq!(FakeBoard::link().send(Request::Ping).unwrap(), Response::Pong);
    }

    #[test]
    fn time_set_over_the_link_reads_back_the_same() {
        let mut link = FakeBoard::link();
        let time = Time { hours: 14, minutes: 32, seconds: 5 };
        assert_eq!(link.send(Request::SetTime(time)).unwrap(), Response::Ack);
        assert_eq!(link.send(Request::GetTime).unwrap(), Response::Time(time));
    }

    #[test]
    fn states_step_over_the_link() {
        let mut link = FakeBoard::link();
        assert_eq!(link.send(Request::NextState).unwrap(), Response::Ack);
        assert_eq!(link.send(Request::GetState).unwrap(), Response::State(State::Clock));
    }

    /// A port that accepts writes but never answers.
    struct SilentBoard;

    impl Write for SilentBoard {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Read for SilentBoard {
        fn read(&mut self, _out: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::TimedOut.into())
        }
    }

    #[test]
    fn a_silent_device_is_an_error_not_a_hang() {
        let mut link = Link::new(SilentBoard);
        assert!(matches!(link.send(Request::Ping), Err(LinkError::Io(_))));
    }
}
