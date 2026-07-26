# Read-only host mount example

This example demonstrates a WSL plugin that exposes a Windows directory inside the WSL virtual machine through a read-only Plan 9 mount.

It is intended as a reference for:

- calling [`ApiV1::mount_folder`] from [`WSLPluginV1::on_vm_started`]
- using the current [`SessionID`] when creating a mount
- mapping a Windows host path to a Linux path
- preventing WSL processes from modifying the mounted host files

## What this example does

When the WSL virtual machine starts, the plugin mounts:

- Windows source: `C:\WSL\Shared`
- Linux target: `/mnt/wsl-plugin-shared`
- mount name: `wsl-plugin-shared`
- access mode: read-only

The source directory must exist on Windows before the VM starts. Create it from PowerShell:

```powershell
New-Item -ItemType Directory -Force C:\WSL\Shared
Set-Content C:\WSL\Shared\example.txt "Hello from Windows"
```

The Linux target is created and managed by the WSL mount API. The plugin returns the Windows error reported by WSL if the mount cannot be created.

## Build and deploy

Build the plugin DLL:

```powershell
cargo build --release -p read-only-host-mount
```

Sign it using the repository helper:

```powershell
.\sign-plugin.ps1 -PluginPath .\target\release\read_only_host_mount.dll -Trust
```

Register the signed DLL using the deployment process described in the repository documentation, then restart WSL so the plugin is loaded and the VM-start hook runs.

## Verify the mount

From WSL, confirm that the host file is readable:

```bash
cat /mnt/wsl-plugin-shared/example.txt
```

Confirm that writes are rejected:

```bash
touch /mnt/wsl-plugin-shared/created-from-wsl.txt
```

The second command should fail because the mount is read-only. This setting only controls access through this WSL mount; Windows permissions and other paths to the source directory remain independent security controls.
