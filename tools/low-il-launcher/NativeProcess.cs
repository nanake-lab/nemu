using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

namespace Nemu.LowIl;

// Handles are private and non-inheritable. The child starts suspended, is put in
// a kill-on-close job and has its token checked BEFORE any worker code runs.
internal sealed class NativeProcess : IDisposable
{
    [StructLayout(LayoutKind.Sequential)]
    private struct BasicLimits
    {
        public long ProcessTime, JobTime;
        public uint Flags;
        public UIntPtr MinWorkingSet, MaxWorkingSet;
        public uint ActiveProcesses;
        public UIntPtr Affinity;
        public uint Priority, Scheduling;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct IoCounters { public ulong ReadOps, WriteOps, OtherOps, ReadBytes, WriteBytes, OtherBytes; }
    [StructLayout(LayoutKind.Sequential)]
    private struct ExtendedLimits
    {
        public BasicLimits Basic;
        public IoCounters Io;
        public UIntPtr ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
    }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct StartupInfo
    {
        public int Size;
        public IntPtr Reserved, Desktop, Title;
        public uint X, Y, XSize, YSize, XCountChars, YCountChars, FillAttribute, Flags;
        public ushort ShowWindow, ReservedSize;
        public IntPtr ReservedBytes, Input, Output, Error;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct ProcessInfo { public IntPtr Process, Thread; public uint ProcessId, ThreadId; }

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr CreateJobObjectW(IntPtr attributes, string? name);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetInformationJobObject(IntPtr job, int kind, ref ExtendedLimits value, int size);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CreateProcessW(string application, StringBuilder command,
        IntPtr processAttributes, IntPtr threadAttributes, [MarshalAs(UnmanagedType.Bool)] bool inherit,
        uint flags, IntPtr environment, string directory, ref StartupInfo startup, out ProcessInfo info);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint ResumeThread(IntPtr thread);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetExitCodeProcess(IntPtr process, out uint code);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool TerminateProcess(IntPtr process, uint code);

    private IntPtr job;
    public NativeProcess()
    {
        job = CreateJobObjectW(IntPtr.Zero, null);
        if (job == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        var limits = new ExtendedLimits { Basic = new BasicLimits { Flags = 0x2000 /* KILL_ON_JOB_CLOSE */ } };
        if (!SetInformationJobObject(job, 9, ref limits, Marshal.SizeOf<ExtendedLimits>()))
        {
            var error = Marshal.GetLastWin32Error();
            Dispose();
            throw new Win32Exception(error);
        }
    }

    public int Run(string application, string[] arguments, string directory, string systemRoot)
    {
        var command = string.Join(' ', new[] { application }.Concat(arguments).Select(Arguments.Quote));
        if (command.Length >= 32767) throw new ArgumentException("Command line too long.");
        // Allowlist only. Do not propagate keys, proxy credentials, loader hooks,
        // caller-provided PATH, or a handle to authoritative storage.
        var environment = new SortedDictionary<string, string>(StringComparer.OrdinalIgnoreCase) {
            ["SystemRoot"] = systemRoot, ["WINDIR"] = systemRoot,
            ["TEMP"] = directory, ["TMP"] = directory,
            ["USERPROFILE"] = directory, ["LOCALAPPDATA"] = directory, ["APPDATA"] = directory,
            ["DOTNET_CLI_HOME"] = directory, ["DOTNET_SKIP_FIRST_TIME_EXPERIENCE"] = "1"
        };
        var block = Marshal.StringToHGlobalUni(string.Join('\0', environment.Select(p => $"{p.Key}={p.Value}")) + "\0\0");
        var info = new ProcessInfo();
        var finished = false;
        try
        {
            var startup = new StartupInfo { Size = Marshal.SizeOf<StartupInfo>() };
            // CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW.
            if (!CreateProcessW(application, new StringBuilder(command), IntPtr.Zero, IntPtr.Zero,
                false, 0x08000404, block, directory, ref startup, out info))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (NativeSecurity.Integrity(info.Process) != "S-1-16-4096")
                throw new InvalidOperationException("Worker token is not Low IL.");
            if (!AssignProcessToJobObject(job, info.Process))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (ResumeThread(info.Thread) == uint.MaxValue)
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (WaitForSingleObject(info.Process, uint.MaxValue) != 0)
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (!GetExitCodeProcess(info.Process, out var code))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            finished = true;
            return unchecked((int)code);
        }
        finally
        {
            // In particular, failure before assignment cannot leave a suspended child.
            if (info.Process != IntPtr.Zero) {
                if (!finished) TerminateProcess(info.Process, 125);
                NativeSecurity.CloseHandle(info.Process);
            }
            if (info.Thread != IntPtr.Zero) NativeSecurity.CloseHandle(info.Thread);
            Marshal.FreeHGlobal(block);
        }
    }

    public void Dispose()
    {
        if (job == IntPtr.Zero) return;
        NativeSecurity.CloseHandle(job); // Also terminates any surviving descendants.
        job = IntPtr.Zero;
    }
}
