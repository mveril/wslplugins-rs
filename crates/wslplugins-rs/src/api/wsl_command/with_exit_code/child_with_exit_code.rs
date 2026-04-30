use super::exit_code_parser::{
    parse_peeked_exit_code, PeekedExitCode, MAX_EXIT_CODE_SEQUENCE_LEN,
};
use crate::api::wsl_command::{
    simple_child::SimpleChild, wsl_stdin::WSLStdin, wsl_stdout::WSLStdout,
};
use std::io::{Read, Write};
use std::net::TcpStream;

const ESC: u8 = 0x1b;

pub struct ChildWithExitCode {
    child: SimpleChild,
    exit_code: Option<u32>,
}

impl ChildWithExitCode {
    pub(crate) fn new(stdin_stream: TcpStream, stdout_stream: TcpStream) -> Self {
        Self {
            child: SimpleChild::new(stdin_stream, stdout_stream),
            exit_code: None,
        }
    }

    #[must_use]
    pub const fn stdin(&self) -> &WSLStdin {
        self.child.stdin()
    }

    #[must_use]
    pub const fn stdin_mut(&mut self) -> &mut WSLStdin {
        self.child.stdin_mut()
    }

    #[must_use]
    pub const fn stdout(&self) -> &WSLStdout {
        self.child.stdout()
    }

    #[must_use]
    pub const fn stdout_mut(&mut self) -> &mut WSLStdout {
        self.child.stdout_mut()
    }

    #[must_use]
    pub const fn exit_code(&self) -> Option<u32> {
        self.exit_code
    }

    fn consume_exit_code_sequence(&mut self) -> std::io::Result<bool> {
        let mut peeked = [0; MAX_EXIT_CODE_SEQUENCE_LEN];
        let peeked_len = self.child.stdout().peek(&mut peeked)?;

        match parse_peeked_exit_code(&peeked[..peeked_len]) {
            PeekedExitCode::Complete {
                exit_code,
                sequence_len,
            } => {
                let mut discard = [0; MAX_EXIT_CODE_SEQUENCE_LEN];
                self.child
                    .stdout_mut()
                    .read_exact(&mut discard[..sequence_len])?;
                self.exit_code = Some(exit_code);
                Ok(true)
            }
            PeekedExitCode::Incomplete | PeekedExitCode::NotExitCode => Ok(false),
        }
    }
}

impl Read for ChildWithExitCode {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            if self.consume_exit_code_sequence()? {
                continue;
            }

            let mut peeked = [0; 1024];
            let peeked_len = self.child.stdout().peek(&mut peeked)?;
            if peeked_len == 0 {
                return Ok(0);
            }

            let read_len = if peeked[0] == ESC {
                1
            } else {
                peeked[..peeked_len]
                    .iter()
                    .position(|byte| *byte == ESC)
                    .unwrap_or(peeked_len)
                    .min(buf.len())
            };

            return self.child.stdout_mut().read(&mut buf[..read_len]);
        }
    }
}

impl Write for ChildWithExitCode {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.child.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.child.flush()
    }
}

impl From<ChildWithExitCode> for TcpStream {
    #[inline]
    fn from(value: ChildWithExitCode) -> Self {
        value.child.into()
    }
}

impl AsRef<WSLStdin> for ChildWithExitCode {
    #[inline]
    fn as_ref(&self) -> &WSLStdin {
        self.child.as_ref()
    }
}

impl AsMut<WSLStdin> for ChildWithExitCode {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdin {
        self.child.as_mut()
    }
}

impl AsRef<WSLStdout> for ChildWithExitCode {
    #[inline]
    fn as_ref(&self) -> &WSLStdout {
        self.child.as_ref()
    }
}

impl AsMut<WSLStdout> for ChildWithExitCode {
    #[inline]
    fn as_mut(&mut self) -> &mut WSLStdout {
        self.child.as_mut()
    }
}
