//! # Plugin Creation and Result Handling Utilities
//!
//! This module provides utility functions for creating WSL plugins and handling results,
//! enabling smooth integration with the WSL Plugin API.

use windows_core::{Error as WinError, Result as WinResult, HRESULT};
use wslpluginapi_sys::{windows_sys::Win32::Foundation::ERROR_ALREADY_INITIALIZED, WSLPluginAPIV1};

use crate::WSLContext;

#[cfg(doc)]
use super::Error;
use super::{Result, WSLPluginV1};

/// Creates a WSL plugin instance with a specified required API version.
///
/// This function ensures that the provided WSL Plugin API meets the required version before
/// creating a new plugin instance. If the API version does not meet the requirements, an error is returned.
///
/// # Type Parameters
/// - `T`: A type implementing the [`WSLPluginV1`] trait, representing the plugin to create.
///
/// # Arguments
/// - `api`: A reference to the WSL Plugin API version 1 structure.
/// - `required_major`: The required major version of the API.
/// - `required_minor`: The required minor version of the API.
/// - `required_revision`: The required revision of the API.
///
/// # Returns
/// - `Ok(plugin)`: The created plugin instance.
/// - `Err(WinError)`: If the API version is insufficient or the plugin is already initialized.
///
/// # Errors
/// - Returns <code>[WinError]::from([ERROR_ALREADY_INITIALIZED])</code> if a plugin is already initialized.
/// - Returns <code>[WinError]::from([WSL_E_PLUGIN_REQUIRES_UPDATE](wslpluginapi_sys::WSL_E_PLUGIN_REQUIRES_UPDATE))</code> error if the API version is insufficient.
#[inline]
pub fn create_plugin_with_required_version<T: WSLPluginV1>(
    api: &'static WSLPluginAPIV1,
    required_major: u32,
    required_minor: u32,
    required_revision: u32,
) -> WinResult<T> {
    // SAFETY: As api reference exist it's safe to call it
    unsafe {
        HRESULT(wslpluginapi_sys::require_version(
            required_major,
            required_minor,
            required_revision,
            api,
        ))
        .ok()?;
    };
    WSLContext::init(api.as_ref())
        .ok_or_else(|| WinError::from_hresult(HRESULT::from_win32(ERROR_ALREADY_INITIALIZED)))
        .and_then(T::try_new)
}

#[expect(clippy::missing_errors_doc)]
/// Converts a generic `Result<T>` using the custom `Error` type into a `WinResult<T>`.
///
/// This function simplifies the interoperability between the custom error handling
/// in the WSL plugin system and the Windows error system by mapping the plugin [Error]
/// into a [`windows_core::Error`] using the `consume_error_message_unwrap` method.
///
/// # Arguments
/// - `result`: A [`Result<T>`] using the custom [Error] type defined in this crate.
///
/// # Returns
/// A `WinResult<T>` where:
/// - `Ok(value)` contains the successful result `T`.
/// - `Err(error)` contains a [`windows_core::Error`] converted from the plugin [Error].
///
/// # Behavior
/// - If the `result` is `Ok`, it is returned as-is.
/// - If the `result` is `Err`, the error is consumed
///   and sent to WSL and is then
///   converted into a [`windows_core::Error`].
///
/// # Usage
/// This utility is intended to facilitate the transition between idiomatic Rust
/// error handling and the Windows API error-handling conventions.
#[inline]
pub fn consume_to_win_result<T>(result: Result<T>) -> WinResult<T> {
    result.map_err(super::error::Error::consume_error_message_unwrap)
}
