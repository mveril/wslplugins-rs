use crate::wsl_user_configuration::WSLUserConfiguration;
use wslplugins_sys;

pub struct WSLVmCreationSettings(*const wslplugins_sys::WSLVmCreationSettings);

impl WSLVmCreationSettings {
    pub fn from_raw(value: *const wslplugins_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }

    #[cfg(feature = "bitflags")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        unsafe {
            WSLUserConfiguration::from_bits_truncate((*self.0).CustomConfigurationFlags)
        }
    }

    #[cfg(feature = "flagset")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        unsafe {
            WSLUserConfiguration::from_bits_retain((*self.0).CustomConfigurationFlags)
                .unwrap()
        }
    }

    #[cfg(feature = "enumflags2")]
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        unsafe {
            WSLUserConfiguration::from_bits_truncate(
                (*self.0).CustomConfigurationFlags as u8,
            )
        }
    }
}