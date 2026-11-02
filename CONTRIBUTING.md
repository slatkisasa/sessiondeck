# Contributing

Development uses short-lived branches and pull requests into `main`.

## Build and verify

Use Rust stable on Windows with the MSVC C++ toolchain and Windows SDK. Before submitting changes, run:

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
Documentation-only PRs skip the Windows build; application changes still require
dependency fetching, a Release build, and tests. UI changes also require manual Windows
checks for resizing and display scaling.

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
2. Release Please creates or updates a release pull request containing the version bump and changelog.
3. Merge the release pull request when the accumulated changes are ready to publish.
4. GitHub Actions creates the tag and GitHub Release, builds the Windows x64 executable, and attaches it with a SHA-256 checksum.

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
