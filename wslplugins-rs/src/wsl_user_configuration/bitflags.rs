//! Provides a [mod@bitflags] implementation for [`WSLUserConfiguration`] flags.
use super::WSLUserConfiguration;
use bitflags::bitflags;

bitflags! {
    /// Represents the user configuration flags for Windows Subsystem for Linux (WSL) as
    /// [mod@bitflags]
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
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct WSLUserConfigurationFlags: i32 {
        /// A custom Linux kernel is used for the WSL instance.
        const CustomKernel = wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel;
        /// A custom kernel command-line is used for the WSL instance.
        const CustomKernelCommandLine = wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine;
    }
}

impl From<WSLUserConfiguration> for WSLUserConfigurationFlags {
    #[inline]
    fn from(value: WSLUserConfiguration) -> Self {
        Self::from_bits_truncate(value.0)
    }
}

impl From<WSLUserConfigurationFlags> for WSLUserConfiguration {
    #[inline]
    fn from(value: WSLUserConfigurationFlags) -> Self {
        value.bits().into()
    }
}

impl Default for WSLUserConfigurationFlags {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_bitflags() {
        let default_config = WSLUserConfigurationFlags::default();
        assert_eq!(default_config, WSLUserConfigurationFlags::empty());
        assert!(default_config.is_empty());
    }

    #[test]
    fn test_set_and_clear_bitflags() {
        let mut config = WSLUserConfigurationFlags::default();
        config.insert(WSLUserConfigurationFlags::CustomKernel);
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernel));
        config.remove(WSLUserConfigurationFlags::CustomKernel);
        assert!(!config.contains(WSLUserConfigurationFlags::CustomKernel));
    }

    #[test]
    fn test_multiple_flags_bitflags() {
        let mut config = WSLUserConfigurationFlags::default();
        config.insert(
            WSLUserConfigurationFlags::CustomKernel
                | WSLUserConfigurationFlags::CustomKernelCommandLine,
        );
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));
        config.remove(WSLUserConfigurationFlags::CustomKernel);
        assert!(!config.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));
    }

    #[test]
    fn test_conversion_between_flags_and_configuration() {
        let config = WSLUserConfiguration(3); // Assuming 3 = CustomKernel | CustomKernelCommandLine
        let flags: WSLUserConfigurationFlags = config.into();
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));

        let back_to_config: WSLUserConfiguration = flags.into();
        assert_eq!(back_to_config, WSLUserConfiguration(3));
    }
}
