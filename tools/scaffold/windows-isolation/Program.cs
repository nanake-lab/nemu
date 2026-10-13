using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Principal;

if (!OperatingSystem.IsWindows()) throw new PlatformNotSupportedException("Windows runner required.");
if (args.Length != 1 || !Directory.Exists(args[0])) throw new ArgumentException("Expected a newly created temporary directory.");
var target = Path.Combine(args[0], "probe.txt");
// Positive control proves the same process can write before lowering integrity.
File.WriteAllText(target, "medium control");
File.Delete(target);
Native.LowerCurrentProcess();
try
{
    File.WriteAllText(target, "unexpected low write");
    throw new InvalidOperationException("Low-integrity process wrote to the medium-integrity directory.");
}
catch (UnauthorizedAccessException)
{
    Console.WriteLine("PASS: process token lowered to Low; medium-directory write denied; positive write control passed.");
}

internal static class Native
{
    [StructLayout(LayoutKind.Sequential)]
    private struct Label { public IntPtr Sid; public uint Attributes; }

    // Test-only Win32 boundary: lowers this disposable process, never the caller or a Nemu process.
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetTokenInformation(IntPtr token, int kind, ref Label value, int size);
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetTokenInformation(IntPtr token, int kind, IntPtr value, int size, out int needed);
    [DllImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(IntPtr handle);

    public static void LowerCurrentProcess()
    {
        if (!OperatingSystem.IsWindows()) throw new PlatformNotSupportedException();
        if (!OpenProcessToken(Process.GetCurrentProcess().Handle, 0x0080 | 0x0008, out var token))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        var sid = new SecurityIdentifier("S-1-16-4096");
        var bytes = new byte[sid.BinaryLength];
        sid.GetBinaryForm(bytes, 0);
        var memory = Marshal.AllocHGlobal(bytes.Length);
        try
        {
            Marshal.Copy(bytes, 0, memory, bytes.Length);
            var label = new Label { Sid = memory, Attributes = 0x20 };
            if (!SetTokenInformation(token, 25, ref label, Marshal.SizeOf<Label>() + bytes.Length))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            GetTokenInformation(token, 25, IntPtr.Zero, 0, out var size);
            var readback = Marshal.AllocHGlobal(size);
            try
            {
                if (!GetTokenInformation(token, 25, readback, size, out _))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                var actual = Marshal.PtrToStructure<Label>(readback);
                if (new SecurityIdentifier(actual.Sid).Value != "S-1-16-4096")
                    throw new InvalidOperationException("Process token is not Low IL.");
            }
            finally { Marshal.FreeHGlobal(readback); }
        }
        finally { Marshal.FreeHGlobal(memory); CloseHandle(token); }
    }
}
