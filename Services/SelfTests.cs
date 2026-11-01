// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Text.Json.Nodes;
using SessionDeck.Models;
using SessionDeck.UI;

namespace SessionDeck.Services;

internal static class SelfTests
{
    public static int Run()
    {
        var directory = Directory.CreateTempSubdirectory("SessionDeckTests-");
        try
        {
            TestAccountStore(directory.FullName);
            TestConfigSwitching(directory.FullName);
            TestResponsiveLayout(directory.FullName);
            Console.WriteLine("All self-tests passed.");
            return 0;
        }
        catch (Exception exception)
        {
            Console.Error.WriteLine(exception);
            return 1;
        }
        finally
        {
            directory.Delete(true);
        }
    }

    private static void TestAccountStore(string directory)
    {
        var file = Path.Combine(directory, "accounts.json");
        var store = new AccountStore(file);
        var expected = new AccountRecord
        {
            DisplayName = "Main",
            AccountIdentifier = "main@example.invalid"
        };
        store.Save([expected]);
        var actual = store.Load().Single();
        Require(actual.Id == expected.Id, "Account ID did not round-trip.");
        Require(actual.DisplayName == expected.DisplayName, "Display name did not round-trip.");
        Require(actual.AccountIdentifier == expected.AccountIdentifier, "Account identifier did not round-trip.");
    }

    private static void TestConfigSwitching(string directory)
    {
        var configPath = Path.Combine(directory, "Battle.net.config");
        var backups = Path.Combine(directory, "backups");
        File.WriteAllText(
            configPath,
            """
            {
              "Client": {
                "SavedAccountNames": "first@example.invalid,second@example.invalid",
                "AutoLogin": "true"
              },
              "Games": { "preserve": true }
            }
            """);

        var config = new BattleNetConfig(configPath, backups);
        Require(config.GetCurrentAccount() == "first@example.invalid", "Current account was not detected.");
        var backup = config.PreferAccount("second@example.invalid");
        Require(File.Exists(backup), "Config backup was not created.");
        Require(config.GetCurrentAccount() == "second@example.invalid", "Selected account was not moved first.");

        var root = JsonNode.Parse(File.ReadAllText(configPath))!;
        Require(root["Games"]?["preserve"]?.GetValue<bool>() == true, "Unrelated config was not preserved.");
        Require(root["Client"]?["AutoLogin"]?.GetValue<string>() == "true", "Client settings were not preserved.");

        config.PrepareNewSignIn();
        Require(config.GetRememberedAccounts().Count == 0, "New sign-in did not clear the selected account field.");
    }

    private static void TestResponsiveLayout(string directory)
    {
        var dataDirectory = Path.Combine(directory, "layout-data");
        var configPath = Path.Combine(directory, "layout-Battle.net.config");
        var launcherPath = Path.Combine(directory, "Battle.net Launcher.exe");
        File.WriteAllText(
            configPath,
            """{"Client":{"SavedAccountNames":"layout@example.invalid"}}""");
        File.WriteAllBytes(launcherPath, []);

        var paths = new AppPaths(
            dataDirectory,
            Path.Combine(dataDirectory, "accounts.json"),
            Path.Combine(dataDirectory, "Backups"),
            configPath,
            launcherPath);
        using var form = new MainForm(new AccountStore(paths.AccountsFile), new BattleNetService(paths));
        _ = form.Handle;

        foreach (var size in new[] { new Size(644, 381), new Size(1000, 700) })
        {
            form.ClientSize = size;
            PerformLayoutRecursively(form);
            foreach (var button in Descendants(form).OfType<Button>())
            {
                var preferredWidth = button.GetPreferredSize(Size.Empty).Width;
                Require(button.Width >= preferredWidth, $"Button '{button.Text}' is narrower than its text.");
                Require(button.Left >= 0 && button.Right <= button.Parent!.ClientSize.Width,
                    $"Button '{button.Text}' is outside its layout container.");
            }
        }
    }

    private static void PerformLayoutRecursively(Control control)
    {
        control.PerformLayout();
        foreach (Control child in control.Controls)
        {
            PerformLayoutRecursively(child);
        }
    }

    private static IEnumerable<Control> Descendants(Control parent)
    {
        foreach (Control child in parent.Controls)
        {
            yield return child;
            foreach (var descendant in Descendants(child))
            {
                yield return descendant;
            }
        }
    }

    private static void Require(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }
}
