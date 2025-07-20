//! # Error Handling for WSL Plugins
//!
//! This module defines a unified error type for handling errors in WSL plugins, combining
//! both custom plugin-specific errors and Windows system errors. It provides seamless
//! interoperability with Windows APIs using `HRESULT`.

use thiserror::Error;
pub mod require_update_error;
pub use require_update_error::Error as RequireUpdateError;
use windows_core::{Error as WinError, HRESULT};
use wslpluginapi_sys::WSL_E_PLUGIN_REQUIRES_UPDATE;

/// A comprehensive error type for WSL plugins.
///
/// This enum encapsulates two main error categories:
/// - Plugin-specific errors (`RequireUpdateError`).
/// - Errors originating from Windows APIs (`WinError`).
#[derive(Debug, Error)]
pub enum Error {
    /// Indicates that the current WSL version does not meet the required version.
    #[error("Require Update Error")]
    RequiresUpdate(#[from] RequireUpdateError),

    /// Represents an error from Windows APIs.
    #[error("Windows Error: {0}")]
    WinError(#[from] WinError),
}

impl From<Error> for HRESULT {
    /// Converts the `Error` enum into an `HRESULT` code.
    ///
    /// - Maps `Error::RequiresUpdate` to `WSL_E_PLUGIN_REQUIRES_UPDATE`.
    /// - Maps `Error::WinError` to its corresponding `HRESULT` code.
    ///
    /// # Returns
    /// An `HRESULT` code representing the error.
    fn from(value: Error) -> Self {
        match value {
            Error::RequiresUpdate(err) => err.into(),
            Error::WinError(err) => err.code(),
        }
    }
}

impl From<Error> for WinError {
    /// Converts the `Error` enum into a `WinError`.
    ///
    /// - Maps `Error::RequiresUpdate` to `WSL_E_PLUGIN_REQUIRES_UPDATE`.
    /// - Maps `Error::WinError` directly to itself.
    ///
    /// # Returns
    /// A `WinError` representing the error.
    fn from(value: Error) -> Self {
        match value {
            Error::RequiresUpdate { .. } => HRESULT(WSL_E_PLUGIN_REQUIRES_UPDATE).into(),
            Error::WinError(error) => error,
        }
    }
}

/// A type alias for results using the custom `Error` type.
///
/// This alias simplifies the return type for functions that may fail, ensuring consistency
/// across the codebase.
pub type Result<T> = std::result::Result<T, Error>;
