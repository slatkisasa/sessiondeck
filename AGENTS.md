# Repository guidance

These instructions apply to the entire repository.

## Project intent

BNet Switcher is a small Windows 11 x64 WinForms application for switching between Battle.net accounts whose authenticated sessions are already retained by the Battle.net launcher. It is currently intended for personal use, but repository changes should remain safe and understandable for other users.

## Architecture

- Target .NET 8 and Windows Forms. The published artifact is a self-contained, single-file `win-x64` executable.
- Keep UI code in `UI/`, persisted-data models in `Models/`, and launcher/configuration logic in `Services/`.
- `AccountStore` owns BNet Switcher metadata under `%LOCALAPPDATA%\BNetSwitcher`.
- `BattleNetConfig` owns narrowly scoped changes to `%APPDATA%\Battle.net\Battle.net.config`.
- `BattleNetService` owns stopping, starting, and locating the Battle.net launcher.

## Security and privacy invariants

- Never collect, store, log, copy, or commit passwords, authentication tokens, cookies, browser profiles, or complete Battle.net account caches.
- Store only the account label and identifier needed to select an existing Battle.net session.
- Preserve unknown Battle.net configuration fields and create a backup before every live configuration write.
- Stop Battle.net before changing its configuration and use safe replacement semantics so an interrupted write cannot corrupt the file.
- Do not add telemetry, analytics, remote APIs, auto-fill, credential-manager access, or administrator requirements without explicit approval.
- Tests must use temporary paths and must never read or modify the user's live Battle.net or BNet Switcher data.

## Build and verification

- Build the portable executable with `./build.ps1`; output belongs in ignored `dist/`.
- Run non-destructive tests with `./.dotnet-sdk/dotnet.exe run --project ./BNetSwitcher.csproj -- --self-test` when the repository-local SDK exists, or use `dotnet run` with the same arguments otherwise.
- Before submitting a change, restore, build in `Release`, and run the self-tests. Treat warnings as issues to investigate.
- For UI changes, verify the minimum window size, normal and maximized resizing, and Windows display scaling. Prefer layout containers and anchoring over manually computed pixel positions.

## Git and releases

- Work on short-lived `feat/`, `fix/`, `hotfix/`, `chore/`, or `docs/` branches and merge through a pull request into `main`.
- Use Conventional Commit pull-request titles. Prefer squash merging because Release Please uses the resulting commit to generate versions and changelog entries.
- Use `feat:` for a minor release, `fix:` for a patch release, and `!` for a breaking release. A `hotfix/` branch still uses a `fix:` pull-request title.
- Let Release Please update version metadata and generated release sections in `CHANGELOG.md`. Do not create release tags manually during normal development.

## Repository hygiene

- Never commit credentials, GitHub tokens, account data, Battle.net configuration/backups, screenshots containing private data, or local machine paths.
- Keep `.idea/`, `.vs/`, `.dotnet-sdk/`, `.dotnet-cli-home/`, `.nuget-packages/`, `bin/`, `obj/`, and `dist/` untracked.
- Preserve unrelated user changes in a dirty worktree.
- The project is licensed under GNU AGPLv3. Keep copyright and SPDX notices intact, and apply the same license notice to new source files.
