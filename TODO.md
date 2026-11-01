# SessionDeck TODO

This is a working list of possible improvements. It is not a release schedule, and the order may change.

## Next improvements

- Upgrade the project from .NET 8 to .NET 10 LTS.
- Let users choose a custom Battle.net installation path when it cannot be found automatically.
- Add a simple way to view and restore SessionDeck's configuration backups.
- Improve error messages and troubleshooting details.
- Add more automated tests for configuration changes and recovery after errors.
- Improve keyboard navigation and screen-reader labels.

## Later ideas

- Add a system tray menu for quick account switching.
- Let users reorder accounts and mark favorites.
- Add light and dark appearance options.
- Add optional startup with Windows.
- Add ARM64 support.
- Code-sign release executables when a suitable certificate is available.
- Add an optional update notification that does not send account information.

## Privacy and safety rules

Future features should follow the same rules as the current app:

- Never save Battle.net passwords, authentication tokens, cookies, or full launcher profiles.
- Never add telemetry or upload account information.
- Back up the Battle.net configuration before changing it.
- Keep network-based features optional and explain what they access.

Have an idea? Open a [GitHub issue](https://github.com/slatkisasa/sessiondeck/issues) without including passwords, authentication data, or private configuration files.
