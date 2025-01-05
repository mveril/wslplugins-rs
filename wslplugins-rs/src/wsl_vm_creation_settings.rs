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
///
/// # Lifetime Parameters
/// - `'a`: The lifetime of the referenced `WSLVmCreationSettings` instance.
pub struct WSLVmCreationSettings<'a>(&'a wslplugins_sys::WSLVmCreationSettings);

impl<'a> From<&'a wslplugins_sys::WSLVmCreationSettings> for WSLVmCreationSettings<'a> {
    /// Creates a `WSLVmCreationSettings` instance from a reference to the raw WSL Plugin API structure.
    ///
    /// # Arguments
    /// - `value`: A reference to a `WSLVmCreationSettings` instance from the WSL Plugin API.
    ///
    /// # Returns
    /// A wrapped `WSLVmCreationSettings` instance.
    fn from(value: &'a wslplugins_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }
}

impl WSLVmCreationSettings<'_> {
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

impl Debug for WSLVmCreationSettings<'_> {
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
