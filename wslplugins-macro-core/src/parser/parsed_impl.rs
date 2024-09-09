use crate::hooks::Hooks;
use syn::parse::{Parse, ParseStream, Result};
use syn::{ImplItem, ItemImpl};
use syn::{Path, Type};

#[derive(Debug)]
pub struct ParsedImpl {
    pub target_type: Box<Type>,
    pub hooks: Box<[Hooks]>,
    pub trait_: Path,
}

impl Parse for ParsedImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin_impl: ItemImpl = input.parse()?;
        let p =
            plugin_impl
                .trait_
                .as_ref()
                .map(|(_, path, _)| path)
                .ok_or(syn::Error::new_spanned(
                    plugin_impl.impl_token,
                    "Expected a trait.",
                ))?;
        let hook_vec: Vec<Hooks> = plugin_impl
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(func) => Hooks::from_trait_method_name(&func.sig.ident.to_string()),
                _ => None,
            })
            .collect();

        Ok(ParsedImpl {
            target_type: plugin_impl.self_ty.clone(),
            hooks: hook_vec.into_boxed_slice(),
            trait_: p.clone(),
        })
    }
}

#[cfg(test)]
mod tests {

    use quote::format_ident;

    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_parsed_impl_with_try_new() {
        let my_trait = syn::parse_str::<Path>("WSLPluginV1").expect("Invalid trait name");
        let vm_started_method = format_ident!("on_vm_started");
        let impl_block = quote! {
            impl #my_trait for Plugin {
                fn try_new(api: ApiV1) -> Result<Self> {
                    Ok(Self {})
                }
                fn #vm_started_method(&self, session: &WSLSessionInformation) -> Result<()> {
                    Ok(())
                }
            }
        };

        let parsed_impl: Result<ParsedImpl> = parse2(impl_block);
        assert!(parsed_impl.is_ok());
        let parsed_impl = parsed_impl.unwrap();
        let expected_hook = vec![Hooks::OnVMStarted];
        let parsed_hooks = parsed_impl.hooks.to_vec();

        assert_eq!(parsed_hooks, expected_hook);
    }

    #[test]
    fn test_parsed_impl_no_trait_specified() {
        let impl_block = quote! {
            impl Plugin {
                fn try_new(api: ApiV1) -> Result<Self> {
                    Ok(Self {})
                }
                fn on_vm_started(&self, session: &WSLSessionInformation) -> Result<()> {
                    Ok(())
                }
            }
        };

        let parsed_impl: Result<ParsedImpl> = parse2(impl_block);
        assert!(parsed_impl.is_err());
    }
}
