#![doc = include_str!("../README.md")]
use wslplugins_rs::prelude::*;

#[derive(Debug)]
pub(crate) struct Plugin;

#[wsl_plugin_v1]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }

    fn on_vm_started(
        &self,
        _session: &WSLSessionInformation,
        _user_settings: &WSLVmCreationSettings,
    ) -> PluginResult<()> {
        let array: [u8; 0] = [];
        #[allow(
            unconditional_panic,
            clippy::out_of_bounds_indexing,
            reason = "This example intentionally panics to demonstrate FFI panic containment"
        )]
        let _ = array[1];
        Ok(())
    }
}
