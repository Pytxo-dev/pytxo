# Windows display scaling for acceptance on disposable runners: the same live
# change the Settings app makes (per-monitor DPI through DisplayConfig, no
# sign-out), plus the DPI a given window actually renders at. The DisplayConfig
# DPI device-info types (-3 get, -4 set) are undocumented; scale is relative to
# the display's recommended step.
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class PytxoDisplayScale {
  [StructLayout(LayoutKind.Sequential)] public struct LUID { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] public struct Header { public int type; public int size; public LUID adapterId; public uint id; }
  [StructLayout(LayoutKind.Sequential)] public struct GetScale { public Header header; public int minRel; public int curRel; public int maxRel; }
  [StructLayout(LayoutKind.Sequential)] public struct SetScale { public Header header; public int rel; }
  [StructLayout(LayoutKind.Sequential)] public struct Source { public LUID adapterId; public uint id; public uint modeIdx; public uint flags; }
  [StructLayout(LayoutKind.Sequential)] public struct Rational { public uint num; public uint den; }
  [StructLayout(LayoutKind.Sequential)] public struct Target { public LUID adapterId; public uint id; public uint modeIdx; public int tech; public int rotation; public int scaling; public Rational refresh; public int scan; public int available; public uint flags; }
  [StructLayout(LayoutKind.Sequential)] public struct Path { public Source source; public Target target; public uint flags; }
  [StructLayout(LayoutKind.Sequential, Size = 64)] public struct Mode { public int infoType; public uint id; public LUID adapterId; }
  [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
  [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] Path[] paths, ref uint modeCount, [Out] Mode[] modes, IntPtr topology);
  [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref GetScale info);
  [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref SetScale info);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
  static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

  static Source Primary() {
    uint pathCount, modeCount;
    GetDisplayConfigBufferSizes(2, out pathCount, out modeCount);
    var paths = new Path[pathCount]; var modes = new Mode[modeCount];
    int status = QueryDisplayConfig(2, ref pathCount, paths, ref modeCount, modes, IntPtr.Zero);
    if (status != 0) throw new Exception("QueryDisplayConfig " + status);
    return paths[0].source;
  }
  static GetScale Read(Source source) {
    var info = new GetScale();
    info.header.type = -3; info.header.size = Marshal.SizeOf(typeof(GetScale)); info.header.adapterId = source.adapterId; info.header.id = source.id;
    int status = DisplayConfigGetDeviceInfo(ref info);
    if (status != 0) throw new Exception("DisplayConfigGetDeviceInfo " + status);
    return info;
  }
  public static string Describe() {
    var info = Read(Primary());
    int recommended = Math.Abs(info.minRel);
    return "current " + Steps[recommended + info.curRel] + "%, allowed " + Steps[0] + "-" + Steps[recommended + info.maxRel] + "%";
  }
  public static void Set(int percent) {
    var source = Primary();
    var info = Read(source);
    int recommended = Math.Abs(info.minRel), step = Array.IndexOf(Steps, percent);
    if (step < 0 || step - recommended > info.maxRel) throw new Exception(percent + "% is not available at this resolution (" + Describe() + ")");
    var change = new SetScale();
    change.header.type = -4; change.header.size = Marshal.SizeOf(typeof(SetScale)); change.header.adapterId = source.adapterId; change.header.id = source.id;
    change.rel = step - recommended;
    int status = DisplayConfigSetDeviceInfo(ref change);
    if (status != 0) throw new Exception("DisplayConfigSetDeviceInfo " + status);
  }
}
"@

# Picks the smallest listed resolution that allows $Percent, then applies it.
function Set-DisplayScale([int]$Percent, [string[]]$Resolutions = @("1920x1080")) {
  foreach ($resolution in $Resolutions) {
    $width, $height = $resolution -split "x"
    try { Set-DisplayResolution -Width $width -Height $height -Force -ErrorAction Stop | Out-Null } catch { Write-Host "Resolution $resolution unavailable: $_"; continue }
    Start-Sleep -Seconds 2
    try { [PytxoDisplayScale]::Set($Percent); Start-Sleep -Seconds 2; return [PytxoDisplayScale]::Describe() } catch { Write-Host "Scale $Percent% at ${resolution}: $_" }
  }
  throw "No listed resolution allows $Percent% display scaling"
}
