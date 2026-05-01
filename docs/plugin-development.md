# Building Plugins with WSLPlugins-rs

WSLPlugins-rs helps plugin authors build WSL plugin DLLs in Rust.

This page is for users of the library. If you want to contribute to this repository, see [CONTRIBUTING.md](../CONTRIBUTING.md).

## Dependency

Most plugin crates should depend on `wslplugins-rs` with the `macro` feature enabled.

```toml
[dependencies]
wslplugins-rs = { version = "0.1.0-beta.3", features = ["macro"] }
```

The `macro` feature provides the `#[wsl_plugin_v1(...)]` attribute used to generate the WSL plugin entry points.

## Minimal Shape

Implement `WSLPluginV1` for your plugin type, then annotate the implementation with the WSL API version your plugin targets.

```rust
use wslplugins_rs::prelude::*;

pub(crate) struct MyPlugin {
	context: &'static WSLContext,
}

#[wsl_plugin_v1(2, 0, 5)]
impl WSLPluginV1 for MyPlugin {
	fn try_new(context: &'static WSLContext) -> WinResult<Self> {
		Ok(Self { context })
	}
}
```

The macro generates the exported functions expected by WSL.

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
