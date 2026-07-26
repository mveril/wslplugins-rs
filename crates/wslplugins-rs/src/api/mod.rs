//! # API Module
//!
//! This module provides the core functionality for interacting with the WSL plugin API.
//! It includes abstractions for API interactions, error handling, and utility functions.

mod api_v1;
pub mod errors;
mod wslc;
pub use wsl_command::WSLCommandExecution;

/// The `ApiV1` struct provides an interface to interact with version 1 of the WSL Plugin API.
///
/// It encapsulates low-level interactions with the API and exposes a safe and idiomatic Rust interface.
pub use api_v1::ApiV1;

/// The `Error` type represents errors that may occur while using the WSL Plugin API.
///
/// This type encapsulates various error scenarios, providing a consistent and ergonomic way
/// to handle failures.
pub use errors::Error;

/// A specialized `Result` type for operations that may fail in the context of the WSL Plugin API.
///
/// This alias simplifies the function signatures by standardizing error handling.
pub use errors::Result;
pub use wslc::{
    WSLCApi, WSLCCommand, WSLCCreateProcessError, WSLCProcess, WSLCProcessFd,
    WSLCProcessOwnedHandle, WSLCSessionID,
};

/// The `utils` module provides utility functions and helpers for working with the WSL Plugin API.
///
/// These utilities simplify common tasks, such as version checking or string manipulation.
pub mod utils;
mod wsl_command;
/// The `PreparedWSLCommand` struct represents a command that has been prepared for execution within the WSL environment and can be reused without re-encoding.
pub use wsl_command::PreparedWSLCommand;
/// The `WSLCommand` struct represents a command that can be executed within the WSL environment.
pub use wsl_command::WSLCommand;
