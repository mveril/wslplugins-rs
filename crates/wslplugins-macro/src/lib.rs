#![allow(missing_docs)]
#![allow(clippy::missing_inline_in_public_items, reason = "Macros")]
//! Provides procedural macros for WSL plugin development.
use proc_macro::TokenStream;
/// Attribute macro for WSL plugin V1.
///
/// This macro should be used on an `impl WSLPluginV1` block to register a
/// plugin implementation and generate the WSL Plugin API entry point.
///
/// The attribute accepts one of these required version forms:
///
/// - `#[wsl_plugin_v1]`, which does not require a specific minimum WSL Plugin
///   API version beyond the entry point being available.
/// - `#[wsl_plugin_v1(major, minor)]`, which requires `major.minor.0`.
/// - `#[wsl_plugin_v1(major, minor, revision)]`, which requires the exact
///   `major.minor.revision` minimum version.
/// - `#[wsl_plugin_v1(capability)]` or
///   `#[wsl_plugin_v1(capability_a | capability_b)]`, which requires the
///   minimum version for the listed
///   [`WSLVersionCapability`](https://docs.rs/wslplugins-rs/latest/wslplugins_rs/enum.WSLVersionCapability.html)
///   value or values.
///
/// # Minimum version example
/// ```rust, ignore
/// #[wsl_plugin_v1]
/// impl WSLPluginV1 for MyPlugin {
///     // Implementation details
/// }
/// ```
///
/// # Explicit version example
/// ```rust, ignore
/// #[wsl_plugin_v1(2, 1, 3)]
/// impl WSLPluginV1 for MyPlugin {
///     // Implementation details
/// }
/// ```
///
/// # Capability example
///
/// This example uses
/// [`WSLVersionCapability::DistributionRegisteredHook`](https://docs.rs/wslplugins-rs/latest/wslplugins_rs/enum.WSLVersionCapability.html#variant.DistributionRegisteredHook)
/// (`2.1.2`) and
/// [`WSLVersionCapability::DistributionUnregisteredHook`](https://docs.rs/wslplugins-rs/latest/wslplugins_rs/enum.WSLVersionCapability.html#variant.DistributionUnregisteredHook)
/// (`2.1.2`).
///
/// ```rust, ignore
/// #[wsl_plugin_v1(
///     WSLVersionCapability::DistributionRegisteredHook
///     | WSLVersionCapability::DistributionUnregisteredHook
/// )]
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
