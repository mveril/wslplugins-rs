use proc_macro::TokenStream;
use wslplugins_macro_core;

#[proc_macro_attribute]
pub fn wsl_plugin_v1(attr: TokenStream, item: TokenStream) -> TokenStream {
    wslplugins_macro_core::wsl_plugin_v1(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
