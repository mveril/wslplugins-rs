use super::exit_code_parser::{parse_peeked_exit_code, PeekedExitCode};
use crate::api::wsl_command::{
    simple_child::SimpleChild, wsl_stdin::WSLStdin, wsl_stdout::WSLStdout,
};
use std::io::{Read, Write};
use std::net::TcpStream;

const ESC: u8 = 0x1b;

pub struct ChildWithExitCode {
    child: SimpleChild,
    exit_code: Option<u32>,
    pending_stdout: Vec<u8>,
}

impl ChildWithExitCode {
    pub(crate) fn new(stdin_stream: TcpStream, stdout_stream: TcpStream) -> Self {
        Self {
            child: SimpleChild::new(stdin_stream, stdout_stream),
            exit_code: None,
            pending_stdout: Vec::new(),
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

    fn consume_exit_code_sequence(&mut self) -> std::io::Result<SequenceState> {
        if self.pending_stdout.is_empty() {
            let mut first_byte = [0];
            let read_len = self.child.stdout_mut().read(&mut first_byte)?;
            if read_len == 0 {
                return Ok(SequenceState::Eof);
            }
            self.pending_stdout.push(first_byte[0]);
        }

        match parse_peeked_exit_code(&self.pending_stdout) {
            PeekedExitCode::Complete {
                exit_code,
                sequence_len,
            } => {
                debug_assert_eq!(sequence_len, self.pending_stdout.len());
                self.pending_stdout.clear();
                self.exit_code = Some(exit_code);
                Ok(SequenceState::Consumed)
            }
            PeekedExitCode::Incomplete => {
                let mut next_byte = [0];
                let read_len = self.child.stdout_mut().read(&mut next_byte)?;
                if read_len == 0 {
                    Ok(SequenceState::Ready)
                } else {
                    self.pending_stdout.push(next_byte[0]);
                    Ok(SequenceState::Pending)
                }
            }
            PeekedExitCode::NotExitCode => Ok(SequenceState::Ready),
        }
    }

    fn read_pending_stdout(&mut self, buf: &mut [u8]) -> usize {
        let read_len = self.pending_stdout.len().min(buf.len());
        buf[..read_len].copy_from_slice(&self.pending_stdout[..read_len]);
        self.pending_stdout.drain(..read_len);
        read_len
    }
}

enum SequenceState {
    Consumed,
    Pending,
    Ready,
    Eof,
}

impl Read for ChildWithExitCode {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            if self.pending_stdout.is_empty() {
                let mut peeked = [0; 1024];
                let peeked_len = self.child.stdout().peek(&mut peeked)?;
                if peeked_len == 0 {
                    return Ok(0);
                }

                if peeked[0] != ESC {
                    let read_len = peeked[..peeked_len]
                        .iter()
                        .position(|byte| *byte == ESC)
                        .unwrap_or(peeked_len)
                        .min(buf.len());

                    return self.child.stdout_mut().read(&mut buf[..read_len]);
                }
            }

            match self.consume_exit_code_sequence()? {
                SequenceState::Consumed | SequenceState::Pending => continue,
                SequenceState::Ready => return Ok(self.read_pending_stdout(buf)),
                SequenceState::Eof => return Ok(0),
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;

    fn tcp_pair() -> std::io::Result<(TcpStream, TcpStream)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let client = TcpStream::connect(listener.local_addr()?)?;
        let (server, _) = listener.accept()?;
        Ok((client, server))
    }

    fn child_and_writer() -> std::io::Result<(ChildWithExitCode, TcpStream)> {
        let (reader, writer) = tcp_pair()?;
        let child = ChildWithExitCode::new(reader.try_clone()?, reader);
        Ok((child, writer))
    }

    fn write_chunks(mut writer: TcpStream, chunks: &[&[u8]]) {
        for chunk in chunks {
            writer.write_all(chunk).unwrap();
            writer.flush().unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn consumes_fragmented_exit_code_sequence() {
        let (mut child, writer) = child_and_writer().unwrap();

        let writer = thread::spawn(move || {
            write_chunks(writer, &[b"\x1b]9;4;", b"123", b"\x07"]);
        });

        let mut output = Vec::new();
        child.read_to_end(&mut output).unwrap();
        writer.join().unwrap();

        assert_eq!(output, b"");
        assert_eq!(child.exit_code(), Some(123));
    }

    #[test]
    fn consumes_fragmented_exit_code_sequence_after_stdout() {
        let (mut child, writer) = child_and_writer().unwrap();

        let writer = thread::spawn(move || {
            write_chunks(writer, &[b"hello\n\x1b]9;4;", b"7", b"\x07"]);
        });

        let mut output = Vec::new();
        child.read_to_end(&mut output).unwrap();
        writer.join().unwrap();

        assert_eq!(output, b"hello\n");
        assert_eq!(child.exit_code(), Some(7));
    }

    #[test]
    fn returns_incomplete_exit_code_prefix_at_eof() {
        let (mut child, writer) = child_and_writer().unwrap();

        thread::spawn(move || {
            write_chunks(writer, &[b"\x1b]9;4;"]);
        })
        .join()
        .unwrap();

        let mut output = Vec::new();
        child.read_to_end(&mut output).unwrap();

        assert_eq!(output, b"\x1b]9;4;");
        assert_eq!(child.exit_code(), None);
    }
}
