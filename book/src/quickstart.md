# End-to-End Quickstart

This chapter shows the shortest complete path from a new Rust crate to a WSL plugin DLL that is
signed, registered, loaded, tested, and removed again.

Run the deployment commands from an elevated PowerShell session because signing trust, registry
registration, and WSL service restarts affect the host machine.

## Create the Crate

```powershell
cargo new my-wsl-plugin --lib
cd my-wsl-plugin
```

Configure the crate as a Windows DLL and add the framework dependency:

```toml
[package]
name = "my-wsl-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wslplugins-rs = { version = "0.1.0-beta.4", features = ["macro"] }
```

Use this minimal `src/lib.rs`:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

pub(crate) struct Plugin {
    _context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { _context: context })
    }
}
```

Run normal Rust validation first:

```powershell
cargo test
cargo clippy --all-targets --all-features
cargo fmt -- --check
```

Then build the DLL:

```powershell
cargo build --release
```

The DLL path uses underscores even when the crate name uses hyphens:

```text
target\release\my_wsl_plugin.dll
```

## Sign and Register

The repository contains a development signing helper. Set variables for the plugin you are testing:

```powershell
$PluginName = "my-wsl-plugin"
$DllPath = "C:\path\to\my-wsl-plugin\target\release\my_wsl_plugin.dll"
$RepoPath = "C:\path\to\wslplugins-rs"
```

Sign the DLL:

```powershell
& "$RepoPath\sign-plugin.ps1" -PluginPath $DllPath
```

For local development, you may also trust the generated certificate on the test machine:

```powershell
& "$RepoPath\sign-plugin.ps1" -PluginPath $DllPath -Trust
```

Register the signed DLL:

```powershell
Set-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" `
    -Name $PluginName `
    -Value $DllPath `
    -Force
```

Restart WSL and trigger plugin loading:

```powershell
Stop-Service -Name "wslservice" -Force
wsl.exe echo "plugin load test"
```

## Verify and Clean Up

Check that the registry value points to the DLL you built:

```powershell
Get-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" `
    -Name $PluginName
```

If WSL reports a plugin load error, use the troubleshooting chapter before changing code.

When finished testing, unregister the plugin and restart WSL:

```powershell
Remove-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" `
    -Name $PluginName `
    -Force

Stop-Service -Name "wslservice" -Force
```

Next: read [Your First Plugin](./first-plugin.md) to add hooks and behavior.
