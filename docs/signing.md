# Signing WSL Plugin DLLs

WSL plugin DLLs must be signed before they can be loaded through the local WSL plugin registration flow.

This repository provides [sign-plugin.ps1](../sign-plugin.ps1) for local development signing.

## Prerequisites

Install the following tools on Windows:

- PowerShell
- OpenSSL
- `SignTool.exe` from the Windows SDK

`SignTool.exe` is easiest to use from a Visual Studio Developer Command Prompt, or from a shell where the Windows SDK tools are on `PATH`.

## Local Signing

Open an elevated PowerShell session, build the plugin DLL, then sign it:

```powershell
.\sign-plugin.ps1 -PluginPath .\target\release\minimal.dll
```

To also trust the generated certificate locally:

```powershell
.\sign-plugin.ps1 -PluginPath .\target\release\minimal.dll -Trust
```

The script requires administrator privileges. The `-Trust` option also imports the certificate into the local machine trusted root store.

Microsoft's WSL plugin documentation also requires Windows test signing for test-signed plugin DLLs. If WSL rejects a locally signed plugin with `TRUST_E_NOSIGNATURE`, enable test signing on the test machine and reboot if required:

```powershell
Bcdedit.exe -set TESTSIGNING ON
```

## Security Notes

Do not commit private certificates, private keys, or signed DLLs.

The script's default `cert.pfx` output is for local testing and is ignored by Git. Do not use a generated local test certificate for real signing or distribution.

When a pull request changes signing behavior, mention any manual signing or trust-store change that was used for validation.
