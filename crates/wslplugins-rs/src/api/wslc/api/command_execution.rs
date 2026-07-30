use super::{WSLCCommandError, WSLCExitStatus, WSLCOutput, WSLCProcess, WSLCSpawnError};

/// Executes a WSLC command or a reusable prepared WSLC command.
pub trait WSLCCommandExecution<'api> {
    /// Creates the process.
    ///
    /// # Errors
    ///
    /// Returns the Windows failure and the Linux `errno` reported by WSL when
    /// process creation fails.
    fn spawn(&self) -> std::result::Result<WSLCProcess<'api>, WSLCSpawnError>;

    /// Executes the command and collects standard output and standard error.
    ///
    /// # Errors
    ///
    /// Returns an error if process creation, stream reading, or waiting fails.
    #[inline]
    fn output(&self) -> std::result::Result<WSLCOutput, WSLCCommandError> {
        self.spawn()?.wait_with_output()
    }

    /// Executes the command, discards its output, and returns its exit status.
    ///
    /// # Errors
    ///
    /// Returns an error if process creation, stream draining, or waiting fails.
    #[inline]
    fn status(&self) -> std::result::Result<WSLCExitStatus, WSLCCommandError> {
        self.spawn()?.wait_with_status()
    }
}
