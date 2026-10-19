param(
    [int]$X = 0,
    [int]$Y = 0,
    [int]$Count = 1,
    [int]$DelayMs = 15
)

$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public class Win32Mouse {
    [DllImport("user32.dll")]
    public static extern bool SetProcessDPIAware();

    [DllImport("user32.dll")]
    public static extern IntPtr WindowFromPoint(POINT pt);

    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    [DllImport("user32.dll")]
    public static extern IntPtr GetAncestor(IntPtr hWnd, uint gaFlags);

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool ScreenToClient(IntPtr hWnd, ref POINT lpPoint);

    public const uint WM_LBUTTONDOWN = 0x0201;
    public const uint WM_LBUTTONUP = 0x0202;
    public const uint MK_LBUTTON = 0x0001;

    static Win32Mouse() {
        try {
            SetProcessDPIAware();
        } catch {}
    }

    public static string Inspect(int x, int y) {
        POINT pt; pt.X = x; pt.Y = y;
        IntPtr hwnd = WindowFromPoint(pt);
        IntPtr root = GetAncestor(hwnd, 2);
        uint pid = 0;
        GetWindowThreadProcessId(hwnd, out pid);
        var sb = new System.Text.StringBuilder(256);
        GetWindowText(hwnd, sb, 256);
        string procName = "unknown";
        try {
            procName = System.Diagnostics.Process.GetProcessById((int)pid).ProcessName;
        } catch {}
        return string.Format("0x{0:X} (root: 0x{1:X}) '{2}' PID={3} ({4})", hwnd.ToInt64(), root.ToInt64(), sb.ToString(), pid, procName);
    }

    public static void Click(int x, int y) {
        POINT pt; pt.X = x; pt.Y = y;
        IntPtr hwnd = WindowFromPoint(pt);
        if (hwnd == IntPtr.Zero) return;

        POINT clientPt = pt;
        ScreenToClient(hwnd, ref clientPt);
        IntPtr lParam = (IntPtr)(((clientPt.Y & 0xFFFF) << 16) | (clientPt.X & 0xFFFF));

        PostMessage(hwnd, WM_LBUTTONDOWN, (IntPtr)MK_LBUTTON, lParam);
        System.Threading.Thread.Sleep(5);
        PostMessage(hwnd, WM_LBUTTONUP, IntPtr.Zero, lParam);
    }
}
'@

$targetWin = [Win32Mouse]::Inspect($X, $Y)
Write-Output "Window under cursor at ($X, $Y): $targetWin"

for ($i = 0; $i -lt $Count; $i++) {
    [Win32Mouse]::Click($X, $Y)
    if ($DelayMs -gt 0) {
        Start-Sleep -Milliseconds $DelayMs
    }
}

Write-Output "Clicked $Count times at ($X, $Y)"
