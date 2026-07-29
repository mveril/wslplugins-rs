//! Experimental safe wrappers for the WSL container (WSLC) API.

mod api;
mod container_information;
mod image_information;
mod object_id;
mod session_information;

pub use api::{
    WSLCApi, WSLCChildStderr, WSLCChildStdin, WSLCChildStdout, WSLCCommand, WSLCCommandError,
    WSLCCreateProcessError, WSLCExitStatus, WSLCOutput, WSLCProcess, WSLCProcessFd, WSLCSessionID,
    WSLCSpawnError,
};
pub use container_information::WSLCContainerInformation;
pub use image_information::WSLCImageInformation;
pub use object_id::WSLCObjectId;
pub use session_information::WSLCSessionInformation;
