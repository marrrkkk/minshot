# Shared Win32 test helpers for driving minshot end-to-end.
# Dot-source from the minshot workdir: . .\scripts\w32.ps1
$c = @"
using System; using System.Text; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class W32 {
  public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc e, IntPtr p);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern int GetWindowThreadProcessId(IntPtr h, out int pid);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, UIntPtr dwExtraInfo);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public struct RECT { public int L,T,R,B; }
  public const int MOVE = 0x0001, LEFTDOWN = 0x0002, LEFTUP = 0x0004, ABS = 0x8000;
  public static List<string> All() {
    var list = new List<string>();
    EnumWindows((h, p) => {
      var sb = new StringBuilder(256); GetWindowText(h, sb, 256);
      int pid = 0; GetWindowThreadProcessId(h, out pid);
      RECT r; GetWindowRect(h, out r);
      list.Add(h + "|" + pid + "|" + (IsWindowVisible(h) ? "V" : "H") + "|" + sb.ToString() + "|rect=" + r.L + "," + r.T + "," + r.R + "," + r.B);
      return true;
    }, IntPtr.Zero);
    return list;
  }
  public static void MoveAbs(int x, int y) {
    int nx = (int)(x * 65535.0 / 1920.0), ny = (int)(y * 65535.0 / 1080.0);
    mouse_event(MOVE | ABS, nx, ny, 0, UIntPtr.Zero);
  }
  public static void Click(int x, int y) {
    MoveAbs(x, y); System.Threading.Thread.Sleep(350);
    mouse_event(LEFTDOWN, 0, 0, 0, System.UIntPtr.Zero); System.Threading.Thread.Sleep(120);
    mouse_event(LEFTUP, 0, 0, 0, System.UIntPtr.Zero); System.Threading.Thread.Sleep(900);
  }
  public static void Drag(int x0, int y0, int x1, int y1) {
    MoveAbs(x0, y0); System.Threading.Thread.Sleep(250);
    mouse_event(LEFTDOWN, 0, 0, 0, System.UIntPtr.Zero); System.Threading.Thread.Sleep(250);
    MoveAbs(x1, y1); System.Threading.Thread.Sleep(250);
    mouse_event(LEFTUP, 0, 0, 0, System.UIntPtr.Zero); System.Threading.Thread.Sleep(900);
  }
}
"@
Add-Type -TypeDefinition $c
function MsWin($pid1, $pattern) { [W32]::All() | Select-String "$pid1" | Select-String $pattern }
function MsRect($line) {
  $m = [regex]::Match($line.ToString(), 'rect=(\d+),(\d+),(\d+),(\d+)')
  @([int]$m.Groups[1].Value, [int]$m.Groups[2].Value, [int]$m.Groups[3].Value, [int]$m.Groups[4].Value)
}
