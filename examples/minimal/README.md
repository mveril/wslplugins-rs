# Minimal example

This example is a close Rust translation of [Microsoft's sample C++ WSL plugin](https://github.com/microsoft/wsl-plugin-sample/blob/main/plugin.cpp).

It keeps the same observable behavior as the original sample:
- it opens `C:\wsl-plugin-demo.txt` when the plugin is loaded
- it logs VM and distribution lifecycle events
- it runs `/bin/cat /proc/version` when the VM starts
- it handles the same registration lifecycle events as the sample plugin

The remaining differences stem from the framework architecture and idiomatic Rust:
- `#[wsl_plugin_v1(...)]` generates the entry point and hook registration using the [`wslplugins_rs::wsl_plugin_v1`] macro
- [`wslplugins_rs::plugin::WSLPluginV1`] stores plugin state in a Rust struct instead of global variables
- `ApiV1::new_command(...).execute()` replaces the manual `ExecuteBinary` and socket handling using the [`wslplugins_rs::api::WSLCommand`] API

New plugins should prefer named `WSLVersionCapability` requirements where available. The book's
version capabilities chapter lists the capabilities for registration hooks, distribution-scoped
command execution, and version-gated distribution fields.
