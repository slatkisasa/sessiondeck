// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using SessionDeck.Services;
using SessionDeck.UI;

namespace SessionDeck;

internal static class Program
{
    [STAThread]
    private static void Main(string[] args)
    {
        ApplicationConfiguration.Initialize();

        if (args.Contains("--self-test", StringComparer.OrdinalIgnoreCase))
        {
            Environment.ExitCode = SelfTests.Run();
            return;
        }

        try
        {
            var paths = AppPaths.CreateDefault();
            paths.ImportLegacyData();
            var accountStore = new AccountStore(paths.AccountsFile);
            var battleNet = new BattleNetService(paths);
            Application.Run(new MainForm(accountStore, battleNet));
        }
        catch (Exception exception)
        {
            MessageBox.Show(
                $"SessionDeck could not start.\n\n{exception.Message}",
                "SessionDeck",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error);
        }
    }
}
