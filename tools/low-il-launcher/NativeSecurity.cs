using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Principal;

namespace Nemu.LowIl;

// Win32 boundary: only this disposable launcher token and one NEW directory are
// changed. No privilege enablement, existing ACL changes, or global settings.
internal static class NativeSecurity
{
    [StructLayout(LayoutKind.Sequential)]
    private struct Label { public IntPtr Sid; public uint Attributes; }
    [StructLayout(LayoutKind.Sequential)]
    private struct SecurityAttributes
    {
        public int Length;
        public IntPtr Descriptor;
        [MarshalAs(UnmanagedType.Bool)] public bool Inherit;
    }

    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetTokenInformation(IntPtr token, int kind, ref Label value, int size);
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetTokenInformation(IntPtr token, int kind, IntPtr value, int size, out int needed);
    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ConvertStringSecurityDescriptorToSecurityDescriptorW(
        string sddl, uint revision, out IntPtr descriptor, out uint size);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CreateDirectoryW(string path, ref SecurityAttributes attributes);
    [DllImport("kernel32.dll")]
    private static extern IntPtr LocalFree(IntPtr pointer);
    [DllImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool CloseHandle(IntPtr handle);
    [DllImport("shell32.dll")]
    private static extern int SHGetKnownFolderPath(ref Guid folder, uint flags, IntPtr token, out IntPtr path);
    [DllImport("advapi32.dll", CharSet = CharSet.Unicode)]
    private static extern uint GetNamedSecurityInfoW(string path, uint type, uint info,
        out IntPtr owner, out IntPtr group, out IntPtr dacl, out IntPtr sacl, out IntPtr descriptor);
    [DllImport("advapi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetAce(IntPtr acl, uint index, out IntPtr ace);

    public static string PathIntegrity(string path)
    {
        var error = GetNamedSecurityInfoW(path, 1 /* SE_FILE_OBJECT */, 0x10 /* LABEL_SECURITY_INFORMATION */,
            out _, out _, out _, out var sacl, out var descriptor);
        if (error != 0) throw new Win32Exception((int)error);
        try
        {
            if (sacl == IntPtr.Zero) return "S-1-16-8192"; // Windows default for unlabelled objects.
            var count = (ushort)Marshal.ReadInt16(sacl, 4);
            string? sid = null;
            for (uint i = 0; i < count; i++)
            {
                if (!GetAce(sacl, i, out var ace)) throw new Win32Exception(Marshal.GetLastWin32Error());
                if (Marshal.ReadByte(ace) != 0x11 /* SYSTEM_MANDATORY_LABEL_ACE_TYPE */) continue;
                if (sid != null || (Marshal.ReadInt32(ace, 4) & 1 /* NO_WRITE_UP */) == 0)
                    throw new InvalidOperationException("Unsupported mandatory label policy.");
                sid = new SecurityIdentifier(IntPtr.Add(ace, 8)).Value;
            }
            return sid ?? "S-1-16-8192";
        }
        finally { LocalFree(descriptor); }
    }

    public static string LocalLow()
    {
        var id = new Guid("A520A1A4-1780-4FF6-BD18-167343C5AF16");
        Marshal.ThrowExceptionForHR(SHGetKnownFolderPath(ref id, 0, IntPtr.Zero, out var path));
        try { return Marshal.PtrToStringUni(path) ?? throw new InvalidOperationException("Missing LocalLow."); }
        finally { Marshal.FreeCoTaskMem(path); }
    }

    public static string CreateWorkerDirectory(string parent)
    {
        using var identity = WindowsIdentity.GetCurrent();
        var sid = identity.User?.Value ?? throw new InvalidOperationException("Missing current user.");
        // Protected DACL: current user and SYSTEM only. Low No-Write-Up label
        // propagates to files/subdirectories. Security is applied at creation;
        // an existing name fails instead of being relabelled.
        var sddl = $"D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)S:(ML;OICI;NW;;;LW)";
        if (!ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl, 1, out var descriptor, out _))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        try
        {
            var path = Path.Combine(parent, $"Nemu-worker-{Guid.NewGuid():N}");
            var attributes = new SecurityAttributes {
                Length = Marshal.SizeOf<SecurityAttributes>(), Descriptor = descriptor, Inherit = false };
            if (!CreateDirectoryW(path, ref attributes)) throw new Win32Exception(Marshal.GetLastWin32Error());
            if (PathIntegrity(path) != "S-1-16-4096")
                throw new InvalidOperationException("Temporary directory label readback failed.");
            return path;
        }
        finally { LocalFree(descriptor); }
    }

    public static string Integrity(IntPtr process)
    {
        if (!OpenProcessToken(process, 0x0008 /* TOKEN_QUERY */, out var token))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        try { return TokenIntegrity(token); }
        finally { CloseHandle(token); }
    }

    private static string TokenIntegrity(IntPtr token)
    {
        GetTokenInformation(token, 25 /* TokenIntegrityLevel */, IntPtr.Zero, 0, out var size);
        if (size <= 0) throw new Win32Exception(Marshal.GetLastWin32Error());
        var buffer = Marshal.AllocHGlobal(size);
        try
        {
            if (!GetTokenInformation(token, 25, buffer, size, out _))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            return new SecurityIdentifier(Marshal.PtrToStructure<Label>(buffer).Sid).Value;
        }
        finally { Marshal.FreeHGlobal(buffer); }
    }

    public static void RequireMedium()
    {
        using var process = Process.GetCurrentProcess();
        if (Integrity(process.Handle) != "S-1-16-8192")
            throw new InvalidOperationException("A non-elevated Medium IL launcher is required.");
    }

    public static void LowerSelf()
    {
        using var process = Process.GetCurrentProcess();
        if (!OpenProcessToken(process.Handle, 0x0080 | 0x0008, out var token))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        var sid = new SecurityIdentifier("S-1-16-4096");
        var bytes = new byte[sid.BinaryLength];
        sid.GetBinaryForm(bytes, 0);
        var memory = Marshal.AllocHGlobal(bytes.Length);
        try
        {
            Marshal.Copy(bytes, 0, memory, bytes.Length);
            var label = new Label { Sid = memory, Attributes = 0x20 /* SE_GROUP_INTEGRITY */ };
            if (!SetTokenInformation(token, 25, ref label, Marshal.SizeOf<Label>() + bytes.Length))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (TokenIntegrity(token) != "S-1-16-4096")
                throw new InvalidOperationException("Low IL token readback failed.");
        }
        finally { Marshal.FreeHGlobal(memory); CloseHandle(token); }
    }
}
