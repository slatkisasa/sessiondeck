// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

namespace SessionDeck.Services;

public sealed record AppPaths(
    string DataDirectory,
    string AccountsFile,
    string BackupDirectory,
    string BattleNetConfig,
    string BattleNetLauncher,
    string? LegacyDataDirectory = null)
{
    public static AppPaths CreateDefault()
    {
        var localData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var roamingData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var programFilesX86 = Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86);
        var dataDirectory = Path.Combine(localData, "SessionDeck");

        return new AppPaths(
            dataDirectory,
            Path.Combine(dataDirectory, "accounts.json"),
            Path.Combine(dataDirectory, "Backups"),
            Path.Combine(roamingData, "Battle.net", "Battle.net.config"),
            Path.Combine(programFilesX86, "Battle.net", "Battle.net Launcher.exe"),
            Path.Combine(localData, "BNetSwitcher"));
    }

    public void ImportLegacyData()
    {
        if (string.IsNullOrWhiteSpace(LegacyDataDirectory) ||
            !Directory.Exists(LegacyDataDirectory) ||
            string.Equals(
                Path.GetFullPath(LegacyDataDirectory),
                Path.GetFullPath(DataDirectory),
                StringComparison.OrdinalIgnoreCase))
        {
            return;
        }

        CopyFileIfMissing(
            Path.Combine(LegacyDataDirectory, "accounts.json"),
            AccountsFile);
        CopyDirectoryFilesIfMissing(
            Path.Combine(LegacyDataDirectory, "Backups"),
            BackupDirectory);
    }

    private static void CopyDirectoryFilesIfMissing(string sourceDirectory, string destinationDirectory)
    {
        if (!Directory.Exists(sourceDirectory))
        {
            return;
        }

        foreach (var sourceFile in Directory.EnumerateFiles(sourceDirectory, "*", SearchOption.AllDirectories))
        {
            var relativePath = Path.GetRelativePath(sourceDirectory, sourceFile);
            CopyFileIfMissing(sourceFile, Path.Combine(destinationDirectory, relativePath));
        }
    }

    private static void CopyFileIfMissing(string sourceFile, string destinationFile)
    {
        if (!File.Exists(sourceFile) || File.Exists(destinationFile))
        {
            return;
        }

        var destinationDirectory = Path.GetDirectoryName(destinationFile)
            ?? throw new InvalidOperationException("The imported data path has no parent directory.");
        Directory.CreateDirectory(destinationDirectory);
        File.Copy(sourceFile, destinationFile, false);
    }
}
