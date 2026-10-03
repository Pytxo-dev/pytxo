# Shared helpers: wait for a launched Desktop to expose its WebView over CDP,
# and, if it never does, record why (process, session, WebView2 runtime, the
# screen, and Pytxo's own data folder) before failing.
function Wait-DesktopCdp([System.Diagnostics.Process]$App, [string]$Evidence, [string]$Name, [int]$Port = 9340, [int]$Seconds = 120) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  while ((Get-Date) -lt $deadline) {
    try {
      $version = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/version" -TimeoutSec 2
      Write-Host "CDP ready for ${Name}: $($version.Browser)"
      return
    } catch { Start-Sleep -Seconds 2 }
  }
  $dir = Join-Path $Evidence "diagnostics-$Name"
  New-Item -ItemType Directory -Force $dir | Out-Null
  $runtime = Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue | Select-Object -First 1
  [ordered]@{
    app_exited = $App.HasExited
    app_exit_code = if ($App.HasExited) { $App.ExitCode } else { $null }
    app_session = (Get-Process -Id $App.Id -ErrorAction SilentlyContinue).SessionId
    runner_session = (Get-Process -Id $PID).SessionId
    main_window = (Get-Process -Id $App.Id -ErrorAction SilentlyContinue).MainWindowHandle.ToInt64()
    webview2_runtime = $runtime.pv
    webview_processes = @(Get-Process msedgewebview2 -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Id) session $($_.SessionId)" })
    listening = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object { $_.OwningProcess -in @((Get-Process msedgewebview2, pytxo-desktop -ErrorAction SilentlyContinue).Id) } | ForEach-Object { "$($_.LocalAddress):$($_.LocalPort)" })
    env_args = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    webview_command_lines = @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -notmatch '--type=' } | ForEach-Object { $_.CommandLine })
    edge_policies = @("HKLM:\SOFTWARE\Policies\Microsoft\Edge", "HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2", "HKCU:\SOFTWARE\Policies\Microsoft\Edge\WebView2") | ForEach-Object { if (Test-Path $_) { "$_ " + ((Get-ItemProperty $_ | Select-Object * -ExcludeProperty PS* | ConvertTo-Json -Compress)) } }
  } | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 (Join-Path $dir "state.json")
  try {
    Add-Type -AssemblyName System.Windows.Forms, System.Drawing
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    [System.Drawing.Graphics]::FromImage($bitmap).CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $dir "screen.png"))
  } catch { Set-Content (Join-Path $dir "screen-error.txt") "$_" }
  if ($env:PYTXO_HOME -and (Test-Path $env:PYTXO_HOME)) {
    Get-ChildItem -Recurse -File $env:PYTXO_HOME | Where-Object { $_.Extension -in ".log", ".txt", ".json" } | Select-Object -First 20 | Copy-Item -Destination $dir -ErrorAction SilentlyContinue
  }
  Get-Content (Join-Path $dir "state.json")
  throw "Desktop ($Name) never exposed its WebView over CDP; see $dir"
}
