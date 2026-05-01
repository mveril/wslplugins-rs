# Contributing

Thanks for considering a contribution to WSLPlugins-rs.

This repository is a Cargo workspace for Rust libraries and examples related to WSL plugins. Changes should stay focused and avoid host-affecting operations unless they are useful for the change being tested.

## Branches

Published library work follows Git Flow:

- Start feature and fix branches from `develop`.
- Target library pull requests back to `develop`.
- Use `release/*` branches for versioned release preparation and final validation.
- Keep `main` as the stable branch.

Changes that are not part of the published libraries can use direct focused branches targeting `main`, such as `doc/*`, `ci/*`, or `example/*`.

## External References

- `wslpluginapi-sys`: https://github.com/mveril/wslpluginapi-sys
- WSL plugin sample: https://github.com/microsoft/wsl-plugin-sample
- WSL repository: https://github.com/microsoft/WSL
- WSL plugin documentation: https://learn.microsoft.com/en-us/windows/wsl/wsl-plugins
- Microsoft WSL Plugin API NuGet package: https://www.nuget.org/packages/Microsoft.WSL.PluginApi

## Development

Use Rust 2021 and the repository formatting settings.

Before submitting a pull request, run the relevant checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

Do not run `cargo build --release` during routine library work. Release DLLs can interfere with local WSL plugin workflows.

Only run:

```powershell
cargo publish --workspace --dry-run
```

from `release/*` branches during release validation.

## Code Style

- Use `snake_case` for crates, modules, files, functions, methods, and variables.
- Use `PascalCase` for public types and traits.
- Return typed errors instead of panicking.
- Do not use `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`, stdout/stderr printing, or process exits in library code.
- Document unsafe blocks with a `SAFETY:` comment.

## Tests

Place ordinary Rust tests next to the implementation or in crate-level `tests/` directories.

Macro behavior belongs in `crates/wslplugins-macro-tests/tests/`, with UI fixtures under `tests/ui/`.

Name tests after behavior, for example:

```text
formats_user_distribution_id_as_guid
```

## Pull Requests

Good pull request descriptions usually include:

- A concise description of the change.
- API, behavior, or compatibility changes, when relevant.
- Test results for the checks that were run.
- Windows validation notes when the change was actually tested with WSL.

Keep commits focused. Recent commit subjects use short imperative or descriptive wording, for example:

```text
Fix offline distribution information visibility
```

## Security

Do not commit private certificates, private keys, signed DLLs, machine-specific registry paths, or local secrets.

Treat `sign-plugin.ps1 -Trust`, registry edits, and WSL service restarts as host-affecting operations. Mention them in pull requests when they are part of the validation.
