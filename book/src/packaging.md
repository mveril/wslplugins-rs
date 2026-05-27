# Packaging and Deployment

WSL loads plugin DLLs from Windows. A development deployment usually has four steps:

1. Build the plugin DLL.
2. Sign the DLL.
3. Register the DLL path in the WSL plugins registry key.
4. Restart the WSL service and trigger plugin loading.

The underlying Microsoft package for the native API is `Microsoft.WSL.PluginApi` on NuGet. Version
`2.4.4` is the current package version checked while writing this page, and it contains the
`WslPluginApi.h` header used to define the native ABI. Rust users normally consume that ABI through
`wslplugins-rs` and `wslpluginapi-sys`; you do not need to add the NuGet package to a Rust plugin
crate unless you are comparing against the C header or writing C/C++ interop code.

## Build a DLL

Build your plugin in release mode when you are ready to deploy it locally:

```powershell
cargo build --release
```

The resulting DLL is written to `target\release\<crate-name>.dll`.

## Sign the DLL

WSL requires a signed DLL. In a real deployment, sign with the certificate and process used for your
environment. During local development, you can use a test certificate and `SignTool.exe`.

The repository includes `sign-plugin.ps1` as a development helper. From an elevated PowerShell
session in a checkout of this repository:

```powershell
.\sign-plugin.ps1 -PluginPath C:\path\to\my-wsl-plugin\target\release\my_wsl_plugin.dll
```

To also trust the generated certificate locally:

```powershell
.\sign-plugin.ps1 -PluginPath C:\path\to\my-wsl-plugin\target\release\my_wsl_plugin.dll -Trust
```

The `-Trust` option imports the generated certificate into the local machine trusted root store.
Do not commit generated certificates, private keys, or signed DLLs.

If WSL rejects a locally signed plugin with `TRUST_E_NOSIGNATURE`, Microsoft documentation may
require Windows test signing on the test machine:

```powershell
Bcdedit.exe -set TESTSIGNING ON
```

A reboot may be required after changing test-signing mode.

If `Bcdedit.exe -set TESTSIGNING ON` fails because the value is protected by Secure Boot policy,
Microsoft's WSL plugin documentation recommends disabling Secure Boot from firmware settings before
trying again. Consider the security impact before doing that on a daily-use machine.

## Register the Plugin

Register the signed DLL under the WSL plugins registry key:

```cmd
reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v my-wsl-plugin /d "C:\path\to\my-wsl-plugin\target\release\my_wsl_plugin.dll" /t REG_SZ
```

Use a value name and DLL path that match the plugin you are testing.

To unregister the plugin after testing, remove the registry value and restart WSL again:

```powershell
Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" -Name "my-wsl-plugin" -Force
Stop-Service -Name "wslservice" -Force
```

## Reload WSL

Restart the WSL service, then run a WSL command:

```powershell
Stop-Service -Name "wslservice" -Force
wsl.exe echo "test"
```

After loading the plugin, verify:

- the DLL path in the registry;
- the signature status;
- the expected plugin logs or side effects;
- Windows validation notes relevant to the change.

Common WSL plugin load failures include:

- `Wsl/Service/CreateInstance/CreateVm/Plugin/ERROR_MOD_NOT_FOUND`: WSL could not load the DLL.
  Check the registered path and any native dependencies.
- `Wsl/Service/CreateInstance/CreateVm/Plugin/TRUST_E_NOSIGNATURE`: the DLL is unsigned or the
  signing certificate is not trusted by the machine.
- `Wsl/Service/CreateInstance/CreateVm/Plugin/*`: the plugin returned an error from the entry point
  or `OnVMStarted`.
- `Wsl/Service/CreateInstance/Plugin/*`: the plugin returned an error from
  `OnDistributionStarted`.

Because plugins run in the WSL service process, treat host validation as a machine-affecting test:
keep a note of the registry value you added, the certificate you trusted, and the service restart
commands you ran.

## Sources

- Microsoft WSL plugin documentation:
  <https://learn.microsoft.com/en-us/windows/wsl/wsl-plugins>
- Microsoft WSL Plugin API NuGet package:
  <https://www.nuget.org/packages/Microsoft.WSL.PluginApi>
- Microsoft WSL plugin sample:
  <https://github.com/microsoft/wsl-plugin-sample>
