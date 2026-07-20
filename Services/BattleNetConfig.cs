// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace SessionDeck.Services;

public sealed class BattleNetConfig
{
    private readonly string _configPath;
    private readonly string _backupDirectory;

    public BattleNetConfig(string configPath, string backupDirectory)
    {
        _configPath = configPath;
        _backupDirectory = backupDirectory;
    }

    public IReadOnlyList<string> GetRememberedAccounts()
    {
        if (!File.Exists(_configPath))
        {
            return [];
        }

        var root = LoadRoot();
        var savedNames = root["Client"]?["SavedAccountNames"]?.GetValue<string>();
        return SplitAccounts(savedNames);
    }

    public string? GetCurrentAccount() => GetRememberedAccounts().FirstOrDefault();

    public string PreferAccount(string username)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(username);
        if (!File.Exists(_configPath))
        {
            throw new FileNotFoundException("Battle.net.config was not found.", _configPath);
        }

        var root = LoadRoot();
        var client = root["Client"] as JsonObject
            ?? throw new InvalidDataException("Battle.net.config does not contain a Client object.");

        var existing = SplitAccounts(client["SavedAccountNames"]?.GetValue<string>());
        var reordered = new[] { username.Trim() }
            .Concat(existing.Where(account =>
                !account.Equals(username.Trim(), StringComparison.OrdinalIgnoreCase)))
            .Distinct(StringComparer.OrdinalIgnoreCase);
        client["SavedAccountNames"] = string.Join(',', reordered);

        return BackupAndSave(root);
    }

    public string PrepareNewSignIn()
    {
        if (!File.Exists(_configPath))
        {
            throw new FileNotFoundException("Battle.net.config was not found.", _configPath);
        }

        var root = LoadRoot();
        var client = root["Client"] as JsonObject
            ?? throw new InvalidDataException("Battle.net.config does not contain a Client object.");
        client["SavedAccountNames"] = string.Empty;
        return BackupAndSave(root);
    }

    private JsonObject LoadRoot()
    {
        var json = File.ReadAllText(_configPath);
        return JsonNode.Parse(json) as JsonObject
            ?? throw new InvalidDataException("Battle.net.config is not a JSON object.");
    }

    private static IReadOnlyList<string> SplitAccounts(string? savedNames) =>
        string.IsNullOrWhiteSpace(savedNames)
            ? []
            : savedNames.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

    private string BackupAndSave(JsonObject root)
    {
        Directory.CreateDirectory(_backupDirectory);
        var backupPath = Path.Combine(
            _backupDirectory,
            $"Battle.net.config.{DateTime.Now:yyyyMMdd-HHmmss-fff}.backup");
        File.Copy(_configPath, backupPath, false);

        var temporaryFile = _configPath + ".bnetswitcher.tmp";
        var json = root.ToJsonString(new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(temporaryFile, json, new UTF8Encoding(false));
        File.Move(temporaryFile, _configPath, true);
        PruneBackups();
        return backupPath;
    }

    private void PruneBackups()
    {
        foreach (var oldBackup in Directory
                     .EnumerateFiles(_backupDirectory, "Battle.net.config.*.backup")
                     .OrderByDescending(File.GetCreationTimeUtc)
                     .Skip(20))
        {
            try { File.Delete(oldBackup); }
            catch (IOException) { }
            catch (UnauthorizedAccessException) { }
        }
    }
}
