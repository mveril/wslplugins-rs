//! Provides an [enumflags2] implementation for [WSLUserConfiguration] flags.
use super::WSLUserConfiguration;
use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
/// Represents the user configuration flags for WSL as an enumflags2 bitflags.
pub enum WSLUserConfigurationFlags {
    CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel as u32,
    CustomKernelCommandLine =
        wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine as u32,
}

impl From<WSLUserConfiguration> for BitFlags<WSLUserConfigurationFlags> {
    fn from(value: WSLUserConfiguration) -> Self {
        BitFlags::from_bits_truncate(value.0 as u32)
    }
}

impl From<BitFlags<WSLUserConfigurationFlags>> for WSLUserConfiguration {
    fn from(value: BitFlags<WSLUserConfigurationFlags>) -> Self {
        (value.bits() as i32).into()
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
