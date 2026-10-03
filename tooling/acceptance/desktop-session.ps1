# Shared by run-acceptance.ps1 and upgrade.ps1, on disposable cloud runners only.

# Device scale for the layout passes. WebView2 150+ takes browser arguments for
# this app only from the machine policy (MicrosoftEdge/WebView2Feedback#5640).
function Set-DesktopBrowserArguments([string]$Arguments) {
  $key = "HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments"
  if (-not (Test-Path $key)) { New-Item -Force $key | Out-Null }
  Set-ItemProperty -Path $key -Name "pytxo-desktop.exe" -Value $Arguments
}
# Hosted runners sign in as a full-token administrator; real users run Desktop as
# standard users, so acceptance does too: a throwaway local account on the
# disposable runner, with a random password that is never printed or stored.
function Get-DesktopTester {
  if ($script:DesktopTester) { return $script:DesktopTester }
  $name = "pytxo-tester"
  $plain = (-join ((65..90) + (97..122) + (48..57) | Get-Random -Count 28 | ForEach-Object { [char]$_ })) + "!a7Q"
  $computer = [ADSI]"WinNT://$env:COMPUTERNAME,computer"
  $user = $computer.Create("User", $name)
  [void]$user.SetPassword($plain)
  [void]$user.SetInfo()
  try { [void]([ADSI]"WinNT://$env:COMPUTERNAME/Users,group").Add("WinNT://$env:COMPUTERNAME/$name,user") } catch { }
  $script:DesktopTester = New-Object System.Management.Automation.PSCredential ("$env:COMPUTERNAME\$name", (ConvertTo-SecureString $plain -AsPlainText -Force))
  $script:DesktopTester
}

# The tester's profile folder (created by its first launch), for default data locations.
function Get-DesktopTesterProfile {
  $sid = (New-Object System.Security.Principal.NTAccount("$env:COMPUTERNAME\pytxo-tester")).Translate([System.Security.Principal.SecurityIdentifier]).Value
  (Get-CimInstance Win32_UserProfile | Where-Object { $_.SID -eq $sid }).LocalPath
}

# Starts Desktop as the tester with $Environment on top of the tester's own
# profile variables, and returns its process once its window is up. If no window
# appears, records why (session, WebView2 runtime, the screen, Pytxo's logs).
function Start-DesktopAsTester([string]$Exe, [hashtable]$Environment, [string]$WorkingDirectory, [string]$Evidence, [string]$Name) {
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
  Start-Process -FilePath cmd.exe -ArgumentList "/d /c $launcher" -Credential (@(Get-DesktopTester) | Where-Object { $_ -is [pscredential] } | Select-Object -First 1) -LoadUserProfile -WorkingDirectory $env:SystemRoot -WindowStyle Hidden
  $deadline = (Get-Date).AddSeconds(90)
  do {
    Start-Sleep -Seconds 1
    $app = Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
  } while (-not $app -and (Get-Date) -lt $deadline)
  if ($app) { Write-Host "Desktop window up for $Name"; return $app }
  Save-DesktopDiagnostics $Evidence $Name
  throw "Desktop ($Name) never showed a window; see diagnostics-$Name"
}

function Save-DesktopDiagnostics([string]$Evidence, [string]$Name) {
  $dir = Join-Path $Evidence "diagnostics-$Name"
  New-Item -ItemType Directory -Force $dir | Out-Null
  $runtime = Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue | Select-Object -First 1
  [ordered]@{
    runner_session = (Get-Process -Id $PID).SessionId
    desktop = @(Get-Process pytxo-desktop -IncludeUserName -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Id) session $($_.SessionId) $($_.UserName) window $($_.MainWindowHandle)" })
    webview2_runtime = $runtime.pv
    webview_command_lines = @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -notmatch '--type=' } | ForEach-Object { $_.CommandLine })
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
}
