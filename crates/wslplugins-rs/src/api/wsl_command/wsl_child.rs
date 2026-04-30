use super::{
    simple_child::SimpleChild, with_exit_code::ChildWithExitCode, wsl_stdin::WSLStdin,
    wsl_stdout::WSLStdout,
};
use std::io::{Read, Write};
use std::net::TcpStream;

pub enum WSLChild {
    Simple(SimpleChild),
    WithExitCode(ChildWithExitCode),
}

impl WSLChild {
    pub(crate) fn new(stdin_stream: TcpStream, stdout_stream: TcpStream) -> Self {
        Self::Simple(SimpleChild::new(stdin_stream, stdout_stream))
    }

    pub(crate) fn new_with_exit_code(stdin_stream: TcpStream, stdout_stream: TcpStream) -> Self {
        Self::WithExitCode(ChildWithExitCode::new(stdin_stream, stdout_stream))
    }

    #[must_use]
    pub const fn stdin(&self) -> &WSLStdin {
        match self {
            Self::Simple(child) => child.stdin(),
            Self::WithExitCode(child) => child.stdin(),
        }
    }

    #[must_use]
    pub const fn stdin_mut(&mut self) -> &mut WSLStdin {
        match self {
            Self::Simple(child) => child.stdin_mut(),
            Self::WithExitCode(child) => child.stdin_mut(),
        }
    }

    #[must_use]
    pub const fn stdout(&self) -> &WSLStdout {
        match self {
            Self::Simple(child) => child.stdout(),
            Self::WithExitCode(child) => child.stdout(),
        }
    }

    #[must_use]
    pub const fn stdout_mut(&mut self) -> &mut WSLStdout {
        match self {
            Self::Simple(child) => child.stdout_mut(),
            Self::WithExitCode(child) => child.stdout_mut(),
        }
    }

    #[must_use]
    pub const fn exit_code(&self) -> Option<u32> {
        match self {
            Self::Simple(_) => None,
            Self::WithExitCode(child) => child.exit_code(),
        }
    }
}

impl Read for WSLChild {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Simple(child) => child.read(buf),
            Self::WithExitCode(child) => child.read(buf),
        }
    }
}

impl Write for WSLChild {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Simple(child) => child.write(buf),
            Self::WithExitCode(child) => child.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Simple(child) => child.flush(),
            Self::WithExitCode(child) => child.flush(),
        }
    }
}

impl From<WSLChild> for TcpStream {
    #[inline]
    fn from(value: WSLChild) -> Self {
        match value {
            WSLChild::Simple(child) => child.into(),
            WSLChild::WithExitCode(child) => child.into(),
        }
    }
}

impl AsRef<WSLStdin> for WSLChild {
    #[inline]
    fn as_ref(&self) -> &WSLStdin {
        self.stdin()
    }
}

impl AsMut<WSLStdin> for WSLChild {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdin {
        self.stdin_mut()
    }
}

impl AsRef<WSLStdout> for WSLChild {
    #[inline]
    fn as_ref(&self) -> &WSLStdout {
        self.stdout()
    }
}

impl AsMut<WSLStdout> for WSLChild {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdout {
        self.stdout_mut()
    }
}
