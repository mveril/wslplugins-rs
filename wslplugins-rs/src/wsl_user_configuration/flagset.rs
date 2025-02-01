//! Provides an [flagset] implementation for [WSLUserConfiguration] flags.
use super::WSLUserConfiguration;
use flagset::{flags, FlagSet};

flags! {
    /// Represents the user configuration flags for Windows Subsystem for Linux (WSL) as
    /// [flagset]
    ///
    /// These flags are used to customize the behavior of WSL instances based on user configuration.
    /// The values correspond to the definitions in the WSL Plugin API provided by Microsoft.
    ///
    /// # Variants
    ///
    /// - `CustomKernel`: Indicates that the WSL instance use use a custom Linux kernel instead of the default kernel.
    /// - `CustomKernelCommandLine`: Specifies that the WSL instance use use a custom kernel command-line during boot.
    ///
    /// # References
    ///
    /// See [WSL Configuration](https://learn.microsoft.com/windows/wsl/wsl-config)
    /// for additional details on WSL user configurations.
    #[derive(Hash)]
    pub enum WSLUserConfigurationFlags: i32 {
        /// A custom Linux kernel is used for the WSL instance.
        CustomKernel = wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel,
        /// A custom kernel command-line is used for the WSL instance.
        CustomKernelCommandLine =
            wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine,
    }
}

impl From<WSLUserConfiguration> for FlagSet<WSLUserConfigurationFlags> {
    fn from(value: WSLUserConfiguration) -> Self {
        FlagSet::new_truncated(value.0)
    }
}

impl From<FlagSet<WSLUserConfigurationFlags>> for WSLUserConfiguration {
    fn from(value: FlagSet<WSLUserConfigurationFlags>) -> Self {
        WSLUserConfiguration::from(value.bits())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flagset() {
        let default_config = FlagSet::<WSLUserConfigurationFlags>::default();
        assert_eq!(default_config, FlagSet::default());
        assert!(default_config.is_empty());
    }

    #[test]
    fn test_set_and_clear_flagset() {
        let mut config = FlagSet::<WSLUserConfigurationFlags>::default();
        config |= WSLUserConfigurationFlags::CustomKernel;
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernel));
        config &= !WSLUserConfigurationFlags::CustomKernel;
        assert!(!config.contains(WSLUserConfigurationFlags::CustomKernel));
    }

    #[test]
    fn test_multiple_flags_flagset() {
        let mut config = FlagSet::<WSLUserConfigurationFlags>::default();
        config |= WSLUserConfigurationFlags::CustomKernel
            | WSLUserConfigurationFlags::CustomKernelCommandLine;
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));
        config &= !WSLUserConfigurationFlags::CustomKernel;
        assert!(!config.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));
    }

    #[test]
    fn test_conversion_between_flags_and_configuration() {
        let config = WSLUserConfiguration(3); // Assuming 3 = CustomKernel | CustomKernelCommandLine
        let flags: FlagSet<WSLUserConfigurationFlags> = config.into();
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));

        let back_to_config: WSLUserConfiguration = flags.into();
        assert_eq!(back_to_config, WSLUserConfiguration(3));
    }
}
