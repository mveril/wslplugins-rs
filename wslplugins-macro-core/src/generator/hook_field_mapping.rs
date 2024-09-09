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

fn generate_hook_fns(hooks: &[Hooks]) -> Result<Vec<TokenStream>> {
    hooks
        .iter()
        .map(|&mapping| {
            let ts = c_funcs_tokens::get_c_func_tokens(mapping)?.expect(&format!(
                "{:?} does not match with predefined C hook value",
                mapping
            ));
            Ok(ts)
        })
        .collect::<Result<Vec<TokenStream>>>()
}

fn create_static_type(imp: &ParsedImpl) -> Result<Type> {
    let mut static_type = imp.target_type.as_ref().clone();
    if let Some(lifetime) = utils::get_path_lifetime(&imp.trait_) {
        utils::replace_lifetime_in_type(
            &mut static_type,
            lifetime,
            &Lifetime::new("'static", Span::call_site()),
        );
    }
    Ok(static_type)
}

// Prepares the hook mappings for the plugin, returns a list of TokenStream
fn prepare_hooks(hook_struct_name: &Ident, hooks: &[Hooks]) -> Result<Vec<TokenStream>> {
    hooks
        .iter()
        .map(|&hook| hook_field_mapping(hook_struct_name, hook))
        .collect()
}

// Maps each hook to its corresponding field in the hooks structure
fn hook_field_mapping(hooks_struct_name: &Ident, hook: Hooks) -> Result<TokenStream> {
    let field: Ident = parse_str(&hook.get_hook_field_name())?;
    let func: Ident = parse_str(&hook.get_c_method_name())?;
    Ok(quote! {
        #hooks_struct_name.#field = Some(#func);
    })
}

// Generates the plugin entry function with hook management
fn generate_entry_point(imp: &ParsedImpl, version: &RequiredVersion) -> Result<TokenStream> {
    let static_plugin_type = create_static_type(&imp)?;
    let hooks_ref_name = format_ident!("hooks_ref");
    let hook_set = prepare_hooks(&hooks_ref_name, &imp.hooks)?;
    let RequiredVersion {
        major,
        minor,
        revision,
    } = version;

    Ok(quote! {
        static PLUGIN: ::std::sync::OnceLock<#static_plugin_type> = ::std::sync::OnceLock::new();
        #[no_mangle]
        pub extern "C" fn WSLPluginAPIV1_EntryPoint(
            api: *const ::wslplugins_rs::sys::WSLPluginAPIV1,
            hooks: *mut ::wslplugins_rs::sys::WSLPluginHooksV1,
        ) -> ::windows::core::HRESULT {
            unsafe {
                let api_ref: &'static ::wslplugins_rs::sys::WSLPluginAPIV1 = &*api;
                let #hooks_ref_name: &mut ::wslplugins_rs::sys::WSLPluginHooksV1 = &mut *hooks;
                create_plugin(api_ref, #hooks_ref_name).into()
            }
        }

        fn create_plugin(
            api: &'static ::wslplugins_rs::sys::WSLPluginAPIV1,
            hooks_ref: &mut ::wslplugins_rs::sys::WSLPluginHooksV1,
        ) -> ::windows::core::Result<()> {
            let plugin: #static_plugin_type = create_plugin_with_required_version(api, #major, #minor, #revision)?;
            #(#hook_set)*
            PLUGIN.set(plugin).map_err(|_| ::windows::core::Error::from(::windows::Win32::Foundation::E_ABORT))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::{format_ident, ToTokens};
    use syn::{parse_quote, Type};

    #[test]
    fn test_create_static_type() {
        let imp = ParsedImpl {
            target_type: parse_quote! { SomeType<'a> },
            trait_: parse_quote! { SomeTrait<'a> },
            hooks: Box::new([]),
        };
        let result = create_static_type(&imp);
        assert!(result.is_ok());
        let expected_output: Type = parse_quote! { SomeType<'static> };
        assert_eq!(
            result.unwrap().to_token_stream().to_string(),
            expected_output.to_token_stream().to_string()
        );
    }

    // Test de la fonction hook_field_mapping
    #[test]
    fn test_hook_field_mapping() {
        let hook = Hooks::OnVMStarted;
        let hooks_struct_name = format_ident!("hooks_struct");
        let result = hook_field_mapping(&hooks_struct_name, hook);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string(),
            quote!(hooks_struct.OnVMStarted = Some(on_vm_started);).to_string()
        )
    }

    // Test de la fonction prepare_hooks
    #[test]
    fn test_prepare_hooks() {
        let hooks = vec![Hooks::OnVMStarted];
        let result = prepare_hooks(&format_ident!("hooks_ref"), &hooks);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

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
