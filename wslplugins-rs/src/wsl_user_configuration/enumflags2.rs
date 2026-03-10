//! Provides an [enumflags2] implementation for [`WSLUserConfiguration`] flags.
use super::WSLUserConfiguration;
use enumflags2::{bitflags, BitFlags};

/// Represents the user configuration flags for Windows Subsystem for Linux (WSL) as
/// [enumflags2]
///
/// These flags are used to customize the behavior of WSL instances based on user configuration.
/// The values correspond to the definitions in the WSL Plugin API provided by Microsoft.
///
/// # Variants
///
/// - `CustomKernel`: Indicates that the WSL instance use use a custom Linux kernel instead of the default kernel.
/// - `CustomKernelCommandLine`: Specifies that the WSL instance use use a custom kernel command-line during boot.
///
///
/// # References
///
/// See [WSL Configuration](https://learn.microsoft.com/windows/wsl/wsl-config)
/// for additional details on WSL user configurations.
#[bitflags]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum WSLUserConfigurationFlags {
    /// A custom Linux kernel is used for the WSL instance.
    CustomKernel = wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel as u32,

    /// A custom kernel command-line is used for the WSL instance.
    CustomKernelCommandLine =
        wslpluginapi_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine as u32,
}

impl From<WSLUserConfiguration> for BitFlags<WSLUserConfigurationFlags> {
    #[inline]
    fn from(value: WSLUserConfiguration) -> Self {
        BitFlags::from_bits_truncate(value.0.cast_unsigned())
    }
}

impl From<BitFlags<WSLUserConfigurationFlags>> for WSLUserConfiguration {
    #[inline]
    fn from(value: BitFlags<WSLUserConfigurationFlags>) -> Self {
        (value.bits().cast_signed()).into()
    }
}

impl WSLUserConfiguration {
    #[inline]
    #[must_use]
    pub fn into_enumflags2(self) -> BitFlags<WSLUserConfigurationFlags> {
        BitFlags::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enumflags2::BitFlags;
    #[test]
    fn test_default_enumflags2() {
        let default_config = BitFlags::<WSLUserConfigurationFlags>::default();
        assert_eq!(default_config, BitFlags::empty());
        assert!(default_config.is_empty());
    }

    #[test]
    fn test_set_and_clear_enumflags2() {
        let mut config = BitFlags::<WSLUserConfigurationFlags>::default();
        config.insert(WSLUserConfigurationFlags::CustomKernel);
        assert!(config.contains(WSLUserConfigurationFlags::CustomKernel));
        config.remove(WSLUserConfigurationFlags::CustomKernel);
        assert!(!config.contains(WSLUserConfigurationFlags::CustomKernel));
    }

    #[test]
    fn test_multiple_flags_enumflags2() {
        let mut config = BitFlags::<WSLUserConfigurationFlags>::default();
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
        let flags: BitFlags<WSLUserConfigurationFlags> = config.into();
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernel));
        assert!(flags.contains(WSLUserConfigurationFlags::CustomKernelCommandLine));

        let back_to_config: WSLUserConfiguration = flags.into();
        assert_eq!(back_to_config, WSLUserConfiguration(3));
    }
}
