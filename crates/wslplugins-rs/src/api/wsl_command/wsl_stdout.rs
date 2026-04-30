use std::io::prelude::*;
use std::net::TcpStream;

pub struct WSLStdout {
    stream: TcpStream,
}

impl WSLStdout {
    pub(super) fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.shutdown(std::net::Shutdown::Read)
    }

    pub(super) fn peek(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.peek(buf)
    }
}

impl Read for WSLStdout {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Drop for WSLStdout {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
