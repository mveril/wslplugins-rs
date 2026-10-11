//! Process and command wrappers for the native WSLC API.

mod command;
mod command_execution;
mod errors;
mod prepared_command;
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
pub use command_execution::WSLCCommandExecution;
pub use errors::{WSLCCommandError, WSLCCreateProcessError, WSLCSpawnError};
pub use prepared_command::PreparedWSLCCommand;
pub use process::{
    WSLCChildStderr, WSLCChildStdin, WSLCChildStdout, WSLCExitStatus, WSLCOutput, WSLCProcess,
};
pub use process_fd::WSLCProcessFd;
pub(crate) use process_fd::WSLCProcessStreams;
pub use session_id::WSLCSessionID;
pub use wslc_api::WSLCApi;
