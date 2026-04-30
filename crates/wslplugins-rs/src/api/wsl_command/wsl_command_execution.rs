use crate::api::{errors::Result as ApiResult, wsl_command::wsl_child::WSLChild};
#[cfg(doc)]
use crate::{api::Error as ApiError, DistributionID};
use std::{io, net::TcpStream};
use thiserror::Error;
use windows_core::HRESULT;

const E_FAIL: HRESULT = HRESULT(0x8000_4005_u32 as i32);

pub type WSLCommandExecutionResult<T> = std::result::Result<T, WSLCommandExecutionError>;

fn hresult_from_io_error(error: &io::Error) -> HRESULT {
    error
        .raw_os_error()
        .map_or(E_FAIL, |code| HRESULT::from_win32(code as u32))
}

#[derive(Debug, Error)]
pub enum WSLCommandExecutionError {
    #[error("WSL Plugin API error: {0}")]
    Api(#[from] crate::api::Error),

    #[error("I/O error while splitting WSL command stream: {source}")]
    Split {
        source: io::Error,
        stream: TcpStream,
    },
}

impl WSLCommandExecutionError {
    #[must_use]
    #[inline]
    pub fn code(&self) -> HRESULT {
        match self {
            Self::Api(error) => error.code(),
            Self::Split { source, .. } => hresult_from_io_error(source),
        }
    }

    #[must_use]
    #[inline]
    pub fn into_stream(self) -> Option<TcpStream> {
        match self {
            Self::Split { stream, .. } => Some(stream),
            Self::Api(_) => None,
        }
    }
}

pub trait WSLCommandExecution {
    /// Executes the command via the underlying WSL Plugin API.
    ///
    /// On success, returns a [`TcpStream`] connected to the process' stdin/stdout.
    ///
    /// # Behavior
    ///
    /// - `argv[0]` is computed as:
    ///   - overridden `arg0` if set,
    ///   - otherwise the program path string.
    /// - The full argv passed to the API is:
    ///   `argv[0]` + all user-provided args.
    /// - The selected execution method depends on [`DistributionID`].
    ///
    /// # Errors
    ///
    /// Returns an [`ApiError`] if the underlying API call fails.
    fn execute_stream(&self) -> ApiResult<TcpStream>;

    /// Executes the command and splits the returned stream into stdin/stdout wrappers.
    ///
    /// # Errors
    ///
    /// Returns [`WSLCommandExecutionError::Api`] if the underlying API call fails, or
    /// [`WSLCommandExecutionError::Split`] if duplicating the stream handle fails.
    fn execute(&self) -> WSLCommandExecutionResult<WSLChild> {
        let stream = self.execute_stream()?;
        match stream.try_clone() {
            Ok(stdin_stream) => Ok(WSLChild::new(stdin_stream, stream)),
            Err(source) => Err(WSLCommandExecutionError::Split { source, stream }),
        }
    }
}
