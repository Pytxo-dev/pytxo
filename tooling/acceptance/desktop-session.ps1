# WebView2 ignores WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS for this app on the
# hosted image, so set the per-user WebView2 policy for its executable instead
# (disposable runner only).
function Set-DesktopBrowserArguments([string]$Arguments) {
  $key = "HKCU:\Software\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments"
  New-Item -Force $key | Out-Null
  Set-ItemProperty -Path $key -Name "pytxo-desktop.exe" -Value $Arguments
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $Arguments
}

# Hosted runners run elevated, and WebView2 150+ refuses remote debugging for
# elevated hosts (MicrosoftEdge/WebView2Feedback#5639). Real users launch Desktop
# unelevated, so do the same: a limited scheduled task first, then a Basic User
# token, then the (unelevated) shell. The first launcher that yields CDP is kept.
$script:DesktopLaunchers = @("task", "runas", "explorer")
function Test-DesktopCdp([int]$Port, [int]$Seconds) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  while ((Get-Date) -lt $deadline) {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/version" -TimeoutSec 2 | Out-Null; return $true } catch { Start-Sleep -Seconds 2 }
  }
  $false
}
function Start-DesktopUnelevated([string]$Exe, [hashtable]$Environment, [string]$WorkingDirectory, [string]$Evidence, [string]$Name, [int]$Port = 9340) {
  $launcher = Join-Path $env:RUNNER_TEMP "launch-$Name.cmd"
  if ($launcher -match '\s') { throw "Launcher path must not contain spaces: $launcher" }
  $lines = @("@echo off")
  foreach ($key in $Environment.Keys) { $lines += "set `"$key=$($Environment[$key])`"" }
  if ($WorkingDirectory) { $lines += "cd /d `"$WorkingDirectory`"" }
  $lines += "start `"`" `"$Exe`""
  Set-Content -Encoding ascii -LiteralPath $launcher -Value $lines
  $app = $null
  foreach ($how in @($script:DesktopLaunchers)) {
    Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 2
    Write-Host "Launching Desktop ($Name) via $how"
    try {
      switch ($how) {
        "task" {
          $task = "pytxo-acceptance-$Name"
          $action = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/d /c $launcher"
          $principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
          Register-ScheduledTask -TaskName $task -Action $action -Principal $principal -Force | Out-Null
          Start-ScheduledTask -TaskName $task
        }
        "runas" { & runas.exe /trustlevel:0x20000 "cmd.exe /d /c $launcher" | Out-Host }
        "explorer" { & explorer.exe $launcher | Out-Host }
      }
    } catch { Write-Host "Launcher $how failed: $_"; continue }
    $deadline = (Get-Date).AddSeconds(30)
    do {
      Start-Sleep -Milliseconds 500
      $app = Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Select-Object -First 1
    } while (-not $app -and (Get-Date) -lt $deadline)
    if ($app -and (Test-DesktopCdp $Port 60)) {
      Write-Host "CDP ready for $Name via $how"
      $script:DesktopLaunchers = @($how) + @($script:DesktopLaunchers | Where-Object { $_ -ne $how })
      $script:DesktopLaunchedVia = $how
      return $app
    }
  }
  Wait-DesktopCdp $app $Evidence $Name $Port 5
}

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
    app_started = [bool]$App
    app_exited = $App -and $App.HasExited
    app_session = $(if ($App) { (Get-Process -Id $App.Id -ErrorAction SilentlyContinue).SessionId })
    runner_session = (Get-Process -Id $PID).SessionId
    main_window = $(if ($App) { (Get-Process -Id $App.Id -ErrorAction SilentlyContinue).MainWindowHandle.ToInt64() })
    uac = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" -ErrorAction SilentlyContinue | Select-Object EnableLUA, ConsentPromptBehaviorAdmin, FilterAdministratorToken
    launchers_tried = $script:DesktopLaunchers
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
