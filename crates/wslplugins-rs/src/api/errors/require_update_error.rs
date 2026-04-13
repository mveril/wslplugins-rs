//! # WSL Plugin Error Handling
//!
//! This module provides an error type and result alias to handle scenarios where the current WSL version
//! does not meet the required version. It integrates with Windows error codes for seamless interop
//! with WSL APIs.

use crate::WSLVersion;
use thiserror::Error;
use windows_core::HRESULT;

/// Represents an error when the current WSL version is unsupported.
///
/// This error is returned when the WSL version being used does not satisfy
/// the required version for a plugin.
///
/// # Fields
/// - `current_version`: The current WSL version.
/// - `required_version`: The required WSL version.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[error("WSLVersion unsupported: current version {current_version}, required version {required_version}")]
pub struct Error {
    /// The current version of WSL.
    pub current_version: WSLVersion,
    /// The required version of WSL.
    pub required_version: WSLVersion,
}

impl Error {
    /// Creates a new `Error` with the specified current and required WSL versions.
    ///
    /// # Arguments
    /// - `current_version`: The version currently in use.
    /// - `required_version`: The version required for compatibility.
    ///
    /// # Returns
    /// A new instance of `Error`.
    #[must_use]
    #[inline]
    pub const fn new(current_version: WSLVersion, required_version: WSLVersion) -> Self {
        Self {
            current_version,
            required_version,
        }
    }
    pub const WSL_E_PLUGIN_REQUIRES_UPDATE: HRESULT =
        HRESULT(wslpluginapi_sys::WSL_E_PLUGIN_REQUIRES_UPDATE);
}

impl From<Error> for HRESULT {
    /// Converts the `Error` into the corresponding `HRESULT` error code.
    ///
    /// This implementation maps the custom `Error` to the `WSL_E_PLUGIN_REQUIRES_UPDATE` HRESULT.
    ///
    /// # Note
    /// [`Error::WSL_E_PLUGIN_REQUIRES_UPDATE`]: Indicates the WSL version is insufficient for the plugin.
    #[inline]
    fn from(_: Error) -> Self {
        Self(Error::WSL_E_PLUGIN_REQUIRES_UPDATE.0)
    }
}

impl From<Error> for windows_core::Error {
    /// Converts the `Error` into a [`windows_core::Error`].
    ///
    /// This implementation creates a [`windows_core::Error`] with the [`Error::WSL_E_PLUGIN_REQUIRES_UPDATE`] code.
    #[inline]
    fn from(value: Error) -> Self {
        Self::from(HRESULT::from(value))
    }
}

/// A result type alias for operations that may return a WSL plugin error.
///
/// This alias provides convenience when working with operations that might fail due to unsupported
/// WSL versions.
pub type Result<T> = std::result::Result<T, Error>;
