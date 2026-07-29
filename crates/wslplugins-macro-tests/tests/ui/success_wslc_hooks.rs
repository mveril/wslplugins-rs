use wslplugins_rs::api::{
    WSLCContainerInformation, WSLCImageInformation, WSLCObjectId, WSLCSessionInformation,
};
use wslplugins_rs::prelude::*;

struct Plugin;

#[wsl_plugin_v1(WSLVersionCapability::WSLC)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }

    fn on_session_created(&self, _session: &WSLCSessionInformation) -> WinResult<()> {
        Ok(())
    }

    fn on_session_stopping(&self, _session: &WSLCSessionInformation) -> WinResult<()> {
        Ok(())
    }

    fn container_started(
        &self,
        _session: &WSLCSessionInformation,
        _container: &WSLCContainerInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    fn container_stopping(
        &self,
        _session: &WSLCSessionInformation,
        _container_id: &WSLCObjectId,
    ) -> WinResult<()> {
        Ok(())
    }

    fn image_created(
        &self,
        _session: &WSLCSessionInformation,
        _image: &WSLCImageInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    fn image_deleted(
        &self,
        _session: &WSLCSessionInformation,
        _image_id: &WSLCObjectId,
    ) -> WinResult<()> {
        Ok(())
    }
}

fn main() {}
