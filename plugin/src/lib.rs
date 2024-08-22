mod plugin;
use crate::plugin::Plugin;
use log::{debug, error};
use log_instrument::instrument;
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
    static PLUGIN: OnceCell<Plugin<'static>> = OnceCell::new();
}

const MAJOR: u32 = 1;
const MINOR: u32 = 0;
const REVISION: u32 = 5;

#[instrument]
#[no_mangle]
pub extern "C" fn on_vm_started(
    session: *const WSLSessionInformation,
    settings: *const WSLVmCreationSettings,
) -> HRESULT {
    let session_ptr = unsafe { &*session };
    let settings_ptr = unsafe { &*settings };
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_vm_started(
                &WSLSessionInformationWrapper::from(session_ptr),
                &WSLVmCreationSettingsWrapper::from(settings_ptr),
            )
            .into()
    })
}

#[instrument]
#[no_mangle]
pub extern "C" fn on_vm_stopping(session: *const WSLSessionInformation) -> HRESULT {
    let session_ptr = unsafe { &*session };
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_vm_stopping(&WSLSessionInformationWrapper::from(session_ptr))
            .into()
    })
}

#[instrument]
#[no_mangle]
pub extern "C" fn on_distro_started(
    session: *const WSLSessionInformation,
    distribution: *const WSLDistributionInformation,
) -> HRESULT {
    let session_ptr = unsafe { &*session };
    let distribution_ptr = unsafe { &*distribution };
    PLUGIN.with(|cell| {
        cell.get()
            .unwrap()
            .on_distribution_started(
                &WSLSessionInformationWrapper::from(session_ptr),
                &DistributionInformationWrapper::from(distribution_ptr),
            )
            .into()
    })
}
#[instrument]
#[no_mangle]
pub extern "C" fn on_distro_stopping(
    session: *const WSLSessionInformation,
    distribution: *const WSLDistributionInformation,
) -> HRESULT {
    let session_ptr = unsafe { &*session };
    let distribution_ptr = unsafe { &*distribution };
    PLUGIN.with(|cell: &OnceCell<Plugin>| {
        cell.get()
            .unwrap()
            .on_distribution_stopping(
                &WSLSessionInformationWrapper::from(session_ptr),
                &DistributionInformationWrapper::from(distribution_ptr),
            )
            .into()
    })
}

fn create_plugin(
    api: &'static WSLPluginAPIV1,
    hooks: &mut WSLPluginHooksV1,
) -> windows::core::Result<()> {
    unsafe {
        wslplugins_sys::require_version(MAJOR, MINOR, REVISION, api).ok()?;
    }
    let plugin = Plugin::try_new(ApiV1::from(api))?;
    set_hooks(hooks);
    PLUGIN.with(|cell| cell.set(plugin)).map_err(|_| {
        error!("Plugin not set !");
        Error::from(E_ABORT)
    })?;
    Ok(())
}

#[instrument]
fn set_hooks(hooks: &mut WSLPluginHooksV1) {
    hooks.OnVMStarted = Some(on_vm_started);
    debug!(
        "{:} defined on {:?}",
        stringify!(hook_ptr.OnVMStarted),
        hooks.OnVMStarted
    );
    hooks.OnVMStopping = Some(on_vm_stopping);
    debug!(
        "{:} defined on {:?}",
        stringify!(hook_ptr.OnVMStopping),
        hooks.OnVMStopping
    );
    hooks.OnDistributionStarted = Some(on_distro_started);
    debug!(
        "{:} defined on {:?}",
        stringify!(hook_ptr.OnDistributionStarted),
        hooks.OnDistributionStarted
    );
    hooks.OnDistributionStopping = Some(on_distro_stopping);
    debug!(
        "{:} defined on {:?}",
        stringify!(hook_ptr.OnDistributionStopping),
        hooks.OnDistributionStopping
    );
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn WSLPluginAPIV1_EntryPoint(
    api: *const WSLPluginAPIV1,
    hooks: *mut WSLPluginHooksV1,
) -> HRESULT {
    let api_ref: &'static WSLPluginAPIV1;
    let hooks_ref: &mut WSLPluginHooksV1;

    unsafe {
        api_ref = &*api;
        hooks_ref = &mut *hooks;
    };

    create_plugin(api_ref, hooks_ref).into()
}
