#![allow(missing_docs)]
#![allow(clippy::missing_inline_in_public_items, reason = "Macros")]
use proc_macro::TokenStream;
/// Attribute macro for WSL plugin V1.
/// This macro should be used on impl block of `WSLPluginV1` in order to register the plugin that implement this interface
/// # Example
/// ``` rust, ignore
/// #[wsl_plugin_v1]
/// impl WSLPluginV1 for MyPlugin {
///     // Implementation details
/// }
/// ```
#[proc_macro_attribute]
pub fn wsl_plugin_v1(attr: TokenStream, item: TokenStream) -> TokenStream {
    wslplugins_macro_core::wsl_plugin_v1(attr.into(), &item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
