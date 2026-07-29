use std::io;
use thiserror::Error;
use windows_core::Error as WinError;

/// Failure returned while creating a process in a WSLC session.
#[derive(Clone, Debug, Error)]
#[error("failed to create WSLC process (errno {errno}): {source}")]
pub struct WSLCCreateProcessError {
    source: WinError,
    errno: i32,
}

/// Failure returned while spawning a WSLC process.
#[derive(Clone, Debug, Error)]
pub enum WSLCSpawnError {
    /// The native create-process operation failed.
    #[error(transparent)]
    Create(#[from] WSLCCreateProcessError),

    /// A standard stream or exit event could not be obtained.
    #[error(transparent)]
    Api(#[from] crate::api::Error),
}

/// Failure returned by [`WSLCCommand::status`] or [`WSLCCommand::output`].
#[derive(Debug, Error)]
pub enum WSLCCommandError {
    /// The process could not be spawned.
    #[error(transparent)]
    Spawn(#[from] WSLCSpawnError),

    /// Process synchronization or status retrieval failed.
    #[error(transparent)]
    Api(#[from] crate::api::Error),

    /// Reading captured output failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl WSLCCreateProcessError {
    pub(super) const fn new(source: WinError, errno: i32) -> Self {
        Self { source, errno }
    }

    /// Returns the Linux error number reported by WSLC.
    #[must_use]
    #[inline]
    pub const fn errno(&self) -> i32 {
        self.errno
    }

    /// Returns the Windows API error.
    #[must_use]
    #[inline]
    pub const fn windows_error(&self) -> &WinError {
        &self.source
    }
}
