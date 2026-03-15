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
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let settings_ptr = unsafe { &*settings };
                PLUGIN.get().map(|plugin|{
                    let result = plugin.#trait_method_ident(
                        session_ptr.as_ref(),
                        settings_ptr.as_ref(),
                    );
                    ::wslplugins_rs::windows_core::HRESULT::from(::wslplugins_rs::plugin::utils::consume_to_win_result(result)).0
                }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
            }
        }),
        Hooks::OnVMStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                let session_ptr = unsafe { &*session };
                PLUGIN.get()
                    .map(|plugin| {
                        let result = plugin.#trait_method_ident(session_ptr.as_ref());
                        ::wslplugins_rs::windows_core::HRESULT::from(result).0
                    })
                    .unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
            }
        }),
        Hooks::OnDistributionStarted => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                PLUGIN.get().map(|plugin|{
                    let result = plugin.#trait_method_ident(
                        session_ptr.as_ref(),
                        distribution_ptr.as_ref(),
                    );
                    ::wslplugins_rs::windows_core::HRESULT::from(::wslplugins_rs::plugin::utils::consume_to_win_result(result)).0
                }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
            }
        }),
        Hooks::OnDistributionStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                PLUGIN.get().map(|plugin|{
                    ::wslplugins_rs::windows_core::HRESULT::from(plugin.#trait_method_ident(
                        session_ptr.as_ref(),
                        distribution_ptr.as_ref(),
                    )).0
                }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
            }
        }),
        Hooks::OnDistributionRegistered | Hooks::OnDistributionUnregistered => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution:  *const ::wslplugins_rs::sys::WslOfflineDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                let session_ptr = unsafe { &*session };
                let distribution_ptr = unsafe { &*distribution };
                PLUGIN.get().map(|plugin|{
                    ::wslplugins_rs::windows_core::HRESULT::from(plugin.#trait_method_ident(
                        session_ptr.as_ref(),
                        distribution_ptr.as_ref(),
                    )).0
                }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
            }
        }),
    };

    Ok(ok_result)
}
