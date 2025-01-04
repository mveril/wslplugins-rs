use crate::wsl_user_configuration::WSLUserConfiguration;
use std::fmt::Debug;

pub struct WSLVmCreationSettings<'a>(&'a wslplugins_sys::WSLVmCreationSettings);

impl<'a> From<&'a wslplugins_sys::WSLVmCreationSettings> for WSLVmCreationSettings<'a> {
    fn from(value: &'a wslplugins_sys::WSLVmCreationSettings) -> Self {
        WSLVmCreationSettings(value)
    }
}

impl WSLVmCreationSettings<'_> {
    pub fn custom_configuration_flags(&self) -> WSLUserConfiguration {
        WSLUserConfiguration::from(self.0.CustomConfigurationFlags)
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
