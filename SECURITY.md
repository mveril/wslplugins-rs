# Security Policy

## Supported Versions

Security fixes are handled on the currently maintained development and release branches.

For published crates, report vulnerabilities against the latest released version unless the issue is only present on an unreleased branch.

## Reporting a Vulnerability

Please do not open a public GitHub issue for a suspected vulnerability.

Report security issues privately through GitHub Security Advisories when available, or contact the maintainer listed in [Cargo.toml](Cargo.toml).

Include as much detail as possible:

- Affected crate, example, script, or workflow.
- Reproduction steps.
- Expected and actual behavior.
- Impact on plugin loading, signing, WSL sessions, command execution, or Windows host state.
- Whether the issue requires administrator privileges.

## Scope

Security-sensitive areas include:

- WSL plugin entry points and hook wiring.
- Unsafe blocks and FFI boundaries.
- Command execution through WSL APIs.
- DLL signing and certificate handling.
- Registry registration under WSL plugin keys.
- WSL service restart workflows.
- CI or release automation that publishes crates or artifacts.

## Host-Affecting Operations

The following operations affect the host machine and should be treated carefully:

- Running `sign-plugin.ps1 -Trust`.
- Installing or trusting certificates.
- Editing Windows registry keys.
- Stopping or starting the WSL service.
- Loading locally built plugin DLLs into WSL.

Do not commit private certificates, private keys, signed DLLs, or machine-specific registry paths.
