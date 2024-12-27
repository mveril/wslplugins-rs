use crate::wsl_user_configuration::WSLUserConfiguration;
use std::fmt::Debug;

#[cfg(feature = "enumflags2")]
use enumflags2::BitFlags;

#[cfg(feature = "flagset")]
use flagset::FlagSet;

pub struct WSLVmCreationSettings<'a>(&'a wslplugins_sys::WSLVmCreationSettings);

impl<'a> From<&'a wslplugins_sys::WSLVmCreationSettings> for WSLVmCreationSettings<'a> {
    fn from(value: &'a wslplugins_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }
}

impl WSLVmCreationSettings<'_> {
    #[cfg(not(any(feature = "bitflags", feature = "flagset", feature = "enumflags2")))]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        WSLUserConfiguration::from(self.0.CustomConfigurationFlags)
    }

    #[cfg(feature = "bitflags")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        WSLUserConfiguration::from_bits_truncate(self.0.CustomConfigurationFlags)
    }

    #[cfg(feature = "flagset")]
    pub fn custom_configuration_flags(&self) -> FlagSet<WSLUserConfiguration> {
        FlagSet::<WSLUserConfiguration>::new_truncated(self.0.CustomConfigurationFlags)
    }

    #[cfg(feature = "enumflags2")]
    pub fn custom_configuration_flags(&self) -> BitFlags<WSLUserConfiguration> {
        BitFlags::from_bits_truncate(self.0.CustomConfigurationFlags as u32)
    }
}

impl Debug for WSLVmCreationSettings<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WSLVmCreationSettings")
            .field(
                "customConfigurationFlags",
                &self.custom_configuration_flags(),
            )
            .finish()
    }
}
