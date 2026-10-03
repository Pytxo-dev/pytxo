# Hosted runners sign in as a full-token administrator, and WebView2 150+ drops
# environment and registry browser arguments (so remote debugging) for elevated
# hosts (MicrosoftEdge/WebView2Feedback#5639). Real users run Desktop as standard
# users, so acceptance does too: a throwaway local account on the disposable
# runner, with a random password that is never printed or stored.
function Get-DesktopTester {
  if ($script:DesktopTester) { return $script:DesktopTester }
  $name = "pytxo-tester"
  $plain = (-join ((65..90) + (97..122) + (48..57) | Get-Random -Count 28 | ForEach-Object { [char]$_ })) + "!a7Q"
  $computer = [ADSI]"WinNT://$env:COMPUTERNAME,computer"
  $user = $computer.Create("User", $name)
  $user.SetPassword($plain)
  $user.SetInfo()
  try { ([ADSI]"WinNT://$env:COMPUTERNAME/Users,group").Add("WinNT://$env:COMPUTERNAME/$name,user") } catch { }
  $script:DesktopTester = New-Object System.Management.Automation.PSCredential ("$env:COMPUTERNAME\$name", (ConvertTo-SecureString $plain -AsPlainText -Force))
  $script:DesktopTester
}

# The tester's profile folder (created by its first launch), for default data locations.
function Get-DesktopTesterProfile {
  $sid = (New-Object System.Security.Principal.NTAccount("$env:COMPUTERNAME\pytxo-tester")).Translate([System.Security.Principal.SecurityIdentifier]).Value
  (Get-CimInstance Win32_UserProfile | Where-Object { $_.SID -eq $sid }).LocalPath
}

function Test-DesktopCdp([int]$Port, [int]$Seconds) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  while ((Get-Date) -lt $deadline) {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/version" -TimeoutSec 2 | Out-Null; return $true } catch { Start-Sleep -Seconds 2 }
  }
  $false
}

# Starts Desktop as the tester with exactly $Environment (plus the tester's own
# profile variables) and returns its process once the WebView answers on CDP.
function Start-DesktopAsTester([string]$Exe, [hashtable]$Environment, [string]$WorkingDirectory, [string]$Evidence, [string]$Name, [int]$Port = 9340) {
  Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 2
  $launcher = Join-Path $env:RUNNER_TEMP "launch-$Name.cmd"
  if ($launcher -match '\s') { throw "Launcher path must not contain spaces: $launcher" }
  $lines = @("@echo off")
  foreach ($key in $Environment.Keys) { $lines += "set `"$key=$($Environment[$key])`"" }
  if ($WorkingDirectory) { $lines += "cd /d `"$WorkingDirectory`"" }
  $lines += "start `"`" `"$Exe`""
  Set-Content -Encoding ascii -LiteralPath $launcher -Value $lines
  icacls $launcher /grant "*S-1-1-0:RX" | Out-Null
  Start-Process -FilePath cmd.exe -ArgumentList "/d /c $launcher" -Credential (Get-DesktopTester) -LoadUserProfile -WorkingDirectory $env:SystemRoot -WindowStyle Hidden
  $deadline = (Get-Date).AddSeconds(45)
  do {
    Start-Sleep -Milliseconds 500
    $app = Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Select-Object -First 1
  } while (-not $app -and (Get-Date) -lt $deadline)
  if ($app -and (Test-DesktopCdp $Port 90)) { Write-Host "CDP ready for $Name"; return $app }
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
    app_user = $(if ($App) { (Get-Process -Id $App.Id -IncludeUserName -ErrorAction SilentlyContinue).UserName })
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
