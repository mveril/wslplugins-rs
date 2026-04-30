use super::{wsl_stdin::WSLStdin, wsl_stdout::WSLStdout};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct SimpleChild {
    pub(crate) stdin: WSLStdin,
    pub(crate) stdout: WSLStdout,
}

impl SimpleChild {
    pub(crate) fn new(stdin_stream: TcpStream, stdout_stream: TcpStream) -> Self {
        Self {
            stdin: WSLStdin::new(stdin_stream),
            stdout: WSLStdout::new(stdout_stream),
        }
    }

    #[must_use]
    pub const fn stdin(&self) -> &WSLStdin {
        &self.stdin
    }

    #[must_use]
    pub const fn stdin_mut(&mut self) -> &mut WSLStdin {
        &mut self.stdin
    }

    #[must_use]
    pub const fn stdout(&self) -> &WSLStdout {
        &self.stdout
    }

    #[must_use]
    pub const fn stdout_mut(&mut self) -> &mut WSLStdout {
        &mut self.stdout
    }
}

impl Read for SimpleChild {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl Write for SimpleChild {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdin.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.stdin.flush()
    }
}

impl From<SimpleChild> for TcpStream {
    fn from(value: SimpleChild) -> Self {
        let stream = value.stdin.into_stream_without_shutdown();
        drop(value.stdout.into_stream_without_shutdown());
        stream
    }
}

impl AsRef<WSLStdin> for SimpleChild {
    #[inline]
    fn as_ref(&self) -> &WSLStdin {
        &self.stdin
    }
}

impl AsMut<WSLStdin> for SimpleChild {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdin {
        &mut self.stdin
    }
}

impl AsRef<WSLStdout> for SimpleChild {
    #[inline]
    fn as_ref(&self) -> &WSLStdout {
        &self.stdout
    }
}

impl AsMut<WSLStdout> for SimpleChild {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdout {
        &mut self.stdout
    }
}
