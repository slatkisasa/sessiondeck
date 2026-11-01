// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using System.Text;

namespace SessionDeck.Services;

internal static class AtomicFile
{
    public static void WriteAllText(string path, string contents)
    {
        var temporaryFile = path + $".{Guid.NewGuid():N}.tmp";
        try
        {
            File.WriteAllText(temporaryFile, contents, new UTF8Encoding(false));
            File.Move(temporaryFile, path, true);
        }
        finally
        {
            try { File.Delete(temporaryFile); }
            catch (IOException) { }
            catch (UnauthorizedAccessException) { }
        }
    }
}
