# Release Process

This project uses Git Flow for published library crates.

## Branches

- `develop` carries versioned crate work.
- `release/*` branches are used for release preparation and validation.
- `main` remains the stable branch.

Changes unrelated to published libraries can use focused branches targeting `main`, such as `doc/*`, `ci/*`, or `example/*`.

## Validation

Run the full validation set before publishing:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

On `release/*` branches, also run:

```powershell
cargo publish --workspace --dry-run
```

Do not run release validation from routine feature branches.

## Windows Notes

When a release changes plugin loading, signing, registry registration, or WSL service behavior, add a short note about what was tested manually.

Relevant notes include:

- WSL version.
- Plugin example tested.
- Any host-affecting validation, such as certificate trust, registry edits, or WSL service restarts.
