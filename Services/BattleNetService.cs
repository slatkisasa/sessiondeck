using System.Diagnostics;
using BNetSwitcher.Models;

namespace BNetSwitcher.Services;

public sealed class BattleNetService
{
    private readonly AppPaths _paths;
    private readonly BattleNetConfig _config;

    public BattleNetService(AppPaths paths)
    {
        _paths = paths;
        _config = new BattleNetConfig(paths.BattleNetConfig, paths.BackupDirectory);
    }

    public bool ConfigExists => File.Exists(_paths.BattleNetConfig);
    public bool LauncherExists => File.Exists(_paths.BattleNetLauncher);
    public string ConfigPath => _paths.BattleNetConfig;
    public string LauncherPath => _paths.BattleNetLauncher;
    public IReadOnlyList<string> GetRememberedAccounts() => _config.GetRememberedAccounts();
    public string? GetCurrentAccount() => _config.GetCurrentAccount();

    public async Task<string> SwitchAsync(AccountRecord account, CancellationToken cancellationToken)
    {
        if (!ConfigExists)
        {
            throw new FileNotFoundException("Battle.net.config was not found.", ConfigPath);
        }

        if (!LauncherExists)
        {
            throw new FileNotFoundException("The Battle.net launcher was not found.", LauncherPath);
        }

        await StopBattleNetAsync(cancellationToken);
        var backupPath = _config.PreferAccount(account.AccountIdentifier);
        StartLauncher();
        return backupPath;
    }

    public async Task<string> StartNewSignInAsync(CancellationToken cancellationToken)
    {
        if (!ConfigExists)
        {
            throw new FileNotFoundException("Battle.net.config was not found.", ConfigPath);
        }

        if (!LauncherExists)
        {
            throw new FileNotFoundException("The Battle.net launcher was not found.", LauncherPath);
        }

        await StopBattleNetAsync(cancellationToken);
        var backupPath = _config.PrepareNewSignIn();
        StartLauncher();
        return backupPath;
    }

    private void StartLauncher() =>
        Process.Start(new ProcessStartInfo(_paths.BattleNetLauncher) { UseShellExecute = true });

    private static async Task StopBattleNetAsync(CancellationToken cancellationToken)
    {
        var processes = Process.GetProcessesByName("Battle.net");
        foreach (var process in processes)
        {
            try
            {
                if (!process.HasExited)
                {
                    process.Kill(true);
                }
            }
            catch (InvalidOperationException) { }
            finally
            {
                process.Dispose();
            }
        }

        var deadline = DateTime.UtcNow + TimeSpan.FromSeconds(8);
        while (IsBattleNetRunning() && DateTime.UtcNow < deadline)
        {
            await Task.Delay(200, cancellationToken);
        }
    }

    private static bool IsBattleNetRunning()
    {
        var processes = Process.GetProcessesByName("Battle.net");
        try { return processes.Length > 0; }
        finally
        {
            foreach (var process in processes)
            {
                process.Dispose();
            }
        }
    }
}
