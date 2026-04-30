use std::io::prelude::*;
use std::mem::ManuallyDrop;
use std::net::TcpStream;

pub struct WSLStdin {
    pub(super) stream: TcpStream,
}

impl WSLStdin {
    pub(super) fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.shutdown(std::net::Shutdown::Write)
    }

    pub(super) fn into_stream_without_shutdown(self) -> TcpStream {
        let value = ManuallyDrop::new(self);
        // Safety: `value` is wrapped in `ManuallyDrop`, so moving out of `stream`
        // cannot leave a partially-dropped `WSLStdin`.
        unsafe { std::ptr::read(&value.stream) }
    }
}

impl Write for WSLStdin {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Drop for WSLStdin {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
