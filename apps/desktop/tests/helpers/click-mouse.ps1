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

    static Win32Mouse() {
        try {
            SetProcessDPIAware();
        } catch {}
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

    public static uint Click(int x, int y) {
        uint moveRes = Move(x, y);
        if (moveRes != 1) {
            return 0;
        }

        int vLeft = GetSystemMetrics(76);
        int vTop = GetSystemMetrics(77);
        int vWidth = GetSystemMetrics(78);
        int vHeight = GetSystemMetrics(79);
        if (vWidth <= 0) vWidth = GetSystemMetrics(0);
        if (vHeight <= 0) vHeight = GetSystemMetrics(1);

        int normX = (int)(((x - vLeft) * 65535.0f / (vWidth - 1)) + 0.5f);
        int normY = (int)(((y - vTop) * 65535.0f / (vHeight - 1)) + 0.5f);

        INPUT[] down = new INPUT[1];
        down[0].type = INPUT_MOUSE;
        down[0].mi = new MOUSEINPUT {
            dx = normX,
            dy = normY,
            dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_LEFTDOWN,
            time = 0,
            dwExtraInfo = IntPtr.Zero
        };
        uint downRes = SendInput(1, down, Marshal.SizeOf(typeof(INPUT)));
        if (downRes != 1) {
            return 0;
        }

        System.Threading.Thread.Sleep(15);

        INPUT[] up = new INPUT[1];
        up[0].type = INPUT_MOUSE;
        up[0].mi = new MOUSEINPUT {
            dx = normX,
            dy = normY,
            dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_LEFTUP,
            time = 0,
            dwExtraInfo = IntPtr.Zero
        };
        uint upRes = SendInput(1, up, Marshal.SizeOf(typeof(INPUT)));
        if (upRes != 1) {
            return 0;
        }

        return 1;
    }
}
'@

if ($MoveOnly) {
    $res = [Win32Mouse]::Move($X, $Y)
    Write-Output "SendInputResult: $res"
    if ($res -ne 1) {
        Write-Error "SendInput move failed (returned $res)"
        exit 1
    }
    Write-Output "Moved cursor to ($X, $Y)"
    exit 0
}

for ($i = 0; $i -lt $Count; $i++) {
    $res = [Win32Mouse]::Click($X, $Y)
    if ($res -ne 1) {
        Write-Output "SendInputResult: $res"
        Write-Error "SendInput click failed (returned $res) at ($X, $Y)"
        exit 1
    }
    if ($DelayMs -gt 0) {
        Start-Sleep -Milliseconds $DelayMs
    }
}

Write-Output "SendInputResult: 1"
Write-Output "Clicked $Count times at ($X, $Y)"
exit 0
