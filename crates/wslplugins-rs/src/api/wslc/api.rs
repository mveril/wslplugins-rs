//! Process and command wrappers for the native WSLC API.

mod command;
mod errors;
mod process;
mod process_fd;
mod session_id;
#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    reason = "Test synchronization and FFI assertions"
)]
mod tests;
mod wslc_api;

pub use command::WSLCCommand;
pub use errors::{WSLCCommandError, WSLCCreateProcessError, WSLCSpawnError};
pub use process::{
    WSLCChildStderr, WSLCChildStdin, WSLCChildStdout, WSLCExitStatus, WSLCOutput, WSLCProcess,
};
pub use process_fd::WSLCProcessFd;
pub use session_id::WSLCSessionID;
pub use wslc_api::WSLCApi;
