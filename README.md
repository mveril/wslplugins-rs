# WSLPlugins-rs

[![Crates.io](https://img.shields.io/crates/v/wslplugins-rs?logo=rust)](https://crates.io/crates/wslplugins-rs)
[![Docs.rs](https://img.shields.io/badge/docs.rs-wslplugins--rs-blue?logo=docs.rs)](https://docs.rs/wslplugins-rs)
[![Book](https://img.shields.io/badge/book-mdBook-blue?logo=mdbook)](https://mveril.github.io/wslplugins-rs/)
[![Build Status](https://github.com/mveril/wslplugins-rs/actions/workflows/rust.yml/badge.svg?logo=github)](https://github.com/mveril/wslplugins-rs/actions)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue?logo=windows&logoColor=white)](#)

WSLPlugins-rs is a Rust framework for building [WSL plugins](https://learn.microsoft.com/windows/wsl/wsl-plugins). It wraps the raw WSL plugin API with safer, more idiomatic Rust types and provides a procedural macro for generating plugin entry points and hook wiring.
The project is intended for Windows hosts that load plugins through WSL. It includes:

- a runtime crate: `wslplugins-rs`
- a proc-macro crate: `wslplugins-macro`
- examples that build real plugin DLLs

## Features

- Safe and ergonomic wrappers around the WSL plugin API
- A `#[wsl_plugin_v1(...)]` macro that generates the exported entry points
- Support for WSL metadata, session information, and command execution
- Examples that can be built, signed, and loaded into WSL

## Prerequisites

Install the following tools on Windows:

- Rust stable and Cargo
- PowerShell
- OpenSSL for certificate generation in the signing script
- `SignTool.exe` from the Windows SDK

Notes:

- `SignTool.exe` is easiest to access from a Visual Studio Developer Command Prompt or a shell where the Windows SDK tools are on `PATH`.
- `sign-plugin.ps1` requires an elevated PowerShell session.
- Running the signing step with `-Trust` installs the generated certificate into the local machine trusted root store.

## Workspace Overview

The workspace is organized around a small public API surface and separate macro implementation crates:

- `wslplugins-rs`: the main framework crate, with safe wrappers around the WSL plugin API, shared plugin context, typed identifiers, and plugin traits.
- `wslplugins-macro`: the procedural macro crate re-exported by `wslplugins-rs` when the `macro` feature is enabled.
- `wslplugins-macro-core`: the internal parsing and code generation implementation used by the procedural macro.
- `wslplugins-macro-tests`: compile-time tests for macro-generated plugin code.

This split keeps plugin authors focused on `wslplugins-rs`, while the macro parsing and generated WSL entry-point wiring stay isolated in internal crates.

## Documentation

- [The wslplugins-rs Book](https://mveril.github.io/wslplugins-rs/) explains the development flow from first plugin to packaging.
- [docs.rs](https://docs.rs/wslplugins-rs) contains the generated API reference.

To build the book locally, install [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html)
with Cargo and run:

```powershell
cargo install mdbook
mdbook build book
```

## Quick Start

Add the crate with the `macro` feature:

```toml
[dependencies]
wslplugins-rs = { version = "0.1.0-beta.4", features = ["macro"] }
```

Then implement a plugin:

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

The `macro` feature re-exports the `wsl_plugin_v1` attribute and generates the WSL entry points for a `WSLPluginV1` implementation.

### Choosing API Requirements

The macro argument controls the WSL Plugin API support checked before your plugin is initialized.
Use no argument when the plugin only needs the base entry point:

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

Use `#[wsl_plugin_v1(major, minor)]` or `#[wsl_plugin_v1(major, minor, revision)]` when the whole
plugin requires a known API version before it can run.

For plugins that require named API capabilities, pass one or more
`WSLVersionCapability` values:

```rust
use wslplugins_rs::prelude::*;

pub(crate) struct RegistrationLogger {
    context: &'static WSLContext,
}

#[wsl_plugin_v1(
    WSLVersionCapability::DistributionRegisteredHook
    | WSLVersionCapability::DistributionUnregisteredHook
)]
impl WSLPluginV1 for RegistrationLogger {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }
}
```

Hooks introduced after the base API, such as distribution registration and
unregistration notifications, are wired only when the host API version supports
the corresponding capability. See the book's
[Version Capabilities](https://mveril.github.io/wslplugins-rs/version-capabilities.html) chapter
for the capability list and cross-references to command execution and plugin events.

## Running Commands in WSL

Use `ApiV1::new_command` to build and execute a Linux command from a plugin session.

```rust
use wslplugins_rs::api::{ApiV1, WSLCommandExecution};
use wslplugins_rs::SessionID;

fn run_version(api: &ApiV1) -> Result<(), Box<dyn std::error::Error>> {
    let stream = api
        .new_command(SessionID::from(0), "/bin/cat")
        .with_arg("/proc/version")
        .execute()?;

    drop(stream);
    Ok(())
}
```

Notes:

- Program paths must be Linux UTF-8 paths such as `/bin/echo`
- `argv[0]` defaults to the program path and can be overridden with `with_arg0`
- `with_distribution_id` targets a specific user distribution and requires the
  `ExecuteBinaryInDistribution` capability
- `execute()` returns a `TcpStream` connected to process stdin/stdout
- stderr is forwarded to Linux `dmesg`

## Examples

Example plugins are included:

- `examples/minimal`: a close Rust translation of Microsoft's sample plugin
- `examples/dist-info`: a plugin focused on distribution metadata and tracing
- `examples/unpackaged-distro-blacklist-policy`: a policy plugin that blocks unpackaged
  distributions
- `examples/read-only-host-mount`: mounts a Windows directory read-only inside the WSL VM

Build one of them in release mode:

```powershell
cargo build --release -p minimal
```

or:

```powershell
cargo build --release -p dist-info
```

The resulting plugin DLLs are produced in `target\release\`.

## Signing a Plugin

Sign the built DLL with the provided PowerShell script:

```powershell
.\sign-plugin.ps1 -PluginPath .\target\release\minimal.dll -Trust
```

or:

```powershell
.\sign-plugin.ps1 -PluginPath .\target\release\dist_info.dll -Trust
```

If you do not want to install the certificate automatically, omit `-Trust`.

Microsoft's WSL plugin documentation also requires Windows test signing for test-signed plugin DLLs. If WSL rejects a locally signed plugin with `TRUST_E_NOSIGNATURE`, enable test signing on the test machine and reboot if required:

```powershell
Bcdedit.exe -set TESTSIGNING ON
```

## Registering the Plugin in WSL

Register the signed DLL in the WSL plugins registry key:

```cmd
reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v minimal /d "C:\path\to\wslplugins-rs\target\release\minimal.dll" /t REG_SZ
```

Adjust the registry value name and DLL path for the plugin you want to load.

Restart the WSL service after registration, then run a WSL command to load the plugin:

```powershell
Stop-Service -Name "wslservice" -Force
wsl.exe echo "test"
```

## Verification

After loading the plugin:

- inspect the log output produced by the example
- verify the DLL path in the registry
- confirm the DLL was signed successfully

The `minimal` example writes to `C:\wsl-plugin-demo.txt`.

## Release Checks

The repository release workflow is centered on these commands:

```powershell
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all -- --check
cargo publish --workspace --dry-run
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for branch, validation, and pull request guidance.

Please report security issues privately. See [SECURITY.md](SECURITY.md).

## License

Licensed under either MIT or Apache-2.0.
