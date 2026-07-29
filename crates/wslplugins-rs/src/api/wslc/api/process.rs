use super::{WSLCApi, WSLCCommandError};
use crate::api::Result;
use std::ffi::c_void;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, OwnedHandle, RawHandle};
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use windows_core::{Error as WinError, HRESULT};
use windows_sys::Win32::Foundation::{WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};
const E_POINTER: HRESULT = HRESULT(0x8000_4003_u32.cast_signed());

/// An owned process created in a WSLC session.
///
/// This type follows the shape of [`std::process::Child`]. Its standard streams
/// are public optional fields so callers can borrow them or transfer ownership
/// with [`Option::take`].
///
/// The native process remains alive until both this value and every standard
/// stream taken from it have been dropped.
pub struct WSLCProcess<'api> {
    pub(super) inner: Arc<WSLCProcessInner<'api>>,
    /// A handle for writing to the child process's standard input.
    pub stdin: Option<WSLCChildStdin<'api>>,
    /// A handle for reading from the child process's standard output.
    pub stdout: Option<WSLCChildStdout<'api>>,
    /// A handle for reading from the child process's standard error.
    pub stderr: Option<WSLCChildStderr<'api>>,
    pub(super) exit_event: OwnedHandle,
    pub(super) exit_status: Option<WSLCExitStatus>,
}

pub(super) struct WSLCProcessInner<'api> {
    pub(super) api: WSLCApi<'api>,
    pub(super) handle: NonNull<c_void>,
}

// SAFETY: the opaque WSLC process object is only accessed through thread-safe
// WSL API operations. Its lifetime is reference-counted and it is released
// exactly once by `Drop`.
unsafe impl Send for WSLCProcessInner<'_> {}
// SAFETY: see the `Send` implementation. No Rust memory behind `handle` is
// accessed directly.
unsafe impl Sync for WSLCProcessInner<'_> {}

impl WSLCProcess<'_> {
    /// Waits for the process to finish and returns its exit status.
    ///
    /// # Errors
    ///
    /// Returns an error when waiting for the exit event or retrieving the exit
    /// code fails.
    #[inline]
    pub fn wait(&mut self) -> Result<WSLCExitStatus> {
        drop(self.stdin.take());
        if let Some(status) = self.exit_status {
            return Ok(status);
        }

        self.wait_for(INFINITE)?
            .ok_or_else(|| WinError::from(E_POINTER).into())
    }

    /// Returns the exit status if the process has finished.
    ///
    /// # Errors
    ///
    /// Returns an error when checking the exit event or retrieving the exit
    /// code fails.
    #[inline]
    pub fn try_wait(&mut self) -> Result<Option<WSLCExitStatus>> {
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }

        self.wait_for(0)
    }

    /// Waits up to `timeout` for the process to finish.
    ///
    /// Standard input is closed before waiting, matching [`Self::wait`].
    /// Returns `Ok(None)` if the timeout expires before the process exits.
    ///
    /// # Errors
    ///
    /// Returns an error when waiting for the exit event or retrieving the exit
    /// code fails.
    #[inline]
    pub fn wait_timeout(&mut self, timeout: Duration) -> Result<Option<WSLCExitStatus>> {
        drop(self.stdin.take());
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }

        self.wait_for(duration_to_milliseconds(timeout))
    }

    /// Waits for the process and collects standard output and standard error.
    ///
    /// Standard input is closed before waiting. Both output streams are read
    /// concurrently to avoid pipe-buffer deadlocks.
    ///
    /// # Errors
    ///
    /// Returns an error when reading a stream or waiting for the process fails.
    #[inline]
    pub fn wait_with_output(mut self) -> std::result::Result<WSLCOutput, WSLCCommandError> {
        drop(self.stdin.take());
        let stdout = self.stdout.take();
        let stderr = self.stderr.take();

        let (stdout, stderr) = std::thread::scope(|scope| {
            let stdout_thread = scope.spawn(move || read_stdout(stdout));
            let stderr = read_stderr(stderr);

            let stdout = stdout_thread
                .join()
                .map_err(|_| io::Error::other("WSLC stdout reader thread panicked"))??;
            Ok::<_, io::Error>((stdout, stderr?))
        })?;

        let status = self.wait()?;
        Ok(WSLCOutput {
            status,
            stdout,
            stderr,
        })
    }

    pub(super) fn wait_with_status(
        mut self,
    ) -> std::result::Result<WSLCExitStatus, WSLCCommandError> {
        drop(self.stdin.take());
        let stdout = self.stdout.take();
        let stderr = self.stderr.take();

        std::thread::scope(|scope| {
            let stdout_thread = scope.spawn(move || drain_stream(stdout));
            let stderr = drain_stream(stderr);

            stdout_thread
                .join()
                .map_err(|_| io::Error::other("WSLC stdout reader thread panicked"))??;
            stderr?;
            Ok::<_, io::Error>(())
        })?;

        self.wait().map_err(Into::into)
    }

    /// Borrows the event that is signaled when this process exits.
    #[must_use]
    #[inline]
    pub fn exit_event(&self) -> BorrowedHandle<'_> {
        self.exit_event.as_handle()
    }

    #[inline]
    fn wait_for(&mut self, timeout: u32) -> Result<Option<WSLCExitStatus>> {
        // SAFETY: `exit_event` is a live, owned synchronization handle.
        let wait_result = unsafe { WaitForSingleObject(self.exit_event.as_raw_handle(), timeout) };
        if wait_result == WAIT_TIMEOUT {
            return Ok(None);
        }
        if wait_result == WAIT_FAILED {
            return Err(WinError::from_hresult(HRESULT::from_thread()).into());
        }
        if wait_result != WAIT_OBJECT_0 {
            return Err(WinError::from(E_POINTER).into());
        }

        let status = WSLCExitStatus(self.inner.api.process_exit_code(self.inner.handle)?);
        self.exit_status = Some(status);
        Ok(Some(status))
    }
}

fn duration_to_milliseconds(duration: Duration) -> u32 {
    const MAX_FINITE_TIMEOUT: u128 = u32::MAX as u128 - 1;

    let milliseconds = duration.as_millis();
    let rounded_up = if duration.subsec_nanos().is_multiple_of(1_000_000) {
        milliseconds
    } else {
        milliseconds.saturating_add(1)
    };
    rounded_up.min(MAX_FINITE_TIMEOUT) as u32
}

impl fmt::Debug for WSLCProcess<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WSLCProcess")
            .field("handle", &self.inner.handle)
            .field("stdin", &self.stdin)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .field("exit_status", &self.exit_status)
            .finish_non_exhaustive()
    }
}

impl Drop for WSLCProcessInner<'_> {
    #[inline]
    fn drop(&mut self) {
        let raw = self.api.raw();
        // SAFETY: this is the final shared owner of the opaque process. All
        // stream handles retain an `Arc` to this object, so they have already
        // been closed before this call.
        unsafe {
            raw.WSLCReleaseProcess.unwrap_unchecked()(self.handle.as_ptr());
        }
    }
}

/// The exit status of a WSLC process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WSLCExitStatus(i32);

impl WSLCExitStatus {
    /// Returns whether the process exited successfully.
    #[must_use]
    #[inline]
    pub const fn success(self) -> bool {
        self.0 == 0
    }

    /// Returns the process exit code.
    #[must_use]
    #[inline]
    pub const fn code(self) -> i32 {
        self.0
    }
}

impl fmt::Display for WSLCExitStatus {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "exit code: {}", self.0)
    }
}

/// The collected output of a finished WSLC process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WSLCOutput {
    /// The process exit status.
    pub status: WSLCExitStatus,
    /// Data written to standard output.
    pub stdout: Vec<u8>,
    /// Data written to standard error.
    pub stderr: Vec<u8>,
}

/// A handle for writing to a WSLC child process's standard input.
pub struct WSLCChildStdin<'api> {
    file: File,
    _process: Arc<WSLCProcessInner<'api>>,
}

impl<'api> WSLCChildStdin<'api> {
    pub(super) fn new(handle: OwnedHandle, process: Arc<WSLCProcessInner<'api>>) -> Self {
        Self {
            file: File::from(handle),
            _process: process,
        }
    }
}

impl Write for WSLCChildStdin<'_> {
    #[inline]
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.file.write(buffer)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// A handle for reading from a WSLC child process's standard output.
pub struct WSLCChildStdout<'api> {
    file: File,
    _process: Arc<WSLCProcessInner<'api>>,
}

impl<'api> WSLCChildStdout<'api> {
    pub(super) fn new(handle: OwnedHandle, process: Arc<WSLCProcessInner<'api>>) -> Self {
        Self {
            file: File::from(handle),
            _process: process,
        }
    }
}

impl Read for WSLCChildStdout<'_> {
    #[inline]
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.file.read(buffer)
    }
}

/// A handle for reading from a WSLC child process's standard error.
pub struct WSLCChildStderr<'api> {
    file: File,
    _process: Arc<WSLCProcessInner<'api>>,
}

impl<'api> WSLCChildStderr<'api> {
    pub(super) fn new(handle: OwnedHandle, process: Arc<WSLCProcessInner<'api>>) -> Self {
        Self {
            file: File::from(handle),
            _process: process,
        }
    }
}

impl Read for WSLCChildStderr<'_> {
    #[inline]
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.file.read(buffer)
    }
}

macro_rules! impl_stream_handle {
    ($stream:ident) => {
        impl AsHandle for $stream<'_> {
            #[inline]
            fn as_handle(&self) -> BorrowedHandle<'_> {
                self.file.as_handle()
            }
        }

        impl AsRawHandle for $stream<'_> {
            #[inline]
            fn as_raw_handle(&self) -> RawHandle {
                self.file.as_raw_handle()
            }
        }

        impl fmt::Debug for $stream<'_> {
            #[inline]
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.file.fmt(formatter)
            }
        }
    };
}

impl_stream_handle!(WSLCChildStdin);
impl_stream_handle!(WSLCChildStdout);
impl_stream_handle!(WSLCChildStderr);

fn read_stdout(stdout: Option<WSLCChildStdout<'_>>) -> io::Result<Vec<u8>> {
    read_stream(stdout)
}

fn read_stderr(stderr: Option<WSLCChildStderr<'_>>) -> io::Result<Vec<u8>> {
    read_stream(stderr)
}

fn read_stream(mut stream: Option<impl Read>) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    if let Some(stream) = stream.as_mut() {
        stream.read_to_end(&mut output)?;
    }
    Ok(output)
}

fn drain_stream(mut stream: Option<impl Read>) -> io::Result<()> {
    if let Some(stream) = stream.as_mut() {
        io::copy(stream, &mut io::sink())?;
    }
    Ok(())
}
