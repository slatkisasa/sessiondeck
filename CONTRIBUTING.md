# Contributing

Development uses short-lived branches and pull requests into `main`.

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
