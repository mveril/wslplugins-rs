use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Ident, Result};

use crate::hooks::Hooks;

pub(super) fn get_c_func_tokens(hook: Hooks) -> Result<Option<TokenStream>> {
    let c_method_ident: Ident = parse_str(hook.get_c_method_name().as_str())?;
    let trait_method_ident: Ident = parse_str(&hook.get_trait_method_name())?;

    let ok_result = match hook {
        Hooks::OnVMStarted => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                settings: *const ::wslplugins_rs::sys::WSLVmCreationSettings,
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let settings_ptr = unsafe { &*settings };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(
                        &::wslplugins_rs::WSLSessionInformation::from(session_ptr),
                        &::wslplugins_rs::WSLVmCreationSettings::from(settings_ptr),
                    ).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
        Hooks::OnVMStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(&::wslplugins_rs::WSLSessionInformation::from(session_ptr)).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
        Hooks::OnDistributionStarted => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(
                        &::wslplugins_rs::WSLSessionInformation::from(session_ptr),
                        &::wslplugins_rs::DistributionInformation::from(distribution_ptr),
                    ).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
        Hooks::OnDistributionStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(
                        &::wslplugins_rs::WSLSessionInformation::from(session_ptr),
                        &::wslplugins_rs::DistributionInformation::from(distribution_ptr),
                    ).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
        Hooks::OnDistributionRegistered => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution:  *const ::wslplugins_rs::sys::WSLOfflineDistributionInformation,
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(
                        &::wslplugins_rs::WSLSessionInformation::from(session_ptr),
                        &::wslplugins_rs::OfflineDistributionInformation::from(distribution_ptr),
                    ).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
        Hooks::OnDistributionUnregistered => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution:  *const ::wslplugins_rs::sys::WSLOfflineDistributionInformation,
            ) -> ::windows::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                if let Some(plugin) = PLUGIN.get() {
                    plugin.#trait_method_ident(
                        &::wslplugins_rs::WSLSessionInformation::from(session_ptr),
                        &::wslplugins_rs::OfflineDistributionInformation::from(distribution_ptr),
                    ).into()
                } else {
                    ::windows::Win32::Foundation::E_FAIL
                }
            }
        }),
    };

    Ok(ok_result)
}
