//! # Plugin Creation and Result Handling Utilities
//!
//! This module provides utility functions for creating WSL plugins and handling results,
//! enabling smooth integration with the WSL Plugin API.

use windows::{
    core::{Error as WinError, Result as WinResult},
    Win32::Foundation::ERROR_ALREADY_INITIALIZED,
};
use wslplugins_sys::WSLPluginAPIV1;

use crate::{api::ApiV1, WSLContext};

use super::{Result, WSLPluginV1};

/// Creates a WSL plugin instance with a specified required API version.
///
/// This function ensures that the provided WSL Plugin API meets the required version before
/// creating a new plugin instance. If the API version does not meet the requirements, an error is returned.
///
/// # Type Parameters
/// - `T`: A type implementing the [WSLPluginV1] trait, representing the plugin to create.
///
/// # Arguments
/// - `api`: A reference to the WSL Plugin API version 1 structure.
/// - `required_major`: The required major version of the API.
/// - `required_minor`: The required minor version of the API.
/// - `required_revision`: The required revision of the API.
///
/// # Returns
/// - `Ok(T)`: The created plugin instance.
/// - `Err(WinError)`: If the API version is insufficient or the plugin is already initialized.
///
/// # Errors
/// - Returns [WinError]`::from(`[ERROR_ALREADY_INITIALIZED]`)` if a plugin is already initialized.
/// - Returns [WinError]`::from(`[WSL_E_PLUGIN_REQUIRES_UPDATE](wslplugins_sys::WSL_E_PLUGIN_REQUIRES_UPDATE)`)` error if the API version is insufficient.
///
/// # Safety
/// This function calls an unsafe API to check the reqred version. Ensure the provided API pointer
/// is valid and correctly initialized.
pub fn create_plugin_with_required_version<T: WSLPluginV1>(
    api: &'static WSLPluginAPIV1,
    required_major: u32,
    required_minor: u32,
    required_revision: u32,
) -> WinResult<T> {
    unsafe {
        wslplugins_sys::require_version(required_major, required_minor, required_revision, api)
            .ok()?;
    }
    if let Some(context) = WSLContext::init(ApiV1::from(api)) {
        let plugin = T::try_new(context)?;
        Ok(plugin)
    } else {
        Err(WinError::from(ERROR_ALREADY_INITIALIZED))
    }
}

pub fn consume_to_win_result<T>(result: Result<T>) -> WinResult<T> {
    result.map_err(|err| err.consume_error_message_unwrap())
}
