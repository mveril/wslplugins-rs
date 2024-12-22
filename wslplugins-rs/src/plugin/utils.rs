use windows::{
    core::{Error as WinError, Result as WinResult},
    Win32::Foundation::ERROR_ALREADY_INITIALIZED,
};
use wslplugins_sys::WSLPluginAPIV1;

use crate::{api::ApiV1, WSLContext};

use super::{Result, WSLPluginV1};

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
