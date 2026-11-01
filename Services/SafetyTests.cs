// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using SessionDeck.Models;

namespace SessionDeck.Services;

internal static class SafetyTests
{
    public static void Run(string directory)
    {
        TestSingleInstance();
        TestRemovalSurvivesRestart(directory);
        TestFailedEdits(directory);
        TestInvalidMetadata(directory);
        TestShutdownFailure(directory).GetAwaiter().GetResult();
    }

    private static void TestSingleInstance()
    {
        var name = "SessionDeckTests-" + Guid.NewGuid().ToString("N");
        var first = SingleInstance.TryAcquire(name);
        Require(first is not null, "Could not acquire the first instance lock.");
        try
        {
            var rejected = false;
            var competitor = new Thread(() =>
            {
                using var second = SingleInstance.TryAcquire(name);
                rejected = second is null;
            });
            competitor.Start();
            competitor.Join();
            Require(rejected, "A competing instance acquired the lock.");
        }
        finally { first!.Dispose(); }
        using var next = SingleInstance.TryAcquire(name);
        Require(next is not null, "The instance lock was not released on shutdown.");
    }

    private static void TestRemovalSurvivesRestart(string directory)
    {
        var file = Path.Combine(directory, "import-accounts.json");
        var store = new AccountStore(file);
        var imports = 0;
        IEnumerable<AccountRecord> Import()
        {
            imports++;
            return [Account()];
        }
        var accounts = store.LoadOrImport(Import);
        var renamed = store.Update(accounts, items => items[0].DisplayName = "Renamed");
        Require(accounts[0].DisplayName == "Main" && renamed[0].DisplayName == "Renamed" &&
            store.Load()[0].DisplayName == "Renamed", "Successful rename did not persist an independent snapshot.");
        Require(accounts.Count == 1, "First-use import failed.");
        store.Update(accounts, items => items.Clear());
        Require(new AccountStore(file).LoadOrImport(Import).Count == 0 && imports == 1,
            "Removed accounts were imported again after restart.");
    }

    private static void TestFailedEdits(string directory)
    {
        var file = Path.Combine(directory, "blocked-accounts.json");
        Directory.CreateDirectory(file); // A directory cannot be replaced with the metadata file.
        var store = new AccountStore(file);
        var current = new List<AccountRecord> { Account() };
        foreach (var remove in new[] { false, true })
        {
            try
            {
                store.Update(current, items =>
                {
                    if (remove) items.Clear();
                    else items[0].DisplayName = "Changed";
                });
                throw new InvalidOperationException("A blocked save unexpectedly succeeded.");
            }
            catch (IOException) { }
            catch (UnauthorizedAccessException) { }
            Require(current.Count == 1 && current[0].DisplayName == "Main",
                "A failed save changed the current accounts.");
            Require(!Directory.EnumerateFiles(directory, "blocked-accounts.json.*.tmp").Any(),
                "A failed save left temporary metadata behind.");
        }
    }

    private static void TestInvalidMetadata(string directory)
    {
        var file = Path.Combine(directory, "invalid-accounts.json");
        var store = new AccountStore(file);
        foreach (var json in new[]
        {
            "{", "null", "[null]", "[{}]",
            """[{"DisplayName":"Main","AccountIdentifier":null}]""",
            """[{"DisplayName":"Main","AccountIdentifier":"a,b"}]""",
            """[{"DisplayName":"A","AccountIdentifier":"a"},{"DisplayName":"B","AccountIdentifier":"A"}]"""
        })
        {
            File.WriteAllText(file, json);
            Expect<InvalidDataException>(() => store.LoadOrImport(() => throw new Exception("Unexpected import")));
            Require(File.ReadAllText(file) == json, "Invalid metadata was overwritten during loading.");
            var preserved = store.PreserveForRecovery();
            Require(File.ReadAllText(preserved) == json, "Recovery did not preserve the original bytes.");
            store.Save([]);
            Require(store.Load().Count == 0, "Recovery could not save a valid empty list.");
        }
    }

    private static async Task TestShutdownFailure(string directory)
    {
        var config = Path.Combine(directory, "shutdown.config");
        var launcher = Path.Combine(directory, "fake-launcher.exe");
        var backups = Path.Combine(directory, "shutdown-backups");
        const string original = """{"Client":{"SavedAccountNames":"original@example.invalid"}}""";
        File.WriteAllText(config, original);
        File.WriteAllBytes(launcher, []);
        var paths = new AppPaths(directory, Path.Combine(directory, "unused.json"), backups, config, launcher);
        foreach (var newSignIn in new[] { false, true })
        {
            foreach (var mode in new[] { "timeout", "cancel-before", "cancel-after-stop" })
            {
                using var cancellation = new CancellationTokenSource();
                var stopped = false;
                var started = false;
                if (mode == "cancel-before") cancellation.Cancel();
                var service = new BattleNetService(paths, () =>
                {
                    stopped = true;
                    if (mode == "cancel-after-stop") cancellation.Cancel();
                }, () => mode == "timeout", () => started = true, TimeSpan.Zero);
                try
                {
                    if (newSignIn) await service.StartNewSignInAsync(cancellation.Token);
                    else await service.SwitchAsync(Account(), cancellation.Token);
                    throw new InvalidOperationException("Switching continued after shutdown failed or was cancelled.");
                }
                catch (TimeoutException) when (mode == "timeout") { }
                catch (OperationCanceledException) when (cancellation.IsCancellationRequested) { }
                Require(!started && File.ReadAllText(config) == original && !Directory.Exists(backups),
                    "Shutdown failure changed configuration, created a backup, or launched Battle.net.");
                Require(mode != "cancel-before" || !stopped, "Pre-cancelled operation stopped processes.");
            }
        }
    }

    private static AccountRecord Account() => new()
    {
        DisplayName = "Main", AccountIdentifier = "main@example.invalid"
    };

    private static void Expect<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new InvalidOperationException($"Expected {typeof(T).Name}.");
    }

    private static void Require(bool condition, string message)
    {
        if (!condition) throw new InvalidOperationException(message);
    }
}
