# Release Process

This document describes how to prepare and publish a new release of DataSynth.

## Versioning Policy

DataSynth follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **Major** (X.0.0): Breaking API changes (removed public types, changed function signatures, incompatible config schema changes)
- **Minor** (0.X.0): New features, new generators, new config sections -- all backward-compatible
- **Patch** (0.0.X): Bug fixes, documentation corrections, internal refactors with no user-visible behavior change

## Pre-Release Checklist

Before cutting a release, verify:

1. **All tests pass:**
   ```bash
   cargo test --workspace
   ```

2. **No clippy warnings:**
   ```bash
   cargo clippy --workspace
   ```

3. **Code formatted:**
   ```bash
   cargo fmt --check
   ```

4. **CHANGELOG.md updated** with the new version section (see format in the file header).

5. **Version bumped** in the workspace `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "X.Y.Z"
   ```

6. **Dry-run publish** to catch metadata or dependency issues:
   ```bash
   cargo publish --dry-run -p datasynth-core
   ```

## Publishing Order

Crates must be published in dependency order. Wait for each crate to appear on crates.io before publishing its dependents.

| Step | Crate               | Notes                          |
|------|----------------------|--------------------------------|
| 1    | `datasynth-core`     | Domain models, traits, distributions |
| 2    | `datasynth-config`   | Configuration schema           |
| 3    | `datasynth-output`   | Output sinks (CSV, JSON, Parquet) |
| 4    | `datasynth-standards`| Accounting/audit standards     |
| 5    | `datasynth-banking`  | KYC/AML banking module         |
| 6    | `datasynth-generators`| All data generators           |
| 7    | `datasynth-eval`     | Evaluation framework           |
| 8    | `datasynth-ocpm`     | OCEL 2.0 process mining        |
| 9    | `datasynth-graph`    | Graph export (PyG, Neo4j, DGL) |
| 10   | `datasynth-runtime`  | Orchestrator                   |
| 11   | `datasynth-fingerprint`| Privacy-preserving fingerprinting |
| 12   | `datasynth-test-utils`| Test utilities                 |
| 13   | `datasynth-server`   | REST/gRPC/WebSocket server     |
| 14   | `datasynth-cli`      | Binary (`datasynth-data`)      |

> **Note:** `datasynth-ui` is a Tauri desktop application and is **not** published to crates.io. Desktop releases are handled separately via Tauri's build/bundle process.

Publish each crate:

```bash
cargo publish -p datasynth-core
# wait for it to appear on crates.io (~30-60 seconds)
cargo publish -p datasynth-config
# ... continue in order
```

## Git Workflow

### 1. Create a release branch

```bash
git checkout -b release/vX.Y.Z
```

### 2. Bump version and update CHANGELOG

- Update `version` in the workspace `Cargo.toml`.
- Add the new version section to `CHANGELOG.md`.
- Run `cargo check --workspace` to ensure `Cargo.lock` is updated.

### 3. Open a PR to main

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: prepare release vX.Y.Z"
git push -u origin release/vX.Y.Z
gh pr create --title "Release vX.Y.Z" --body "Release preparation for vX.Y.Z"
```

Review, get approval, and merge.

### 4. Tag the release

After the PR is merged into `main`:

```bash
git checkout main
git pull
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### 5. Publish to crates.io

Follow the publishing order above.

### 6. Create a GitHub release (optional)

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes-from-tag
```

## Post-Release

- Verify the published crates on [crates.io](https://crates.io/search?q=datasynth).
- Announce the release in relevant channels.
- If a critical bug is found, follow the same process with a patch bump (vX.Y.Z+1).
