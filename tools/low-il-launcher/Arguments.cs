using System.Text;

namespace Nemu.LowIl;

internal static class Arguments
{
    // Microsoft C runtime quoting; CreateProcessW gets an explicit application
    // path. There is no shell, expansion, PATH search, or joined unquoted input.
    public static string Quote(string value)
    {
        if (value.Contains('\0')) throw new ArgumentException("NUL in argument.");
        var result = new StringBuilder("\"");
        var slashes = 0;
        foreach (var c in value)
        {
            if (c == '\\') { slashes++; continue; }
            result.Append('\\', c == '"' ? slashes * 2 + 1 : slashes);
            result.Append(c);
            slashes = 0;
        }
        return result.Append('\\', slashes * 2).Append('"').ToString();
    }

    public static string ExistingLocalPath(string path, bool directory)
    {
        if (!Path.IsPathFullyQualified(path) || path.StartsWith(@"\\", StringComparison.Ordinal))
            throw new ArgumentException("An absolute local path is required.");
        var full = Path.GetFullPath(path);
        if (directory ? !Directory.Exists(full) : !File.Exists(full))
            throw new ArgumentException("Path does not exist.");
        var drive = new DriveInfo(Path.GetPathRoot(full)!);
        if (drive.DriveType != DriveType.Fixed || drive.DriveFormat != "NTFS")
            throw new ArgumentException("Local NTFS is required.");
        for (var current = full; current != null; current = Path.GetDirectoryName(current))
        {
            if ((File.GetAttributes(current) & FileAttributes.ReparsePoint) != 0)
                throw new ArgumentException("Reparse points are not accepted.");
        }
        return full;
    }
}
