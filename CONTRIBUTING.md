# Contributing

Development uses short-lived branches and pull requests into `main`.

## Build and verify

Use Rust via rustup on Windows with the MSVC C++ toolchain and Windows SDK.
`rust-toolchain.toml` pins the compiler and required components for local builds, CI,
and releases. Update the pinned version deliberately in a pull request. Before submitting changes, run:

```powershell
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
cargo test --locked --release
./build.ps1
```

Tests inject temporary paths and never touch live Battle.net or SessionDeck data.
The portable executable goes to ignored `dist/`. Rust's CRT is linked statically.
SessionDeck is built with Cargo; the legacy .NET project is not maintained or used
for builds. UI code lives in `UI/app.rs`, launcher and storage logic in `Services/`,
and account models in `Models/`.

PR title validation runs for every PR, including title edits and documentation-only
changes. Workflow linting checks all GitHub Actions workflows with actionlint 1.7.12.
To run it locally after installing that version, use `actionlint` from the repository root.
Windows build/tests run for Rust source, Cargo/toolchain configuration, assets,
packaging scripts, and build/release workflow changes. Release Please configuration
and documentation alone skip that job. The dependency audit runs for Cargo/toolchain
changes and its own workflow. Secret/privacy scans, title validation, workflow lint,
and changed-path regression tests remain on every PR. Skipped jobs retain their
required check names and count as passing; detection errors fail the required jobs.
Renames check both paths, and incomplete changed-file lists run every check.
Scheduled and manual runs always run full checks. UI changes also require manual Windows
checks for resizing and display scaling.

CI and releases share a Windows MSVC dependency cache. CI populates the cache on
`main` after application or workflow changes and refreshes it on Mondays and Thursdays
so new PRs can restore a recent default-branch cache. Manual CI dispatches on `main`
can also populate it. Compiler, dependencies, and Cargo configuration remain part of
the cache key; changes can still require a fresh build.

## Branch names

- `feat/<short-name>` for features
- `fix/<short-name>` for normal bug fixes
- `hotfix/<short-name>` for urgent production fixes
- `chore/<short-name>` for maintenance
- `docs/<short-name>` for documentation

Use lowercase words separated by hyphens, for example `feat/tray-menu`.

## Pull request titles

Use Conventional Commit format. Release Please uses the squash-merged title to determine the next version and generate `CHANGELOG.md`:

- `feat:` creates a minor release.
- `fix:` creates a patch release.
- `feat!:` or `fix!:` creates a major release.
- `docs:`, `test:`, `chore:`, and `ci:` do not normally create a release.

A `hotfix/` branch should still use a `fix:` pull request title so it produces a patch release.

Prefer **Squash and merge** so each pull request becomes one clear changelog entry on `main`.

## Release process

1. Merge conventional pull requests into `main`.
2. Release Please uses its Rust strategy to update `Cargo.toml`, the root package in `Cargo.lock`, and the changelog together. `Cargo.toml` is the application version source.
3. Merge the release pull request when the accumulated changes are ready to publish.
4. Release Please creates a draft release and its tag together (`force-tag-creation`), so it can locate the release boundary before preparing another PR. GitHub Actions builds and tests the Windows x64 executable, uploads it with a SHA-256 checksum, and then publishes the release.

If building or uploading fails, the release stays in draft. Re-run the failed jobs,
or manually run the **Release** workflow from `main` with `release_tag` set to the
existing draft tag (for example `v1.2.3`). Recovery skips Release Please and rebuilds
the original commit; it replaces draft assets before publishing. Published releases
are rejected to prevent replacing downloads already in use. Leave `release_tag`
blank for a normal Release Please run.

Do not manually edit the generated release section in `CHANGELOG.md`; make release-note corrections through the release pull request.

## Secret and privacy checks

Install [Gitleaks](https://github.com/gitleaks/gitleaks#installing) 8.30.1 or newer,
Python 3, and [pre-commit](https://pre-commit.com/#install). On macOS:

```sh
brew install gitleaks pre-commit
pre-commit install
pre-commit run --all-files
```

On Windows, install Gitleaks from its official releases, add it to PATH, then run
`py -m pip install pre-commit` and `py -m pre_commit install` from this checkout.
Each clone needs its own hook installation. The hook scans staged content and
blocks commits when a scanner fails or Gitleaks is missing.

PRs run Secret scan and Privacy scan, including documentation-only changes.
Rust dependency audits run when Cargo/toolchain inputs or the audit workflow change.
Security checks also run on pushes to `main`, manual dispatches, and weekly
on Mondays. The dependency audit checks `Cargo.lock` against RustSec advisories and
fails on known vulnerabilities. Investigate findings and update dependencies; do
not add blanket advisory exclusions. The current lockfile has informational
unmaintained-crate warnings for `ttf-parser` (egui font rendering) and the
`zbus_names`/`zvariant`/`zvariant_derive` family (Linux accessibility). RustSec lists
no patched versions for these maintenance notices. Keep them visible and review
them during dependency upgrades; they are not vulnerability exclusions. To run
it locally:

```sh
cargo install cargo-audit --version 0.22.2 --locked
cargo audit --file Cargo.lock
```
Gitleaks detects credentials and private keys. The privacy check scans Git history
in CI and the index locally for email addresses, personal filesystem paths, and
account/configuration files. Examples must use `example.invalid` (or the reserved
example.com/org/net domains); GitHub noreply addresses are allowed. Findings do
not print matching values. To run the history checks locally:

```sh
gitleaks git --redact --no-banner
python3 scripts/check_privacy.py --history
```

These checks do not recognize arbitrary private IDs or personal data in images.
Review screenshots manually. Ignore rules are a convenience, not a security
boundary. GitHub push protection blocks supported credentials before upload;
PR checks run after upload. If a real credential leaks, revoke it immediately.

## CodeQL scope

The repository uses the checked-in `codeql.yml` workflow (advanced setup), replacing
GitHub's default setup. Rust, Python, and Actions scans run only for their relevant
paths on PRs and pushes. Rust also scans after Cargo/toolchain, asset, or packaging
changes. CodeQL workflow and path-detector changes run all three language scans.
Weekly and manual runs scan all three. Legacy C# is not scanned. Keep default setup
disabled so GitHub accepts analysis uploads from this workflow.
