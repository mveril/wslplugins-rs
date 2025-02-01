//! # WSL VM Creation Settings
//!
//! This module provides a safe abstraction for handling WSL VM creation settings. It wraps the
//! `WSLVmCreationSettings` structure from the WSL Plugin API and provides utilities for accessing
//! custom configuration flags with support for multiple flag management libraries.

use std::fmt::Debug;

use crate::WSLUserConfiguration;

/// Represents WSL VM creation settings.
///
/// This struct wraps the `WSLVmCreationSettings` structure from the WSL Plugin API, providing
/// safe and idiomatic Rust access to its fields.
pub struct WSLVmCreationSettings(wslpluginapi_sys::WSLVmCreationSettings);

impl From<wslpluginapi_sys::WSLVmCreationSettings> for WSLVmCreationSettings {
    fn from(value: wslpluginapi_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }
}

impl From<WSLVmCreationSettings> for wslpluginapi_sys::WSLVmCreationSettings {
    fn from(value: WSLVmCreationSettings) -> Self {
        value.0
    }
}

impl AsRef<wslpluginapi_sys::WSLVmCreationSettings> for WSLVmCreationSettings {
    fn as_ref(&self) -> &wslpluginapi_sys::WSLVmCreationSettings {
        &self.0
    }
}

impl AsRef<WSLVmCreationSettings> for wslpluginapi_sys::WSLVmCreationSettings {
    fn as_ref(&self) -> &WSLVmCreationSettings {
        unsafe {
            &*(self as *const wslpluginapi_sys::WSLVmCreationSettings
                as *const WSLVmCreationSettings)
        }
    }
}

impl WSLVmCreationSettings {
    /// Retrieves the custom configuration flags for the VM.
    ///
    /// # Returns
    /// A wrapper type type representing the custom configuration flags.
    /// This type is convertible to some flags if the associated feature is enabled
    /// - **`bitflags`**: Uses the [bitflags]  crate for managing flags.
    /// - **`flagset`**: Uses the [flagset] crate for managing flags.
    /// - **`enumflags2`**: Uses the [enumflags2] crate for managing flags.
    ///
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        WSLUserConfiguration::from(self.0.CustomConfigurationFlags)
    }
}

impl Debug for WSLVmCreationSettings {
    /// Formats the VM creation settings for debugging.
    ///
    /// The debug output includes the custom configuration flags.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WSLVmCreationSettings")
            .field(
                "customConfigurationFlags",
                &self.custom_configuration_flags(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_transparence;

    use super::WSLVmCreationSettings;

    #[test]
    fn test_layouts() {
        test_transparence::<wslpluginapi_sys::WSLVmCreationSettings, WSLVmCreationSettings>();
    }
}
