#![doc = include_str!("../README.md")]

use wslplugins_rs::prelude::*;

const WINDOWS_SOURCE: &str = r"C:\WSL\Shared";
const LINUX_TARGET: &str = "/mnt/wsl-plugin-shared";
const MOUNT_NAME: &str = "wsl-plugin-shared";

#[derive(Debug)]
pub(crate) struct Plugin {
    context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        _user_settings: &WSLVmCreationSettings,
    ) -> PluginResult<()> {
        self.context.api.mount_folder(
            session.id(),
            WINDOWS_SOURCE,
            LINUX_TARGET,
            true,
            MOUNT_NAME,
        )?;
        Ok(())
    }
}
