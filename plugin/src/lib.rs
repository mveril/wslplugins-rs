mod plugin;
use crate::plugin::Plugin;
use std::cell::OnceCell;
use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::E_ABORT;
use wslplugins_rs::{ApiV1, WSLPluginV1};
use wslplugins_rs::{
    DistributionInformation as DistributionInformationWrapper,
    WSLSessionInformation as WSLSessionInformationWrapper,
    WSLVmCreationSettings as WSLVmCreationSettingsWrapper,
};
use wslplugins_sys::*;

// WSLService call the plugins synchroniously so we know all the call to the plugin struct will be on one thread
thread_local! {
    static PLUGIN: OnceCell<Plugin> = OnceCell::new();
}

const MAJOR: u32 = 1;
const MINOR: u32 = 0;
const REVISION: u32 = 5;

#[no_mangle]
pub extern "C" fn on_vm_started(
    session: *const WSLSessionInformation,
    settings: *const WSLVmCreationSettings,
) -> HRESULT {
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_vm_started(
                &WSLSessionInformationWrapper::from_raw(session),
                &WSLVmCreationSettingsWrapper::from_raw(settings),
            )
            .into()
    })
}

#[no_mangle]
pub extern "C" fn on_vm_stopping(session: *const WSLSessionInformation) -> HRESULT {
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_vm_stopping(&WSLSessionInformationWrapper::from_raw(session))
            .into()
    })
}

// Gestion des événements de la distribution
#[no_mangle]
pub extern "C" fn on_distro_started(
    session: *const WSLSessionInformation,
    distribution: *const WSLDistributionInformation,
) -> HRESULT {
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_distribution_started(
                &WSLSessionInformationWrapper::from_raw(session),
                &DistributionInformationWrapper::from_raw(distribution),
            )
            .into()
    })
}

#[no_mangle]
pub extern "C" fn on_distro_stopping(
    session: *const WSLSessionInformation,
    distribution: *const WSLDistributionInformation,
) -> HRESULT {
    PLUGIN.with(|cell: &OnceCell<Plugin>| {
        cell.get()
            .unwrap()
            .on_distribution_stopping(
                &WSLSessionInformationWrapper::from_raw(session),
                &DistributionInformationWrapper::from_raw(distribution),
            )
            .into()
    })
}

fn create_plugin(
    api: *const WSLPluginAPIV1,
    hooks: *mut WSLPluginHooksV1,
) -> windows::core::Result<()> {
    unsafe {
        wslplugins_sys::require_version(MAJOR, MINOR, REVISION, api).ok()?;
    }
    let plugin = Plugin::try_new(ApiV1::from_raw(api))?;
    let hook_ptr = &mut unsafe { *hooks };
    hook_ptr.OnVMStarted = Some(on_vm_started);
    hook_ptr.OnVMStopping = Some(on_vm_stopping);
    hook_ptr.OnDistributionStarted = Some(on_distro_started);
    hook_ptr.OnDistributionStopping = Some(on_distro_stopping);
    if PLUGIN.with(|cell| cell.set(plugin)).is_err() {
        return Err(Error::from(E_ABORT));
    }
    Ok(())
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn WSLPluginAPIV1_EntryPoint(
    api: *const WSLPluginAPIV1,
    hooks: *mut WSLPluginHooksV1,
) -> HRESULT {
    create_plugin(api, hooks).into()
}
