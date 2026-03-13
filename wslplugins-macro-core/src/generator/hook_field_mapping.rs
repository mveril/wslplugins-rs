use super::utils;
use crate::{
    generator::c_funcs_tokens,
    hooks::Hooks,
    parser::{ParsedImpl, RequiredVersion},
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_str, Ident, Lifetime, Result, Type};

// Main function to generate the complete TokenStream for the plugin
pub fn generate(imp: &ParsedImpl, version: &RequiredVersion) -> Result<TokenStream> {
    let entry_point: TokenStream = generate_entry_point(imp, version)?;
    let hooks_funcs = generate_hook_fns(imp.hooks.as_ref())?;
    Ok(quote! {
        #entry_point
        #(#hooks_funcs)*
    })
}

fn version_gated_hooks(hook: Hooks) -> bool {
    matches!(
        hook,
        Hooks::OnDistributionRegistered | Hooks::OnDistributionUnregistered
    )
}

// Generate hook function implementations based on provided hook mappings
fn generate_hook_fns(hooks: &[Hooks]) -> Result<Vec<TokenStream>> {
    hooks
        .iter()
        .map(|&mapping| {
            let ts = c_funcs_tokens::get_c_func_tokens(mapping)?.unwrap_or_else(|| {
                panic!("{mapping:?} does not match with predefined C hook value")
            });
            Ok(ts)
        })
        .collect::<Result<Vec<TokenStream>>>()
}

// Create a static version of the type for plugin management
fn create_static_type(imp: &ParsedImpl) -> Type {
    let mut static_type = imp.target_type.as_ref().clone();
    if let Some(lifetime) = utils::get_path_lifetime(&imp.trait_) {
        utils::replace_lifetime_in_type(
            &mut static_type,
            lifetime,
            &Lifetime::new("'static", Span::call_site()),
        );
    }
    static_type
}

// Prepare hooks by mapping them to their respective fields in the hook structure
fn prepare_hooks(hook_struct_name: &Ident, hooks: &[Hooks]) -> Result<Vec<TokenStream>> {
    hooks
        .iter()
        .map(|&hook| hook_field_mapping(hook_struct_name, hook))
        .collect()
}

// Map each hook to its corresponding field in the hooks structure
fn hook_field_mapping(hooks_struct_name: &Ident, hook: Hooks) -> Result<TokenStream> {
    let field_str = hook.get_hook_field_name();
    let field: Ident = parse_str(&field_str)?;
    let func: Ident = parse_str(&hook.get_c_method_name())?;
    let base = quote! {
        #hooks_struct_name.#field = Some(#func);
    };
    let result = match hook {
        Hooks::OnDistributionRegistered | Hooks::OnDistributionUnregistered => {
            let required_version = "2.1.2";
            quote! {
                if current_version >= ::wslplugins_rs::WSLVersion::new(2, 1, 2) {
                    #base
                } else {
                    ::wslplugins_rs::__private::debug!(
                        "Hook {} not applied due to insufficient version (found: {}, required: {})",
                        #field_str,
                        current_version,
                        #required_version
                    );
                }
            }
        }
        _ => base,
    };
    Ok(result)
}

// Generate the plugin entry function with hook management and initialization
fn generate_entry_point(imp: &ParsedImpl, version: &RequiredVersion) -> Result<TokenStream> {
    let static_plugin_type = create_static_type(imp);
    let hooks_ref_name = format_ident!("hooks_ref");
    let current_version = imp
        .hooks
        .iter()
        .cloned()
        .any(version_gated_hooks)
        .then_some(quote! {
            let current_version = ::wslplugins_rs::WSLVersion::from(api.Version);
        })
        .unwrap_or_default();
    let hook_set = prepare_hooks(&hooks_ref_name, &imp.hooks)?;
    let RequiredVersion {
        major,
        minor,
        revision,
    } = version;

    Ok(quote! {
        static PLUGIN: ::std::sync::OnceLock<#static_plugin_type> = ::std::sync::OnceLock::new();
        #[no_mangle]
        pub unsafe extern "C" fn WSLPluginAPIV1_EntryPoint(
            api: *const ::wslplugins_rs::sys::WSLPluginAPIV1,
            hooks: *mut ::wslplugins_rs::sys::WSLPluginHooksV1,
        ) -> ::wslplugins_rs::windows_core::HRESULT {
            unsafe {
                let api_ref: &'static ::wslplugins_rs::sys::WSLPluginAPIV1 = unsafe { &*api};
                let #hooks_ref_name: &mut ::wslplugins_rs::sys::WSLPluginHooksV1 = unsafe{ &mut *hooks };
                create_plugin(api_ref, #hooks_ref_name).into()
            }
        }

        fn create_plugin(
            api: &'static ::wslplugins_rs::sys::WSLPluginAPIV1,
            hooks_ref: &mut ::wslplugins_rs::sys::WSLPluginHooksV1,
        ) -> ::wslplugins_rs::windows_core::Result<()> {
            let plugin: #static_plugin_type = ::wslplugins_rs::plugin::create_plugin_with_required_version(api, #major, #minor, #revision)?;
            #current_version
            #(#hook_set)*
            PLUGIN.set(plugin).map_err(|_| ::wslplugins_rs::windows_core::Error::from(::wslplugins_rs::windows_core::HRESULT(::wslplugins_rs::sys::windows_sys::Win32::Foundation::E_ABORT)))
        }
    })
}
// test
#[allow(clippy::expect_used, clippy::unwrap_used, reason = "Tests")]
#[cfg(test)]
mod tests {
    use super::*;

    use quote::{format_ident, ToTokens};
    use syn::{parse_quote, Type};

    // Test for creating a static version of a type
    #[test]
    fn test_create_static_type() {
        let imp = ParsedImpl {
            target_type: parse_quote! { SomeType<'a> },
            trait_: parse_quote! { SomeTrait<'a> },
            hooks: Box::new([]),
        };
        let result = create_static_type(&imp);
        let expected_output: Type = parse_quote! { SomeType<'static> };
        assert_eq!(
            result.to_token_stream().to_string(),
            expected_output.to_token_stream().to_string()
        );
    }

    // Test for hook field mapping
    #[test]
    fn test_hook_field_mapping() {
        let hook = Hooks::OnVMStarted;
        let hooks_struct_name = format_ident!("hooks_struct");
        let result = hook_field_mapping(&hooks_struct_name, hook);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string(),
            quote!(hooks_struct.OnVMStarted = Some(on_vm_started);).to_string()
        );
    }

    // Test for hook field mapping with version condition
    #[test]
    fn test_hook_field_mapping_with_version() {
        let hook = Hooks::OnDistributionRegistered;
        let hooks_struct_name = format_ident!("hooks_struct");
        let result: std::result::Result<TokenStream, syn::Error> =
            hook_field_mapping(&hooks_struct_name, hook);
        assert_eq!(
            result.unwrap().to_string(),
            quote!(
                if current_version >= ::wslplugins_rs::WSLVersion::new(2, 1, 2) {
                    hooks_struct.OnDistributionRegistered = Some(on_distribution_registered);
                } else {
                    ::wslplugins_rs::__private::debug!(
                        "Hook {} not applied due to insufficient version (found: {}, required: {})",
                        "OnDistributionRegistered",
                        current_version,
                        "2.1.2"
                    );
                }
            )
            .to_string()
        );
    }

    // Test for preparing hooks
    #[test]
    fn test_prepare_hooks() {
        let hooks = vec![Hooks::OnVMStarted];
        let result = prepare_hooks(&format_ident!("hooks_ref"), &hooks);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    // Test for generating hook functions
    #[test]
    fn test_generate_hook_fns() {
        let hooks = vec![Hooks::OnVMStarted];
        let result = generate_hook_fns(&hooks);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        let result_str = result.first().to_token_stream().to_string();
        assert!(result_str.contains("extern \"C\" fn on_vm_started"));
    }
}
