use super::WSLUserConfiguration;
use flagset::{flags, FlagSet};

flags! {
    #[derive(Hash)]
    pub enum WSLUserConfigurationFlags: i32 {
        CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel,
        CustomKernelCommandLine =
            wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine,
    }
}

impl From<WSLUserConfiguration> for FlagSet<WSLUserConfigurationFlags> {
    fn from(value: WSLUserConfiguration) -> Self {
        FlagSet::new_truncated(value.0 as i32)
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
