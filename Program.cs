// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using BNetSwitcher.Services;
using BNetSwitcher.UI;

namespace BNetSwitcher;

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
            var accountStore = new AccountStore(paths.AccountsFile);
            var battleNet = new BattleNetService(paths);
            Application.Run(new MainForm(accountStore, battleNet));
        }
        catch (Exception exception)
        {
            MessageBox.Show(
                $"BNet Switcher could not start.\n\n{exception.Message}",
                "BNet Switcher",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error);
        }
    }
}
