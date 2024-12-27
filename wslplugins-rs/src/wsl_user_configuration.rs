extern crate wslplugins_sys;

#[cfg(feature = "bitflags")]
use bitflags::bitflags;
#[cfg(feature = "enumflags2")]
use enumflags2::{bitflags, BitFlags};
#[cfg(feature = "flagset")]
use flagset::flags;

// Define with nothing
#[cfg(not(any(feature = "bitflags", feature = "flagset", feature = "enumflags2")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct WSLUserConfiguration(i32);

#[cfg(not(any(feature = "bitflags", feature = "flagset", feature = "enumflags2")))]
impl From<i32> for WSLUserConfiguration {
    fn from(value: i32) -> Self {
        WSLUserConfiguration(value)
    }
}

// Define flags with bitflags
#[cfg(feature = "bitflags")]
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct WSLUserConfiguration: i32 {
        const CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel;
        const CustomKernelCommandLine = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine;
    }
}

#[cfg(feature = "bitflags")]
impl Default for WSLUserConfiguration {
    fn default() -> Self {
        Self::empty()
    }
}

// Define flags with flagset
#[cfg(feature = "flagset")]
flags! {
    #[derive(Hash)]
    pub enum WSLUserConfiguration : i32 {
        CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel,
        CustomKernelCommandLine =
            wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine,
    }
}

// Define flags with enumflags2
#[cfg(feature = "enumflags2")]
#[bitflags]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum WSLUserConfiguration {
    CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel as u32,
    CustomKernelCommandLine =
        wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine as u32,
}

// Manual validation of active features
#[cfg(any(
    all(feature = "bitflags", feature = "flagset"),
    all(feature = "bitflags", feature = "enumflags2"),
    all(feature = "flagset", feature = "enumflags2"),
))]
compile_error!("The features 'bitflags', 'flagset', and 'enumflags2' are mutually exclusive. Please activate only one.");

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "flagset")]
    use flagset::FlagSet;

    // Tests pour la feature `bitflags`
    #[cfg(feature = "bitflags")]
    #[test]
    fn test_default_bitflags() {
        let default_config = WSLUserConfiguration::default();
        assert_eq!(default_config, WSLUserConfiguration::empty());
        assert!(default_config.is_empty());
    }

    #[cfg(feature = "bitflags")]
    #[test]
    fn test_set_and_clear_bitflags() {
        let mut config = WSLUserConfiguration::default();
        config.insert(WSLUserConfiguration::CustomKernel);
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        config.remove(WSLUserConfiguration::CustomKernel);
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
    }

    #[cfg(feature = "bitflags")]
    #[test]
    fn test_multiple_flags_bitflags() {
        let mut config = WSLUserConfiguration::default();
        config.insert(
            WSLUserConfiguration::CustomKernel | WSLUserConfiguration::CustomKernelCommandLine,
        );
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
        config.remove(WSLUserConfiguration::CustomKernel);
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
    }

    #[cfg(feature = "bitflags")]
    #[test]
    fn test_debug_display_bitflags() {
        let config =
            WSLUserConfiguration::CustomKernel | WSLUserConfiguration::CustomKernelCommandLine;
        assert_eq!(
            format!("{:?}", config),
            "WSLUserConfiguration(CustomKernel | CustomKernelCommandLine)"
        );
    }

    // Tests pour la feature `flagset`
    #[cfg(feature = "flagset")]
    #[test]
    fn test_default_flagset() {
        let default_config = FlagSet::<WSLUserConfiguration>::default();
        assert_eq!(default_config, FlagSet::default());
        assert!(default_config.is_empty());
    }

    #[cfg(feature = "flagset")]
    #[test]
    fn test_set_and_clear_flagset() {
        let mut config = FlagSet::<WSLUserConfiguration>::default();
        config ^= WSLUserConfiguration::CustomKernel;
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        config -= WSLUserConfiguration::CustomKernel;
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
    }

    #[cfg(feature = "flagset")]
    #[test]
    fn test_multiple_flags_flagset() {
        let mut config = FlagSet::<WSLUserConfiguration>::default();
        config ^=
            WSLUserConfiguration::CustomKernel | WSLUserConfiguration::CustomKernelCommandLine;
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
        config -= WSLUserConfiguration::CustomKernel;
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
    }

    #[cfg(feature = "flagset")]
    #[test]
    fn test_debug_display_flagset() {
        let config =
            WSLUserConfiguration::CustomKernel | WSLUserConfiguration::CustomKernelCommandLine;
        assert_eq!(format!("{:?}", config), "FlagSet { bits: 3 }");
    }

    // Tests pour la feature `enumflags2`
    #[cfg(feature = "enumflags2")]
    #[test]
    fn test_default_enumflags2() {
        let default_config = BitFlags::<WSLUserConfiguration>::default();
        assert_eq!(default_config, BitFlags::empty());
        assert!(default_config.is_empty());
    }

    #[cfg(feature = "enumflags2")]
    #[test]
    fn test_set_and_clear_enumflags2() {
        let mut config = BitFlags::<WSLUserConfiguration>::default();
        config.insert(WSLUserConfiguration::CustomKernel);
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        config.remove(WSLUserConfiguration::CustomKernel);
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
    }

    #[cfg(feature = "enumflags2")]
    #[test]
    fn test_multiple_flags_enumflags2() {
        let mut config = BitFlags::<WSLUserConfiguration>::default();
        config.insert(
            WSLUserConfiguration::CustomKernel | WSLUserConfiguration::CustomKernelCommandLine,
        );
        assert!(config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
        config.remove(WSLUserConfiguration::CustomKernel);
        assert!(!config.contains(WSLUserConfiguration::CustomKernel));
        assert!(config.contains(WSLUserConfiguration::CustomKernelCommandLine));
    }

    #[cfg(feature = "enumflags2")]
    #[test]
    fn test_debug_display_enumflags2() {
        let config = BitFlags::<WSLUserConfiguration>::from(WSLUserConfiguration::CustomKernel)
            | BitFlags::<WSLUserConfiguration>::from(WSLUserConfiguration::CustomKernelCommandLine);
        assert_eq!(format!("{:?}", config), "BitFlags(0b11)");
    }
}
