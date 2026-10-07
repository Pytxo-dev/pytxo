# Cloud acceptance for one candidate MSI on a disposable Windows runner:
# clean install, the full mixed-agent journey with stand-in agent CLIs, then
# layout passes at 150% and 200% device scale. Writes everything to -Evidence.
param(
  [Parameter(Mandatory)] [string]$Msi,
  [Parameter(Mandatory)] [string]$Evidence
)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "desktop-session.ps1")
$root = Resolve-Path (Join-Path $PSScriptRoot "../..")
New-Item -ItemType Directory -Force $Evidence | Out-Null
$Evidence = (Resolve-Path $Evidence).Path
$summary = [ordered]@{ msi = (Split-Path $Msi -Leaf); msi_sha256 = (Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash }

# --- Display: the hosted desktop starts at 1024x768. ---
try { Set-DisplayResolution -Width 1920 -Height 1080 -Force -ErrorAction Stop } catch { Write-Host "Display resolution unchanged: $_" }
Add-Type -AssemblyName System.Windows.Forms
$summary.screen = "{0}x{1}" -f [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height

# --- Clean install. ---
$log = Join-Path $Evidence "install.log"
$install = Start-Process msiexec.exe -ArgumentList @("/i", "`"$Msi`"", "/qn", "/norestart", "/l*v", "`"$log`"") -Wait -PassThru
if ($install.ExitCode -ne 0) { throw "msiexec /i exited $($install.ExitCode)" }
$uninstall = Get-ChildItem HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall, HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall, HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall -ErrorAction SilentlyContinue |
  Get-ItemProperty | Where-Object { $_.DisplayName -like "Pytxo Desktop*" } | Select-Object -First 1
if (-not $uninstall) { throw "No uninstall entry for Pytxo Desktop" }
$exe = $null
if ($uninstall.InstallLocation) { $exe = Get-ChildItem -Recurse -File -Filter pytxo-desktop.exe -LiteralPath $uninstall.InstallLocation -ErrorAction SilentlyContinue | Select-Object -First 1 }
if (-not $exe) { $exe = Get-ChildItem -Recurse -File -Filter pytxo-desktop.exe "C:\Program Files", "$env:LOCALAPPDATA\Programs" -ErrorAction SilentlyContinue | Select-Object -First 1 }
if (-not $exe) { throw "Installed pytxo-desktop.exe not found" }
$shortcut = Get-ChildItem -Recurse -File -Filter "*.lnk" "$env:ProgramData\Microsoft\Windows\Start Menu", "$env:APPDATA\Microsoft\Windows\Start Menu" -ErrorAction SilentlyContinue | Where-Object { $_.Name -like "Pytxo*" } | Select-Object -First 1
$summary.install = [ordered]@{
  display_name = $uninstall.DisplayName
  display_version = $uninstall.DisplayVersion
  exe = $exe.FullName
  exe_sha256 = (Get-FileHash -LiteralPath $exe.FullName -Algorithm SHA256).Hash
  file_version = $exe.VersionInfo.FileVersion
  start_menu_shortcut = [bool]$shortcut
  authenticode = (Get-AuthenticodeSignature -LiteralPath $exe.FullName).Status.ToString()
}

# --- Stand-in agent CLIs first on PATH (see stand-in-agent.mjs). ---
$bin = Join-Path $env:RUNNER_TEMP "pytxo-stand-in-agents"
New-Item -ItemType Directory -Force $bin | Out-Null
Copy-Item (Join-Path $PSScriptRoot "stand-in-agent.mjs") $bin
# Matching tasks replay the recorded 2 October workers' output and files.
Copy-Item (Join-Path $PSScriptRoot "replay-20261002.json") (Join-Path $bin "replay.json")
foreach ($cli in "codex", "claude", "cursor-agent", "opencode", "agy") {
  Set-Content -Encoding ascii (Join-Path $bin "$cli.cmd") "@node `"%~dp0stand-in-agent.mjs`" $cli %*"
}
$env:PATH = "$bin;$env:PATH"
# Desktop runs as the standard-user tester (see desktop-session.ps1) on folders this
# elevated runner created; Pytxo strips GIT_* variables, so trust them system-wide.
git config --system --add safe.directory "*"

function Start-Desktop([string]$Name) {
  $dir = Join-Path $env:RUNNER_TEMP "pytxo-$Name"
  New-Item -ItemType Directory -Force "$dir\home", "$dir\webview" | Out-Null
  # The tester needs write access to this elevated runner's temp tree.
  icacls $env:RUNNER_TEMP /grant "*S-1-1-0:(OI)(CI)F" /T /C /Q | Out-Null
  $env:PYTXO_HOME = "$dir\home"
  Start-DesktopAsTester $exe.FullName @{
    PYTXO_HOME = "$dir\home"
    WEBVIEW2_USER_DATA_FOLDER = "$dir\webview"
    PATH = $env:PATH
  } "$dir\home" $Evidence $Name
}
function Stop-Desktop($Process) {
  if ($Process -and -not $Process.HasExited) { Stop-Process -Id $Process.Id -Force; $Process.WaitForExit(15000) | Out-Null }
  Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 2
}

# --- Full journey at 100%. ---
$fixture = Join-Path $env:RUNNER_TEMP "fleet\taskboard"
& (Join-Path $root "docs\demo\fleet\setup.ps1") -Path $fixture
$baselineTests = (& npm --prefix $fixture test 2>&1 | Out-String)
$app = Start-Desktop "journey"
try {
  python (Join-Path $PSScriptRoot "journey.py") --out (Join-Path $Evidence "journey") --mode full --repo $fixture --mission (Join-Path $root "docs\demo\fleet\mission.txt") --team "Claude Code,Cursor Agent,OpenCode,Antigravity"
  if ($LASTEXITCODE -ne 0) { throw "Journey failed" }
} finally { Stop-Desktop $app }
# The applied task board itself, opened in Edge as its user would.
node (Join-Path $PSScriptRoot "result-app.mjs") --out (Join-Path $Evidence "journey") --repo $fixture
if ($LASTEXITCODE -ne 0) { throw "The applied task board did not run" }

# Apply must have written exactly the reviewed files, and the project's tests must still pass.
$journey = Get-Content (Join-Path $Evidence "journey\receipt.json") -Raw | ConvertFrom-Json
$changed = @(git -C $fixture status --porcelain | ForEach-Object { $_.Substring(3).Trim('"') }) | Sort-Object
$reviewed = @($journey.checks.review.files) | Sort-Object
$tests = & npm --prefix $fixture test 2>&1 | Out-String
$summary.journey = [ordered]@{
  result = $journey.result
  fleet = $journey.checks.fleet
  reviewed_files = $reviewed
  changed_files = $changed
  changed_matches_reviewed = (($changed -join "|") -eq ($reviewed -join "|"))
  operator_note_left = Test-Path (Join-Path $fixture "operator-note.txt")
  tests_after_apply = if ($LASTEXITCODE -eq 0) { "passed" } else { "failed" }
}
# How many applied files are byte-identical to the recorded workers' files (replay coverage).
$recorded = @((Get-Content (Join-Path $PSScriptRoot "replay-20261002.json") -Raw | ConvertFrom-Json).tasks.PSObject.Properties.Value | ForEach-Object { $_.files.PSObject.Properties })
$summary.journey.replayed_files = "{0}/{1}" -f @($recorded | Where-Object { (Get-Content -Raw -LiteralPath (Join-Path $fixture $_.Name) -ErrorAction SilentlyContinue) -ceq $_.Value }).Count, $recorded.Count
Set-Content -Encoding utf8 (Join-Path $Evidence "fixture-tests.txt") "--- before ---`n$baselineTests`n--- after Apply ---`n$tests"
git -C $fixture diff > (Join-Path $Evidence "fixture-applied.diff")

# --- Windows display scaling at 150% and 200%: the real per-monitor DPI, set live
# as the Settings app does, with the DPI Desktop's window reports as evidence. ---
. (Join-Path $PSScriptRoot "display-scale.ps1")
$summary.display_scaling = [ordered]@{}
# 200% needs a larger display than hosted runners offer (their maximum is 175%);
# it is recorded as unavailable there, not passed, and stays a real-hardware check.
foreach ($percent in 150, 175, 200) {
  $name = "dpi-$percent"
  $entry = [ordered]@{}
  try {
    $entry.display = Set-DisplayScale $percent @("1920x1080", "2560x1600", "2560x1440", "2048x1536")
    $entry.screen = "{0}x{1}" -f [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height
    $app = Start-Desktop $name
    try {
      Start-Sleep -Seconds 2
      $entry.window_dpi = [PytxoDisplayScale]::GetDpiForWindow((Get-Process -Id $app.Id).MainWindowHandle)
      python (Join-Path $PSScriptRoot "journey.py") --out (Join-Path $Evidence $name) --mode layout
      $entry.result = if ($LASTEXITCODE -ne 0) { "failed" } elseif ($entry.window_dpi -ne [math]::Round(96 * $percent / 100)) { "failed: window DPI $($entry.window_dpi)" } else { "passed" }
    } finally { Stop-Desktop $app }
  } catch { $entry.result = if ("$_" -like "No listed resolution allows*") { "unavailable on this display" } else { "failed: $_" } }
  $summary.display_scaling["$percent"] = $entry
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $Evidence "acceptance.json")
Get-Content (Join-Path $Evidence "acceptance.json")
$failed = -not $summary.journey.changed_matches_reviewed -or $summary.journey.tests_after_apply -ne "passed" -or $summary.journey.operator_note_left -or @($summary.display_scaling.Values | Where-Object { $_.result -ne "passed" -and $_.result -ne "unavailable on this display" }).Count -or $summary.display_scaling["150"].result -ne "passed" -or $summary.display_scaling["175"].result -ne "passed"
if ($failed) { throw "Acceptance checks failed; see acceptance.json" }
