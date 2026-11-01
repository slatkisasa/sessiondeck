// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Security.Principal;
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
            var userId = WindowsIdentity.GetCurrent().User?.Value
                ?? throw new InvalidOperationException("The current Windows user could not be identified.");
            using var instance = SingleInstance.TryAcquire(@"Global\SessionDeck.Application." + userId);
            if (instance is null)
            {
                MessageBox.Show("SessionDeck is already running.", "SessionDeck",
                    MessageBoxButtons.OK, MessageBoxIcon.Information);
                return;
            }

            var paths = AppPaths.CreateDefault();
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
