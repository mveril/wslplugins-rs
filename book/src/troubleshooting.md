# Troubleshooting

This page maps common WSL plugin failures to the first checks to run. Start with the registered DLL
path, signature, WSL service restart, and API version before changing plugin logic.

## Plugin Does Not Load

Error:

```text
Wsl/Service/CreateInstance/CreateVm/Plugin/ERROR_MOD_NOT_FOUND
```

Likely causes:

- the registry value points to the wrong path;
- the DLL was rebuilt under a different crate name;
- a native dependency of the DLL is missing.

Check the registered path:

```powershell
Get-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins"
```

Check that the DLL exists:

```powershell
Test-Path "C:\path\to\plugin.dll"
```

Remember that Cargo converts hyphens to underscores for library artifact names. A crate named
`my-wsl-plugin` produces `my_wsl_plugin.dll`.

## Signature Is Rejected

Error:

```text
Wsl/Service/CreateInstance/CreateVm/Plugin/TRUST_E_NOSIGNATURE
```

Likely causes:

- the DLL is unsigned;
- the signing certificate is not trusted by the machine;
- Windows test signing is required for the development scenario.

Check the signature:

```powershell
Get-AuthenticodeSignature "C:\path\to\plugin.dll"
```

For local development, sign with a test certificate and trust it on the test machine. If WSL still
rejects the DLL, Microsoft documentation may require test signing:

```powershell
Bcdedit.exe -set TESTSIGNING ON
```

A reboot may be required. If Secure Boot blocks the setting, decide whether disabling Secure Boot is
acceptable for that machine before continuing.

## Hook Does Not Run

Likely causes:

- the DLL was registered after WSL had already loaded plugins;
- the event you are testing does not trigger the hook you implemented;
- the hook requires a newer WSL plugin API version than the host provides.

Restart the service after registration or rebuilds:

```powershell
Stop-Service -Name "wslservice" -Force
```

Then run a command that creates the lifecycle event you need. For VM startup hooks, a simple command
is enough:

```powershell
wsl.exe echo "test"
```

For distribution startup hooks, target a specific distribution:

```powershell
wsl.exe -d Ubuntu -- echo "test"
```

## WSL Reports Plugin Error

Errors under these paths usually mean the plugin returned an error:

```text
Wsl/Service/CreateInstance/CreateVm/Plugin/*
Wsl/Service/CreateInstance/Plugin/*
```

Check which hook can fail in that path:

- `CreateVm/Plugin/*`: entry point initialization or `on_vm_started`;
- `CreateInstance/Plugin/*`: `on_distribution_started`.

Use `PluginResult` with a message for user-facing policy failures. Use ordinary `Result` handling
for I/O, parsing, and WSL API calls instead of panicking.

## Runtime API Is Too Old

If the plugin requests a newer API version than WSL provides, initialization fails with
`WSL_E_PLUGIN_REQUIRES_UPDATE`.

Use the version in `#[wsl_plugin_v1(major, minor, revision)]` as the minimum API version for the
plugin as a whole. Keep it as low as practical, and use runtime `Result` checks for optional fields
or optional behavior.

Common version-sensitive features include:

| Feature | Minimum API version |
| --- | --- |
| `init_pid()` | `2.0.5` |
| Distribution registration hooks | `2.1.2` |
| `ExecuteBinaryInDistribution` | `2.1.2` |
| `flavor()` and `version()` | `2.4.4` |

## Cleanup After a Bad Deployment

Remove the registry value and restart WSL:

```powershell
Remove-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" `
    -Name "my-wsl-plugin" `
    -Force

Stop-Service -Name "wslservice" -Force
```

If WSL still behaves unexpectedly, verify that no other plugin values remain under the plugins key:

```powershell
Get-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins"
```
