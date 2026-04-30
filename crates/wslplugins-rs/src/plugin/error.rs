//! # WSL Plugin Error Handling
//!
//! This module defines a custom error type for handling errors in WSL plugins.
//! It integrates with Windows APIs, supports error codes and optional error messages,
//! and provides utility methods for error creation and consumption.

use crate::api::{errors::RequireUpdateError, Error as ApiError, WSLCommandExecutionError};
use crate::WSLContext;
use std::borrow::ToOwned;
use std::ffi::{OsStr, OsString};
use std::num::NonZeroI32;
use thiserror::Error;
#[cfg(feature = "tracing")]
use tracing::debug;
use windows_core::{Error as WinError, HRESULT};

/// A specialized result type for operations that may return a WSL plugin error.
///
/// This alias simplifies the function signatures throughout the WSL plugin codebase.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents an error in the WSL plugin system.
///
/// This struct encapsulates:
/// - An error code (`HRESULT`) derived from Windows APIs.
/// - An optional error message (`OsString`).
#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    /// The error code associated with the failure.
    code: NonZeroI32,
    /// An optional error message providing more context about the error.
    message: Option<OsString>,
}

impl std::fmt::Display for Error {
    /// Formats the error for display.
    ///
    /// If an error message is present, it is included in the output.
    /// Otherwise, only the error code is displayed.
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(message) => write!(
                f,
                "WSLPluginError {}: {}",
                self.code,
                message.to_string_lossy()
            ),
            None => write!(f, "WSLPluginError {}", self.code),
        }
    }
}

impl Error {
    /// Creates a new error with a specified code and optional message.
    ///
    /// # Arguments
    /// - `code`: The HRESULT representing the error code.
    /// - `message`: An optional error message.
    ///
    /// # Returns
    /// A new instance of `Error`.
    #[must_use]
    #[inline]
    pub fn new<S: AsRef<OsStr>>(code: HRESULT, message: Option<S>) -> Self {
        let code = if code.is_ok() {
            WinError::from_hresult(code).code()
        } else {
            code
        };
        // SAFETY: the code getted from WinError is guaranteed to be valid
        let code = unsafe { NonZeroI32::new_unchecked(code.0) };

        Self {
            code,
            message: message.map(|m| m.as_ref().to_owned()),
        }
    }

    /// Creates an error with only an error code.
    ///
    /// # Arguments
    /// - `code`: The HRESULT representing the error code.
    ///
    /// # Returns
    /// A new instance of `Error` without an associated message.
    #[must_use]
    #[inline]
    pub fn with_code(code: HRESULT) -> Self {
        Self::new::<&OsStr>(code, None)
    }

    /// Creates an error with both a code and a message.
    ///
    /// # Arguments
    /// - `code`: The HRESULT representing the error code.
    /// - `message`: An associated error message.
    ///
    /// # Returns
    /// A new instance of `Error`.
    #[must_use]
    #[inline]
    pub fn with_message<S: AsRef<OsStr>>(code: HRESULT, message: S) -> Self {
        Self::new(code, Some(message))
    }

    /// Retrieves the error code as an `HRESULT`.
    ///
    /// # Returns
    /// The error code wrapped in an `HRESULT`.
    #[inline]
    pub const fn code(&self) -> HRESULT {
        HRESULT(self.code.get())
    }

    /// Retrieves the optional error message.
    ///
    /// # Returns
    /// A reference to the error message, if present.
    #[must_use]
    #[inline]
    pub fn message(&self) -> Option<&OsStr> {
        self.message.as_deref()
    }

    /// Consumes the error and optionally sets the plugin's error message in the WSL context.
    ///
    /// # Type Parameters
    /// - `R`: The type to return after consuming the error.
    ///
    /// # Returns
    /// The consumed error converted to the specified type.
    pub(crate) fn consume_error_message_unwrap<R: From<Self>>(self) -> R {
        if let Some(ref mess) = self.message {
            if let Some(context) = WSLContext::get_current() {
                #[cfg_attr(not(feature = "tracing"), expect(unused_variables))]
                let plugin_error_result = context.api.plugin_error(mess.as_os_str());
                #[cfg(feature = "tracing")]
                if let Err(err) = plugin_error_result {
                    debug!(
                        "Unable to set plugin error message {} due to error: {}",
                        mess.to_string_lossy(),
                        err
                    );
                }
            }
        }
        R::from(self)
    }
}

impl From<Error> for WinError {
    /// Converts the `Error` into a `WinError`.
    ///
    /// # Returns
    /// A `WinError` constructed from the error's code and message.
    #[inline]
    fn from(value: Error) -> Self {
        let code = value.code();
        value.message.as_ref().map_or_else(
            || Self::from_hresult(code),
            |message| {
                let msg_string = message.to_string_lossy();
                Self::new(code, &msg_string)
            },
        )
    }
}

impl From<WinError> for Error {
    /// Converts a `WinError` into an `Error`.
    ///
    /// # Returns
    /// An `Error` containing the code and message from the `WinError`.
    #[inline]
    fn from(value: WinError) -> Self {
        let os_message = if value.message().is_empty() {
            None
        } else {
            Some(OsString::from(value.message()))
        };

        Self {
            // SAFETY: As we have a valid WinError, we can safely extract the code.
            code: unsafe { NonZeroI32::new_unchecked(value.code().0) },
            message: os_message,
        }
    }
}

impl From<HRESULT> for Error {
    /// Converts an `HRESULT` into an `Error`.
    ///
    /// # Returns
    /// An `Error` containing the `HRESULT` as its code.
    #[inline]
    fn from(value: HRESULT) -> Self {
        Self::new::<&OsStr>(value, None)
    }
}

impl From<RequireUpdateError> for Error {
    #[inline]
    fn from(value: RequireUpdateError) -> Self {
        Self::from(HRESULT::from(value))
    }
}

impl From<ApiError> for Error {
    #[inline]
    fn from(value: ApiError) -> Self {
        Self::from(HRESULT::from(value))
    }
}

impl From<WSLCommandExecutionError> for Error {
    #[inline]
    fn from(value: WSLCommandExecutionError) -> Self {
        let code = value.code();
        match value {
            WSLCommandExecutionError::Api(error) => Self::from(error),
            WSLCommandExecutionError::Split { source, .. } => {
                Self::with_message(code, source.to_string())
            }
        }
    }
}
