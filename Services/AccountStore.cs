// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Text.Json;
using SessionDeck.Models;

namespace SessionDeck.Services;

public sealed class AccountStore
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };
    private readonly string _accountsFile;

    public AccountStore(string accountsFile) => _accountsFile = accountsFile;

    public bool Exists => File.Exists(_accountsFile);

    public List<AccountRecord> LoadOrImport(Func<IEnumerable<AccountRecord>> import)
    {
        if (Exists)
        {
            return Load().ToList();
        }

        var accounts = import().ToList();
        Save(accounts);
        return accounts;
    }

    public List<AccountRecord> Update(IEnumerable<AccountRecord> current, Action<List<AccountRecord>> change)
    {
        var snapshot = current.Select(account => new AccountRecord
        {
            Id = account.Id,
            DisplayName = account.DisplayName,
            AccountIdentifier = account.AccountIdentifier,
            CreatedAt = account.CreatedAt,
            LastUsedAt = account.LastUsedAt
        }).ToList();
        change(snapshot);
        Save(snapshot);
        return snapshot;
    }

    public string PreserveForRecovery()
    {
        var preserved = _accountsFile + $".damaged-{Guid.NewGuid():N}";
        File.Copy(_accountsFile, preserved, false);
        return preserved;
    }

    public IReadOnlyList<AccountRecord> Load()
    {
        if (!File.Exists(_accountsFile))
        {
            return [];
        }

        var json = File.ReadAllText(_accountsFile);
        try
        {
            var accounts = JsonSerializer.Deserialize<List<AccountRecord>>(json, JsonOptions)
                ?? throw new InvalidDataException("The account file must contain a list.");
            Validate(accounts);
            return accounts;
        }
        catch (JsonException exception)
        {
            throw new InvalidDataException("The account file contains invalid JSON.", exception);
        }
    }

    public void Save(IEnumerable<AccountRecord> accounts)
    {
        var snapshot = accounts.ToList();
        Validate(snapshot);
        var directory = Path.GetDirectoryName(_accountsFile)
            ?? throw new InvalidOperationException("The account storage path has no parent directory.");
        Directory.CreateDirectory(directory);

        var json = JsonSerializer.Serialize(snapshot, JsonOptions);
        AtomicFile.WriteAllText(_accountsFile, json);
    }

    private static void Validate(IReadOnlyList<AccountRecord> accounts)
    {
        var ids = new HashSet<Guid>();
        var identifiers = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var account in accounts)
        {
            if (account is null || account.Id == Guid.Empty ||
                string.IsNullOrWhiteSpace(account.DisplayName) ||
                string.IsNullOrWhiteSpace(account.AccountIdentifier) ||
                account.AccountIdentifier != account.AccountIdentifier.Trim() ||
                account.AccountIdentifier.Contains(',') ||
                !ids.Add(account.Id) || !identifiers.Add(account.AccountIdentifier))
            {
                throw new InvalidDataException("The account file contains invalid or duplicate accounts.");
            }
        }
    }
}
