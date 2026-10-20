param([string]$HwndHex)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public class Win32WindowStyle {
    [DllImport("user32.dll", EntryPoint = "GetWindowLongPtrW", SetLastError = true)]
    public static extern IntPtr GetWindowLongPtr64(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll", EntryPoint = "GetWindowLongW", SetLastError = true)]
    public static extern int GetWindowLong32(IntPtr hWnd, int nIndex);

    public static long GetExStyle(IntPtr hWnd) {
        if (IntPtr.Size == 8) {
            return (long)GetWindowLongPtr64(hWnd, -20);
        } else {
            return (long)GetWindowLong32(hWnd, -20);
        }
    }
}
'@

$handleVal = [Convert]::ToInt64($HwndHex, 16)
$handle = [IntPtr]$handleVal
$exStyle = [Win32WindowStyle]::GetExStyle($handle)

$isToolWindow = ($exStyle -band 0x80) -ne 0
$isAppWindow = ($exStyle -band 0x40000) -ne 0

[PSCustomObject]@{
    Hwnd = $HwndHex
    ExStyle = "0x$($exStyle.ToString('X'))"
    WS_EX_TOOLWINDOW = $isToolWindow
    WS_EX_APPWINDOW = $isAppWindow
} | ConvertTo-Json -Compress
