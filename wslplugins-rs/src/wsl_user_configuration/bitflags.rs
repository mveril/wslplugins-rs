use super::WSLUserConfiguration;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct WSLUserConfigurationFlags: i32 {
        const CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel;
        const CustomKernelCommandLine = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine;
    }
}

impl From<WSLUserConfiguration> for WSLUserConfigurationFlags {
    fn from(value: WSLUserConfiguration) -> Self {
        WSLUserConfigurationFlags::from_bits_truncate(value.0)
    }
}

impl From<WSLUserConfigurationFlags> for WSLUserConfiguration {
    fn from(value: WSLUserConfigurationFlags) -> Self {
        value.bits().into()
    }
}

impl Default for WSLUserConfigurationFlags {
    fn default() -> Self {
        WSLUserConfigurationFlags::empty()
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
