namespace BNetSwitcher.Services;

public sealed record AppPaths(
    string DataDirectory,
    string AccountsFile,
    string BackupDirectory,
    string BattleNetConfig,
    string BattleNetLauncher)
{
    public static AppPaths CreateDefault()
    {
        var localData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var roamingData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var programFilesX86 = Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86);
        var dataDirectory = Path.Combine(localData, "BNetSwitcher");

        return new AppPaths(
            dataDirectory,
            Path.Combine(dataDirectory, "accounts.json"),
            Path.Combine(dataDirectory, "Backups"),
            Path.Combine(roamingData, "Battle.net", "Battle.net.config"),
            Path.Combine(programFilesX86, "Battle.net", "Battle.net Launcher.exe"));
    }
}
