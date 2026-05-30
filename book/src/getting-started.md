# Getting Started

This chapter assumes you are creating a plugin crate outside this repository and want to consume
`wslplugins-rs` as a dependency.

If you want the shortest copy-and-run path first, use the
[End-to-End Quickstart](./quickstart.md), then return here for the background details.

## Prerequisites

Install these tools on Windows:

- [Rust stable and Cargo](https://www.rust-lang.org/tools/install).
- PowerShell.
- [`SignTool.exe`](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool) from the
  Windows SDK.
- [WSL installed](https://learn.microsoft.com/en-us/windows/wsl/install) and able to run at least
  one distribution.
- A Windows environment that can build native MSVC Rust targets.

`SignTool.exe` is usually easiest to access from a Visual Studio Developer Command Prompt or from a
shell where the Windows SDK tools are on `PATH`.

Packaging and host validation also require administrator access because local testing can touch the
machine certificate store, the `HKLM` registry hive, and the WSL service.

## Create a Plugin Crate

Create a Rust library crate and configure it to produce a DLL:

```powershell
cargo new my-wsl-plugin --lib
cd my-wsl-plugin
```

```toml
[lib]
crate-type = ["cdylib"]
```

The crate can contain normal Rust modules and dependencies. The important requirement is that the
final artifact is a native Windows DLL that WSL can load.

## Add the Framework

Most plugins should enable the `macro` feature so the crate generates the required WSL exports:

```toml
[dependencies]
wslplugins-rs = { version = "0.1.0-beta.3", features = ["macro"] }
```

Import the prelude in your plugin code:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;
```

## Implement the Plugin

A minimal plugin is a type plus a `WSLPluginV1` implementation:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

pub(crate) struct Plugin {
    context: &'static WSLContext,
}

#[wsl_plugin_v1(2, 0, 5)]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }
}
```

Add hook methods only for the events your plugin handles. The default implementation for each hook
does nothing and returns success.

## Validate During Development

Run ordinary Rust checks while developing:

```powershell
cargo test
cargo clippy --all-targets --all-features
cargo fmt -- --check
```

When you are ready to load the plugin into WSL, build a release DLL:

```powershell
cargo build --release
```

The DLL is written under `target\release\` using your crate name.

## Use the Examples

The repository examples show complete plugins for common scenarios:

- `minimal`: lifecycle hooks and command execution inside a WSL session.
- `dist-info`: distribution metadata access.
- `unpackaged-distro-blacklist-policy`: a policy-style plugin.

Use them as references for plugin behavior, not as required workspace structure.

Next: follow the [End-to-End Quickstart](./quickstart.md) for a complete deployment loop, or read
[Your First Plugin](./first-plugin.md) to add hook behavior.
