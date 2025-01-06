//! Provides types and utilities to interact with WSL (Windows Subsystem for Linux) [user configurations](https://learn.microsoft.com/windows/wsl/wsl-config).
//!
//! This module defines the `WSLUserConfiguration` struct for representing user configurations as an integer,
//! along with optional submodules for flag handling depending on feature flags.
//!
//!
//! # References
//!
//! See [WSL Configuration](https://learn.microsoft.com/windows/wsl/wsl-config)
//! for additional details on WSL user configurations.
//! # Features
//!
//! - **`bitflags`**: Provides a [bitflags]-based implementation for handling user configuration flags.
//! - **`enumflags2`**: Provides an [enumflags2]-based implementation for handling user configuration flags.
//! - **`flagset`**: Provides a [flagset]-based implementation for handling user configuration flags.

#[cfg(feature = "bitflags")]
pub mod bitflags;

#[cfg(feature = "enumflags2")]
pub mod enumflags2;

#[cfg(feature = "flagset")]
pub mod flagset;

/// Represents a WSL user configuration as an integer.
///
/// This struct provides a simple wrapper around a 32-bit integer ([i32]), allowing for
/// easy conversion to and from [i32] values and also flags depending on the enabled feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct WSLUserConfiguration(i32);

impl From<i32> for WSLUserConfiguration {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<WSLUserConfiguration> for i32 {
    fn from(value: WSLUserConfiguration) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::WSLUserConfiguration;

    #[test]
    fn test_i32_to_wsl_user_configuration() {
        let value: i32 = 42;
        let config: WSLUserConfiguration = value.into();
        assert_eq!(config, WSLUserConfiguration(42));
    }

    #[test]
    fn test_wsl_user_configuration_to_i32() {
        let config = WSLUserConfiguration(99);
        let value: i32 = config.into();
        assert_eq!(value, 99);
    }

    #[test]
    fn test_default_wsl_user_configuration() {
        let config = WSLUserConfiguration::default();
        assert_eq!(config, WSLUserConfiguration(0));
    }
}
