using System.Text;
using System.Text.Json;
using BNetSwitcher.Models;

namespace BNetSwitcher.Services;

public sealed class AccountStore
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };
    private readonly string _accountsFile;

    public AccountStore(string accountsFile) => _accountsFile = accountsFile;

    public IReadOnlyList<AccountRecord> Load()
    {
        if (!File.Exists(_accountsFile))
        {
            return [];
        }

        var json = File.ReadAllText(_accountsFile);
        return JsonSerializer.Deserialize<List<AccountRecord>>(json, JsonOptions) ?? [];
    }

    public void Save(IEnumerable<AccountRecord> accounts)
    {
        var directory = Path.GetDirectoryName(_accountsFile)
            ?? throw new InvalidOperationException("The account storage path has no parent directory.");
        Directory.CreateDirectory(directory);

        var temporaryFile = _accountsFile + ".tmp";
        var json = JsonSerializer.Serialize(accounts, JsonOptions);
        File.WriteAllText(temporaryFile, json, new UTF8Encoding(false));
        File.Move(temporaryFile, _accountsFile, true);
    }

}
