use std::net::TcpStream;

use crate::api::errors::Result as ApiResult;
#[cfg(doc)]
use crate::{api::Error as ApiError, DistributionID};
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
    fn execute(&self) -> ApiResult<TcpStream>;
}
