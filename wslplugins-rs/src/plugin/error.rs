use crate::WSLContext;
use log::debug;
use std::ffi::{OsStr, OsString};
use std::num::NonZeroI32;
use thiserror::Error;
use windows::core::{Error as WinError, HRESULT};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub struct Error {
    code: NonZeroI32,
    message: Option<OsString>,
}

impl std::fmt::Display for Error {
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
    pub fn new(code: HRESULT, message: Option<&OsStr>) -> Self {
        let code = if code.is_ok() {
            WinError::from_hresult(code).code()
        } else {
            code
        };
        let code = unsafe { NonZeroI32::new_unchecked(code.0) };

        Self {
            code,
            message: message.map(|m| m.to_owned()),
        }
    }

    pub fn with_code(code: HRESULT) -> Self {
        Self::new(code, None)
    }

    pub fn with_message(code: HRESULT, message: &OsStr) -> Self {
        Self::new(code, Some(message))
    }

    pub fn code(&self) -> HRESULT {
        HRESULT(self.code.get())
    }

    pub fn message(&self) -> Option<&OsStr> {
        self.message.as_deref()
    }

    pub(crate) fn consume_error_message_unwrap<R: From<Self>>(self) -> R {
        if let Some(ref mess) = self.message {
            if let Some(context) = WSLContext::get_current() {
                if let Err(err) = context.api.plugin_error(mess.as_os_str()) {
                    debug!(
                        "Unable to set plugin error message {} due to error: {}",
                        mess.to_string_lossy(),
                        err
                    )
                }
            }
        }
        R::from(self)
    }
}

impl From<Error> for WinError {
    fn from(value: Error) -> Self {
        match value.message {
            Some(ref message) => {
                let msg_string = message.to_string_lossy();
                WinError::new(value.code(), &msg_string)
            }
            None => WinError::from_hresult(value.code()),
        }
    }
}

impl From<WinError> for Error {
    fn from(value: WinError) -> Self {
        let os_message = if !value.message().is_empty() {
            Some(OsString::from(value.message()))
        } else {
            None
        };

        Self {
            code: unsafe { NonZeroI32::new_unchecked(value.code().0) },
            message: os_message,
        }
    }
}

impl From<HRESULT> for Error {
    fn from(value: HRESULT) -> Self {
        Self::new(value, None)
    }
}
