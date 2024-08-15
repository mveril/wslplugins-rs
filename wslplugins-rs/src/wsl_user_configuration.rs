extern crate wslplugins_sys;

#[cfg(feature = "bitflags")]
use bitflags::bitflags;
#[cfg(feature = "enumflags2")]
use enumflags2::BitFlags;
#[cfg(feature = "flagset")]
use flagset::FlagSet;

// Define flags with bitflags
#[cfg(feature = "bitflags")]
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct WSLUserConfiguration: i32 {
        const CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel;
        const CustomKernelCommandLine = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine;
    }
}

// Define flags with flagset
#[cfg(feature = "flagset")]
#[derive(FlagSet, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WSLUserConfiguration {
    CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel,
    CustomKernelCommandLine =
        wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine,
}

// Define flags with enumflags2
#[cfg(feature = "enumflags2")]
#[derive(BitFlags, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WSLUserConfiguration {
    CustomKernel = wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernel,
    CustomKernelCommandLine =
        wslplugins_sys::WSLUserConfiguration_WSLUserConfigurationCustomKernelCommandLine,
}

// Manual validation of active features
#[cfg(any(
    all(feature = "bitflags", feature = "flagset"),
    all(feature = "bitflags", feature = "enumflags2"),
    all(feature = "flagset", feature = "enumflags2"),
))]
compile_error!("The features 'bitflags', 'flagset', and 'enumflags2' are mutually exclusive. Please activate only one.");
