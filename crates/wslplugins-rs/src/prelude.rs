//! Common imports for building WSL plugins with this crate.
//!
//! # Example
//! ```rust
//! use wslplugins_rs::prelude::*;
//!
//! struct MyPlugin;
//!
//! impl WSLPluginV1 for MyPlugin {
//!     fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
//!         Ok(Self)
//!     }
//! }
//! ```

pub use crate::api::{ApiV1, PreparedWSLCommand, WSLCommand, WSLCommandExecution};
pub use crate::api::{Error as ApiError, Result as ApiResult};
pub use crate::plugin::{Error as PluginError, Result as PluginResult, WSLPluginV1};
pub use crate::windows_core::{Error as WinError, Result as WinResult};
#[cfg(feature = "semver")]
pub use crate::SemverConversionError;
pub use crate::{
    CoreDistributionInformation, DistributionID, DistributionInformation, HasSessionId,
    OfflineDistributionInformation, SessionID, UserDistributionID, WSLContext,
    WSLSessionInformation, WSLUserConfiguration, WSLVersion, WSLVmCreationSettings,
};

#[cfg(feature = "macro")]
pub use crate::wsl_plugin_v1;
