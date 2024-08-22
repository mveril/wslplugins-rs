use crate::wsl_user_configuration::WSLUserConfiguration;
use wslplugins_sys;

pub struct WSLVmCreationSettings<'a>(&'a wslplugins_sys::WSLVmCreationSettings);

impl<'a> From<&'a wslplugins_sys::WSLVmCreationSettings> for WSLVmCreationSettings<'a> {
    fn from(value: &'a wslplugins_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }
}

impl WSLVmCreationSettings<'_> {
    

    #[cfg(feature = "bitflags")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        WSLUserConfiguration::from_bits_truncate(self.0.CustomConfigurationFlags)
    }

    #[cfg(feature = "flagset")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
            WSLUserConfiguration::from_bits_retain(self.0.CustomConfigurationFlags).unwrap()
    }

    #[cfg(feature = "enumflags2")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
            WSLUserConfiguration::from_bits_truncate(self.0.CustomConfigurationFlags as u8)
    }
}
