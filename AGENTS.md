# Repository Guidelines

## Project Structure & Module Organization

This is a Cargo workspace for Rust WSL plugins. The main crate is `crates/wslplugins-rs`; API wrappers are in `src/api/`, plugin traits in `src/plugin/`, core WSL types in `src/`, and benchmarks in `benches/`. Macro code is split across `crates/wslplugins-macro` and `crates/wslplugins-macro-core`; macro tests are in `crates/wslplugins-macro-tests/tests/`. Example plugins live under `examples/`.

For new modules, prefer modern Rust layout: create `src/foo.rs` with children under `src/foo/` instead of adding new `src/foo/mod.rs` files. Leave existing `mod.rs` files alone unless refactoring that module deliberately.

## Build, Test, and Development Commands

- `cargo build --workspace --all-features`: build all crates without release DLLs.
- `cargo test --workspace --all-features`: run unit, integration, and macro tests.
- `cargo clippy --workspace --all-targets --all-features`: run workspace lints.
- `cargo fmt --all -- --check`: verify formatting only.

Do not run `cargo build --release` during routine work; release DLLs can interfere with local WSL plugin workflows. Run `cargo publish --workspace --dry-run` only on `release/*` branches for published-crate release validation.

## Coding Style & Naming Conventions

Use Rust 2021 and the repository `rustfmt.toml` settings. Use `snake_case` for crates, modules, files, functions, methods, and variables; use `PascalCase` for public types and traits. Workspace lints deny `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`, stdout/stderr printing, and process exits. Return typed errors and document unsafe blocks with `SAFETY:` comments.

## Testing Guidelines

Place ordinary Rust tests next to the code or in crate-level `tests/` directories. Macro behavior belongs in `crates/wslplugins-macro-tests/tests/`, with UI fixtures under `tests/ui/`. Name tests after behavior, for example `formats_user_distribution_id_as_guid`. Run the full workspace test command before opening a pull request that changes code.

## Documentation Guidelines

The mdBook in `book/` is the user-oriented guide for this library. When changing public APIs, examples, plugin behavior, packaging, validation, or anything users need to understand to build WSL plugins, update the book in the same change. If a change intentionally does not require a book update, make that explicit in the pull request.

## Git Flow & Pull Requests

Git Flow applies to published library crates: `develop` carries versioned crate work, while `main` remains the stable branch. Create `feature/*` and `fix/*` branches from `develop`, and target their pull requests back to `develop`. Use `release/*` branches for versioned release preparation, final validation, and publishing dry runs before merging to `main` and back to `develop`.

Changes unrelated to published libraries are not versioned and should use direct branches targeting `main`, such as `example/*`, `doc/*`, `ci/*`, or similar focused names. Pull requests need a concise description, linked issue when applicable, test results, and Windows validation notes when signing, registry registration, or WSL service restarts are involved.

## Commit Guidelines

Recent commits use short imperative or descriptive subjects, often with PR numbers after merge, such as `Create unpackaged-distro-blacklist-policy example (#43)` or `Fix: offline distribution information should be public`. Keep commits focused and explain API or behavior changes in the body.

## External References

- `wslpluginapi-sys`: https://github.com/mveril/wslpluginapi-sys
- WSL plugin sample: https://github.com/microsoft/wsl-plugin-sample
- WSL repository: https://github.com/microsoft/WSL
- WSL plugin documentation: https://learn.microsoft.com/en-us/windows/wsl/wsl-plugins
- Microsoft WSL Plugin API NuGet package: https://www.nuget.org/packages/Microsoft.WSL.PluginApi

## Security & Configuration Tips

Do not commit private certificates, keys, signed DLLs, or machine-specific registry paths. Treat `sign-plugin.ps1 -Trust`, registry edits, and WSL service restarts as host-affecting operations.
