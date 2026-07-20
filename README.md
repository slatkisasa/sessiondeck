# SessionDeck

A portable Windows 11 account session switcher for the Battle.net desktop app.

SessionDeck is an independent project and is not affiliated with or endorsed by Blizzard Entertainment.

## How it works

Battle.net keeps authenticated sessions itself. SessionDeck does **not** save passwords, authentication tokens, browser data, or a full copy of the launcher profile. It remembers account identifiers already present in `%APPDATA%\Battle.net\Battle.net.config` and safely changes which identifier appears first in `Client.SavedAccountNames`.

Every config change is backed up under `%LOCALAPPDATA%\SessionDeck\Backups` before Battle.net is restarted. Unrelated Battle.net settings are preserved. Existing data from `%LOCALAPPDATA%\BNetSwitcher` is imported automatically without overwriting newer SessionDeck data.

## First-time setup

1. Open `SessionDeck.exe`.
2. Click **Capture current** to save the Battle.net account currently selected by the launcher.
3. Click **Sign in new...** to open Battle.net at its sign-in screen.
4. Sign into the other account and enable **Stay logged in**.
5. Return to SessionDeck and click **Capture current**.
6. Double-click an account, or select it and click **Switch account**.

If Battle.net expires a session, authenticate normally in Battle.net and capture the account again if needed.

## Data and privacy

- Account labels and identifiers: `%LOCALAPPDATA%\SessionDeck\accounts.json`
- Config backups: `%LOCALAPPDATA%\SessionDeck\Backups`
- No network calls, telemetry, password collection, or auto-fill
- The application runs as the current user and does not request administrator access

## Download and portability

The release executable is self-contained for Windows x64. It includes the .NET Desktop Runtime and Windows Forms libraries, so users do not need to install .NET, run an installer, or have administrator access before opening SessionDeck.

Bundling those dependencies makes the executable much larger than the application code itself. This is intentional: the single download remains portable and works on a normal Windows 11 x64 installation without a separate runtime dependency.

## Build

Building from source requires the [.NET 8 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/8.0). Regular users downloading the release executable do not need the SDK or the .NET Desktop Runtime.

Run:

```powershell
.\build.ps1
```

The self-contained Windows x64 executable is written to `dist\SessionDeck.exe`.

To run the built-in non-destructive tests:

```powershell
.\.dotnet-sdk\dotnet.exe run --project .\SessionDeck.csproj -- --self-test
```

The tests operate only on temporary files and do not touch the live Battle.net configuration.

## Development and releases

Changes are developed on short-lived branches such as `feat/tray-menu`, `fix/window-resize`, or `hotfix/session-restore`, then merged into `main` through pull requests.

Pull request titles use Conventional Commit format. Release Please uses those titles to maintain `CHANGELOG.md`, choose the next semantic version, and open a release pull request. Merging that release pull request creates a GitHub Release containing the generated changelog, the self-contained Windows x64 executable, and its SHA-256 checksum.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the branch, pull request, and release conventions.

## License

Copyright © 2026 [slatkisasa](https://github.com/slatkisasa).

SessionDeck is licensed under the [GNU Affero General Public License v3.0](LICENSE.md). You may use, modify, and distribute it, including commercially, provided you follow the license. In particular, copyright and license notices must remain intact, and covered modifications must make their corresponding source available under the same license.
