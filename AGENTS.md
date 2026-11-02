# Repository guidance

These instructions apply to the entire repository.

## Project intent

SessionDeck is a small Windows 11 x64 Rust application with an egui interface for switching between Battle.net accounts whose authenticated sessions are already retained by the Battle.net launcher. It is currently intended for personal use, but repository changes should remain safe and understandable for other users.

## Architecture

- Target Rust/egui on Windows 11 x64 using the pinned toolchain in `rust-toolchain.toml`. Publish a standalone MSVC executable with a statically linked CRT.
- The legacy .NET project is retained but is not maintained or used for builds.
- Keep UI code in `UI/`, persisted-data models in `Models/`, and launcher/configuration logic in `Services/`.
- `Services/storage.rs` owns metadata under `%LOCALAPPDATA%\SessionDeck`, default paths, backups, and narrowly scoped Battle.net configuration changes.
- `Services/launcher.rs` owns launcher process management and account-switch orchestration.
- `Services/updates.rs` checks public GitHub release metadata; account data stays local.

## Security and privacy invariants

- Never collect, store, log, copy, or commit passwords, authentication tokens, cookies, browser profiles, or complete Battle.net account caches.
- Store only the account label and identifier needed to select an existing Battle.net session.
- Preserve unknown Battle.net configuration fields and create a backup before every live configuration write.
- Stop Battle.net before changing its configuration and use safe replacement semantics so an interrupted write cannot corrupt the file.
- Do not add telemetry, analytics, remote APIs, auto-fill, credential-manager access, or administrator requirements without explicit approval.
- Tests must use temporary paths and must never read or modify the user's live Battle.net or SessionDeck data.

## Build and verification

- Build the portable executable with `./build.ps1`; output belongs in ignored `dist/`.
- Before submitting a change, run the Cargo fetch, formatting, lint, Release build, and test commands in `CONTRIBUTING.md`. Tests use temporary paths. Treat warnings as issues to investigate.
- For UI changes, verify the minimum window size, normal and maximized resizing, and Windows display scaling. Use egui layouts that adapt to the available space.

## Git and releases

- Work on short-lived `feat/`, `fix/`, `hotfix/`, `chore/`, or `docs/` branches and merge through a pull request into `main`.
- `main` requires a pull request, squash merge, and a passing `Build and test (Windows x64)` check. Direct pushes, force-pushes, and deletion are blocked.
- CI runs for pull requests, relevant pushes to `main`, scheduled cache refreshes, and manual dispatches. Release automation runs after relevant pushes to `main`; documentation-only and workflow-only changes are ignored.
- Use Conventional Commit pull-request titles. Prefer squash merging because Release Please uses the resulting commit to generate versions and changelog entries.
- Use `feat:` for a minor release, `fix:` for a patch release, and `!` for a breaking release. A `hotfix/` branch still uses a `fix:` pull-request title.
- Releases stay in draft until the executable and checksum upload successfully. For recovery, follow `CONTRIBUTING.md`.
- Let Release Please update version metadata and generated release sections in `CHANGELOG.md`. Do not create release tags manually during normal development.

## Repository hygiene

- Never commit credentials, GitHub tokens, account data, Battle.net configuration/backups, screenshots containing private data, or local machine paths.
- Keep `.idea/`, `.vs/`, `.dotnet-sdk/`, `.dotnet-cli-home/`, `.nuget-packages/`, `bin/`, `obj/`, `target/`, and `dist/` untracked.
- Preserve unrelated user changes in a dirty worktree.
- The project is licensed under GNU AGPLv3. Keep copyright and SPDX notices intact, and apply the same license notice to new source files.

## Agent skills

### Issue tracker

Track issues and specs in GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five default triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use a single-context layout. See `docs/agents/domain.md`.
