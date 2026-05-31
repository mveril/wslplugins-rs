# Building Plugins with WSLPlugins-rs

WSLPlugins-rs helps plugin authors build WSL plugin DLLs in Rust.

This page is for users of the library. If you want to contribute to this repository, see [CONTRIBUTING.md](../CONTRIBUTING.md).

## Dependency

Most plugin crates should depend on `wslplugins-rs` with the `macro` feature enabled.

```toml
[dependencies]
wslplugins-rs = { version = "0.1.0-beta.4", features = ["macro"] }
```

The `macro` feature provides the `#[wsl_plugin_v1]` attribute used to generate the WSL plugin entry points.

## Minimal Shape

Implement `WSLPluginV1` for your plugin type, then annotate the implementation.

```rust
use wslplugins_rs::prelude::*;

pub(crate) struct MyPlugin {
	context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

The macro generates the exported functions expected by WSL, initializes the shared `WSLContext`, creates one plugin instance by calling `try_new`, and wires implemented hook methods into the WSL hook table.

## API Requirements

The macro can also check that the host WSL Plugin API supports the version or capability required
before the plugin is initialized.

Use no argument when the plugin only needs the base entry point:

```rust
#[wsl_plugin_v1]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

Use an explicit version when the whole plugin depends on a specific minimum API version:

```rust
#[wsl_plugin_v1(2, 1)]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

```rust
#[wsl_plugin_v1(2, 1, 2)]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

Use named capabilities when the plugin depends on specific API features:

```rust
#[wsl_plugin_v1(WSLVersionCapability::DistributionRegisteredHook)]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

Multiple capabilities can be combined with `|`. The macro requires the highest WSL version needed by the listed capabilities:

```rust
#[wsl_plugin_v1(
	WSLVersionCapability::DistributionRegisteredHook
	| WSLVersionCapability::DistributionUnregisteredHook
)]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

If the version check fails, WSL receives `WSL_E_PLUGIN_REQUIRES_UPDATE` and the plugin is not initialized.

The complete capability list is maintained in the book's
[Version Capabilities](../book/src/version-capabilities.md) chapter.

## Version-Gated Hooks

Some hook fields are available only on newer WSL Plugin API versions. For example, distribution registration and unregistration hooks require the matching `WSLVersionCapability`.

When a plugin implements one of these hooks, the generated entry point checks the host API version before writing that hook into the hook table. On older hosts, the plugin can still initialize, but the unsupported hook is skipped.

If the plugin cannot operate correctly without those hooks, list the capability in `#[wsl_plugin_v1(...)]` so initialization fails on unsupported hosts.

## Reliability

WSL loads plugin code into the WSL service process. If a plugin crashes, it can crash the WSL service.

For that reason:

- Prefer returning errors over panicking.
- Avoid `panic!`, `unwrap`, and `expect` in plugin code.
- Keep unsafe code narrow and document why each unsafe block is valid.
- Be careful with blocking work inside plugin hooks.

## Examples

Start with the included examples:

- `examples/minimal`: a close Rust translation of Microsoft's sample plugin.
- `examples/dist-info`: an example focused on distribution metadata and tracing.
- `examples/unpackaged-distro-blacklist-policy`: an example policy plugin.

See [signing.md](signing.md) for local DLL signing and [windows-validation.md](windows-validation.md) for testing a plugin on a Windows host.
