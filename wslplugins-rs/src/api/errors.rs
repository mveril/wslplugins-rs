use thiserror::Error;
pub mod require_update_error;
pub use require_update_error::Error as RequireUpdateError;
use windows::core::{Error as WinError, HRESULT};
use wslplugins_sys::WSL_E_PLUGIN_REQUIRES_UPDATE;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Require Update Error")]
    RequiresUpdate(#[from] RequireUpdateError),
    #[error("Windows Error: {0}")]
    WinError(#[from] WinError),
}

impl From<Error> for HRESULT {
    fn from(value: Error) -> Self {
        match value {
            Error::RequiresUpdate(err) => err.into(),
            Error::WinError(err) => err.code(),
        }
    }
}

impl From<Error> for WinError {
    fn from(value: Error) -> Self {
        match value {
            Error::RequiresUpdate { .. } => WSL_E_PLUGIN_REQUIRES_UPDATE.into(),
            Error::WinError(error) => error,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
