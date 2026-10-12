use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Ident, Result};

use crate::hooks::Hooks;

#[expect(
    clippy::too_many_lines,
    reason = "Each supported hook needs a distinct generated C ABI signature"
)]
pub(super) fn get_c_func_tokens(hook: Hooks) -> Result<Option<TokenStream>> {
    let c_method_ident: Ident = parse_str(hook.get_c_method_name().as_str())?;
    let trait_method_ident: Ident = parse_str(&hook.get_trait_method_name())?;

    let ok_result = match hook {
        Hooks::OnVMStarted => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                settings: *const ::wslplugins_rs::sys::WSLVmCreationSettings,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                if session.is_null() || settings.is_null() {
                    return ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_POINTER;
                }
                let result = ::wslplugins_rs::__private::catch_unwind_plugin(|| {
                    PLUGIN.get().map_or_else(
                        || Err(::wslplugins_rs::plugin::Error::with_code(
                            ::wslplugins_rs::windows_core::HRESULT(
                                ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL,
                            ),
                        )),
                        |plugin| {
                        // SAFETY: WSL provides a non-null baseline session valid during this callback.
                        let session_ptr = unsafe { &*session };
                        // SAFETY: WSL provides non-null baseline settings valid during this callback.
                        let settings_ptr = unsafe { &*settings };
                            plugin.#trait_method_ident(
                                session_ptr.as_ref(),
                                settings_ptr.as_ref(),
                            )
                        },
                    )
                });
                ::wslplugins_rs::windows_core::HRESULT::from(
                    ::wslplugins_rs::plugin::utils::consume_to_win_result(result),
                ).0
            }
        }),
        Hooks::OnVMStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                if session.is_null() {
                    return ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_POINTER;
                }
                ::wslplugins_rs::__private::catch_unwind_or(
                    ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL,
                    || {
                        // SAFETY: WSL provides a non-null baseline session valid during this callback.
                        let session_ptr = unsafe { &*session };
                        PLUGIN.get()
                            .map(|plugin| {
                                let result = plugin.#trait_method_ident(session_ptr.as_ref());
                                ::wslplugins_rs::windows_core::HRESULT::from(result).0
                            })
                            .unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
                    },
                )
            }
        }),
        Hooks::OnDistributionStarted => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                if session.is_null() || distribution.is_null() {
                    return ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_POINTER;
                }
                let result = ::wslplugins_rs::__private::catch_unwind_plugin(|| {
                    PLUGIN.get().map_or_else(
                        || Err(::wslplugins_rs::plugin::Error::with_code(
                            ::wslplugins_rs::windows_core::HRESULT(
                                ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL,
                            ),
                        )),
                        |plugin| {
                        // SAFETY: WSL provides a non-null baseline session valid during this callback.
                        let session_ptr = unsafe { &*session };
                        // SAFETY: WSL provides this versioned allocation for the callback duration.
                        let distribution_ptr = unsafe { ::wslplugins_rs::WSLDistributionInformation::from_raw(distribution) };
                            plugin.#trait_method_ident(
                                session_ptr.as_ref(),
                                distribution_ptr,
                            )
                        },
                    )
                });
                ::wslplugins_rs::windows_core::HRESULT::from(
                    ::wslplugins_rs::plugin::utils::consume_to_win_result(result),
                ).0
            }
        }),
        Hooks::OnDistributionStopping => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution: *const ::wslplugins_rs::sys::WSLDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                if session.is_null() || distribution.is_null() {
                    return ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_POINTER;
                }
                ::wslplugins_rs::__private::catch_unwind_or(
                    ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL,
                    || {
                        // SAFETY: WSL provides a non-null baseline session valid during this callback.
                        let session_ptr = unsafe { &*session };
                        // SAFETY: WSL provides this versioned allocation for the callback duration.
                        let distribution_ptr = unsafe { ::wslplugins_rs::WSLDistributionInformation::from_raw(distribution) };
                        PLUGIN.get().map(|plugin|{
                            ::wslplugins_rs::windows_core::HRESULT::from(plugin.#trait_method_ident(
                                session_ptr.as_ref(),
                                distribution_ptr,
                            )).0
                        }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
                    },
                )
            }
        }),
        Hooks::OnDistributionRegistered | Hooks::OnDistributionUnregistered => Some(quote! {
            extern "C" fn #c_method_ident(
                session: *const ::wslplugins_rs::sys::WSLSessionInformation,
                distribution:  *const ::wslplugins_rs::sys::WslOfflineDistributionInformation,
            ) -> ::wslplugins_rs::sys::windows_sys::core::HRESULT {
                if session.is_null() || distribution.is_null() {
                    return ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_POINTER;
                }
                ::wslplugins_rs::__private::catch_unwind_or(
                    ::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL,
                    || {
                        // SAFETY: WSL provides a non-null baseline session valid during this callback.
                        let session_ptr = unsafe { &*session };
                        // SAFETY: WSL provides this versioned allocation for the callback duration.
                        let distribution_ptr = unsafe { ::wslplugins_rs::WSLOfflineDistributionInformation::from_raw(distribution) };
                        PLUGIN.get().map(|plugin|{
                            ::wslplugins_rs::windows_core::HRESULT::from(plugin.#trait_method_ident(
                                session_ptr.as_ref(),
                                distribution_ptr,
                            )).0
                        }).unwrap_or(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_FAIL)
                    },
                )
            }
        }),
    };

    Ok(ok_result)
}
