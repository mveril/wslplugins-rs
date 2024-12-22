use thiserror::Error;
use windows::core::HRESULT;
use wslplugins_sys::{WSLVersion, WSL_E_PLUGIN_REQUIRES_UPDATE};

#[derive(Debug, Error)]
#[error("WSLVersion unsupported: current version {current_version}, required version {required_version}")]
pub struct Error {
    pub current_version: WSLVersion,
    pub required_version: WSLVersion,
}

impl Error {
    pub fn new(current_version: WSLVersion, required_version: WSLVersion) -> Self {
        Self {
            current_version,
            required_version,
        }
    }
}

impl From<Error> for HRESULT {
    fn from(_: Error) -> Self {
        WSL_E_PLUGIN_REQUIRES_UPDATE
    }
}

pub type Result<T> = std::result::Result<T, Error>;
