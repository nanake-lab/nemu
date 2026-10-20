param(
    [int]$X = 0,
    [int]$Y = 0,
    [int]$Count = 1,
    [int]$DelayMs = 15,
    [switch]$MoveOnly
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
    public const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    public const uint MOUSEEVENTF_LEFTUP = 0x0004;
    public const uint MOUSEEVENTF_ABSOLUTE = 0x8000;
    public const uint MOUSEEVENTF_VIRTUALDESK = 0x4000;

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

    public static uint Move(int x, int y) {
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
        SetCursorPos(x, y);
        return res;
    }

    public static void Click(int x, int y) {
        uint res = Move(x, y);
        if (res > 0) {
            INPUT[] inputs = new INPUT[2];
            inputs[0].type = INPUT_MOUSE;
            inputs[0].mi = new MOUSEINPUT {
                dwFlags = MOUSEEVENTF_LEFTDOWN,
                time = 0,
                dwExtraInfo = IntPtr.Zero
            };
            inputs[1].type = INPUT_MOUSE;
            inputs[1].mi = new MOUSEINPUT {
                dwFlags = MOUSEEVENTF_LEFTUP,
                time = 0,
                dwExtraInfo = IntPtr.Zero
            };
            SendInput(2, inputs, Marshal.SizeOf(typeof(INPUT)));
            return;
        }

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

if ($MoveOnly) {
    $res = [Win32Mouse]::Move($X, $Y)
    Write-Output "SendInputResult: $res"
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
