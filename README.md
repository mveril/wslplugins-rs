# WSLPlugins-rs
WSLPlugins-rs is a Rust framework for building [WSL plugins](https://learn.microsoft.com/windows/wsl/wsl-plugins). It wraps the raw WSL plugin API with safer, more idiomatic Rust types and provides a procedural macro for generating the plugin entry points and hook wiring.
The project is intended for Windows hosts that load plugins through WSL. It includes:

- a runtime crate: `wslplugins-rs`
- a proc-macro crate: `wslplugins-macro`
- examples that build real plugin DLLs

## Features

- Safe and ergonomic wrappers around the WSL plugin API
- A `#[wsl_plugin_v1(...)]` macro to generate the exported entry points
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
- Running the signing step with `-Trust` requires administrator privileges because it installs the generated certificate locally.

## Crates

- `wslplugins-rs`: main framework crate
- `wslplugins-macro`: procedural macro crate re-exported by `wslplugins-rs` when the `macro` feature is enabled

## Quick Start

Add the crate with the `macro` feature:

```toml
[dependencies]
wslplugins-rs = { version = "0.1.0-beta.1", features = ["macro"] }
```

Then implement a plugin:

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

The `macro` feature re-exports the `wsl_plugin_v1` attribute and generates the WSL entry points for a `WSLPluginV1` implementation.

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
- `with_distribution_id` targets a specific user distribution
- `execute()` returns a `TcpStream` connected to process stdin/stdout
- stderr is forwarded to Linux `dmesg`

## Examples

Two example plugins are included:

- `examples/minimal`: a close Rust translation of Microsoft's sample plugin
- `examples/dist-info`: a plugin focused on distribution metadata and tracing

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

## Registering the Plugin in WSL

Register the signed DLL in the WSL plugins registry key:

```cmd
reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v minimal /d "C:\path\to\wslplugins-rs\target\release\minimal.dll" /t REG_SZ
```

Adjust the registry value name and DLL path for the plugin you want to load.

Restart the WSL service after registration:

```cmd
sc.exe stop wslservice
sc.exe start wslservice
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

Contributions are welcome. Open an issue or submit a pull request with tests and a clear description of the change.

## License

Licensed under either MIT or Apache-2.0.
