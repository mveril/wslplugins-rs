use windows::core::Result as WinResult;
use wslplugins_rs::plugin::WSLPluginV1;
use wslplugins_rs::{wsl_plugin_v1, WSLContext, WSLVersionCapability};

pub(crate) struct Plugin;

#[wsl_plugin_v1(WSLVersionCapability::WSLC)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}

fn main() {}
