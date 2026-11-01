// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Diagnostics;
using SessionDeck.Models;

namespace SessionDeck.Services;

public sealed class BattleNetService
{
    private readonly AppPaths _paths;
    private readonly BattleNetConfig _config;

    private readonly Action _stopProcesses;
    private readonly Func<bool> _isRunning;
    private readonly Action _startLauncher;
    private readonly TimeSpan _stopTimeout;

    public BattleNetService(AppPaths paths)
        : this(paths, StopProcesses, IsBattleNetRunning,
            () => Process.Start(new ProcessStartInfo(paths.BattleNetLauncher) { UseShellExecute = true }),
            TimeSpan.FromSeconds(8)) { }

    internal BattleNetService(AppPaths paths, Action stopProcesses, Func<bool> isRunning,
        Action startLauncher, TimeSpan stopTimeout)
    {
        _stopProcesses = stopProcesses;
        _isRunning = isRunning;
        _startLauncher = startLauncher;
        _stopTimeout = stopTimeout;
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
        cancellationToken.ThrowIfCancellationRequested();
        var backupPath = _config.PreferAccount(account.AccountIdentifier);
        _startLauncher();
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
        cancellationToken.ThrowIfCancellationRequested();
        var backupPath = _config.PrepareNewSignIn();
        _startLauncher();
        return backupPath;
    }

    private async Task StopBattleNetAsync(CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        _stopProcesses();
        var timer = Stopwatch.StartNew();
        while (_isRunning())
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (timer.Elapsed >= _stopTimeout)
            {
                throw new TimeoutException("Battle.net did not close. Close it manually and try again. Configuration was not changed.");
            }
            await Task.Delay(200, cancellationToken);
        }
    }

    private static void StopProcesses()
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
