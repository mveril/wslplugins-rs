# Windows and WSL Validation

Some changes require validation on a Windows host with WSL installed.

## When to Validate Manually

Manual validation is relevant when a change affects:

- Plugin DLL loading.
- DLL signing.
- Certificate trust.
- Registry registration.
- WSL service restart behavior.
- WSL command execution.
- Session, distribution, or VM metadata.

## Suggested Notes

When manual Windows validation is relevant, keep the notes short. The most useful details are:

- WSL version.
- Plugin example tested.
- Anything host-affecting, such as trusting a certificate, editing the registry, or restarting `wslservice`.

## Registry Registration

Register plugin DLLs under:

```text
HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins
```

Use machine-specific paths only in local commands or pull request notes. Do not commit machine-specific registry paths to source files.

## Service Restart

After registration changes, restart the WSL service:

```powershell
Stop-Service -Name "wslservice" -Force
wsl.exe echo "test"
```

This affects the host machine, so it is useful to mention when it was part of manual validation.
