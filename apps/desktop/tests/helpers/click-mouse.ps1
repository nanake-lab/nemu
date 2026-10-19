param(
    [int]$X = 0,
    [int]$Y = 0,
    [int]$Count = 1,
    [int]$DelayMs = 15,
    [switch]$MoveOnly,
    [string]$StageHwndHex = ""
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

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int X, int Y);

    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int nIndex);

    [DllImport("user32.dll")]
    public static extern bool EnumChildWindows(IntPtr hwndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct MOUSEINPUT {
        public int dx;
        public int dy;
        public uint mouseData;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT {
        public uint type;
        public MOUSEINPUT mi;
    }

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint SendInput(uint nInputs, [MarshalAs(UnmanagedType.LPArray), In] INPUT[] pInputs, int cbSize);

    public const uint INPUT_MOUSE = 0;
    public const uint MOUSEEVENTF_MOVE = 0x0001;
    public const uint MOUSEEVENTF_ABSOLUTE = 0x8000;
    public const uint MOUSEEVENTF_VIRTUALDESK = 0x4000;

    public const uint WM_MOUSEMOVE = 0x0200;
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

    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(uint dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll")]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll")]
    public static extern bool CloseDesktop(IntPtr hDesktop);

    public static void Move(int x, int y) {
        SetCursorPos(x, y);

        int vLeft = GetSystemMetrics(76);   // SM_XVIRTUALSCREEN
        int vTop = GetSystemMetrics(77);    // SM_YVIRTUALSCREEN
        int vWidth = GetSystemMetrics(78);  // SM_CXVIRTUALSCREEN
        int vHeight = GetSystemMetrics(79); // SM_CYVIRTUALSCREEN
        if (vWidth <= 0) vWidth = GetSystemMetrics(0);
        if (vHeight <= 0) vHeight = GetSystemMetrics(1);

        int normX = (int)(((x - vLeft) * 65535.0f / (vWidth - 1)) + 0.5f);
        int normY = (int)(((y - vTop) * 65535.0f / (vHeight - 1)) + 0.5f);

        INPUT[] inputs = new INPUT[1];
        inputs[0].type = INPUT_MOUSE;
        inputs[0].mi = new MOUSEINPUT {
            dx = normX,
            dy = normY,
            mouseData = 0,
            dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
            time = 0,
            dwExtraInfo = IntPtr.Zero
        };

        uint res = SendInput(1, inputs, Marshal.SizeOf(typeof(INPUT)));
        if (res == 0) {
            IntPtr inputDesk = OpenInputDesktop(0, false, 0x01FF);
            if (inputDesk != IntPtr.Zero) {
                var t = new System.Threading.Thread(() => {
                    SetThreadDesktop(inputDesk);
                    SendInput(1, inputs, Marshal.SizeOf(typeof(INPUT)));
                });
                t.Start();
                t.Join();
                CloseDesktop(inputDesk);
            }
        }
        SetCursorPos(x, y);
    }

    public static void ForwardMouseMove(IntPtr stageHwnd, int screenX, int screenY) {
        if (stageHwnd == IntPtr.Zero) return;
        POINT pt; pt.X = screenX; pt.Y = screenY;
        POINT clientPt = pt;
        ScreenToClient(stageHwnd, ref clientPt);
        IntPtr lParam = (IntPtr)(((clientPt.Y & 0xFFFF) << 16) | (clientPt.X & 0xFFFF));
        PostMessage(stageHwnd, WM_MOUSEMOVE, IntPtr.Zero, lParam);

        EnumChildWindows(stageHwnd, (childHwnd, _) => {
            POINT cPt = pt;
            ScreenToClient(childHwnd, ref cPt);
            IntPtr cLParam = (IntPtr)(((cPt.Y & 0xFFFF) << 16) | (cPt.X & 0xFFFF));
            PostMessage(childHwnd, WM_MOUSEMOVE, IntPtr.Zero, cLParam);
            return true;
        }, IntPtr.Zero);
    }

    public static void Click(int x, int y) {
        Move(x, y);
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

if ($StageHwndHex) {
    try {
        $stageVal = [Convert]::ToInt64($StageHwndHex, 16)
        [Win32Mouse]::ForwardMouseMove([IntPtr]$stageVal, $X, $Y)
    } catch {}
}

if ($MoveOnly) {
    [Win32Mouse]::Move($X, $Y)
    Write-Output "Moved cursor to ($X, $Y)"
    exit 0
}

$targetWin = [Win32Mouse]::Inspect($X, $Y)
Write-Output "Window under cursor at ($X, $Y): $targetWin"

for ($i = 0; $i -lt $Count; $i++) {
    [Win32Mouse]::Click($X, $Y)
    if ($DelayMs -gt 0) {
        Start-Sleep -Milliseconds $DelayMs
    }
}

Write-Output "Clicked $Count times at ($X, $Y)"
