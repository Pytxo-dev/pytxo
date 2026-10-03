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
foreach ($cli in "codex", "claude", "cursor-agent", "opencode", "agy") {
  Set-Content -Encoding ascii (Join-Path $bin "$cli.cmd") "@node `"%~dp0stand-in-agent.mjs`" $cli %*"
}
$env:PATH = "$bin;$env:PATH"

function Start-Desktop([string]$Name, [string]$Scale) {
  $dir = Join-Path $env:RUNNER_TEMP "pytxo-$Name"
  New-Item -ItemType Directory -Force "$dir\home", "$dir\webview" | Out-Null
  # Desktop runs as the standard-user tester (see desktop-session.ps1): it needs
  # write access to this elevated runner's temp tree and must trust its repositories.
  icacls $env:RUNNER_TEMP /grant "*S-1-1-0:(OI)(CI)F" /T /C /Q | Out-Null
  $env:PYTXO_HOME = "$dir\home"
  Start-DesktopAsTester $exe.FullName @{
    PYTXO_HOME = "$dir\home"
    WEBVIEW2_USER_DATA_FOLDER = "$dir\webview"
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9340 --force-device-scale-factor=$Scale"
    GIT_CONFIG_COUNT = "1"
    GIT_CONFIG_KEY_0 = "safe.directory"
    GIT_CONFIG_VALUE_0 = "*"
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
$app = Start-Desktop "journey" "1"
try {
  node (Join-Path $PSScriptRoot "journey.mjs") --cdp 9340 --out (Join-Path $Evidence "journey") --mode full --repo $fixture --mission (Join-Path $root "docs\demo\fleet\mission.txt") --team "Claude Code,Cursor Agent,OpenCode,Antigravity"
  if ($LASTEXITCODE -ne 0) { throw "Journey failed" }
} finally { Stop-Desktop $app }

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
Set-Content -Encoding utf8 (Join-Path $Evidence "fixture-tests.txt") "--- before ---`n$baselineTests`n--- after Apply ---`n$tests"
git -C $fixture diff > (Join-Path $Evidence "fixture-applied.diff")

# --- Layout at 150% and 200% device scale (WebView emulation, not Windows display scaling). ---
$summary.layout = [ordered]@{}
foreach ($scale in "1.5", "2") {
  $app = Start-Desktop "layout-$scale" $scale
  try {
    node (Join-Path $PSScriptRoot "journey.mjs") --cdp 9340 --out (Join-Path $Evidence "layout-$scale") --mode layout
    $summary.layout[$scale] = if ($LASTEXITCODE -eq 0) { "passed" } else { "failed" }
  } finally { Stop-Desktop $app }
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $Evidence "acceptance.json")
Get-Content (Join-Path $Evidence "acceptance.json")
$failed = -not $summary.journey.changed_matches_reviewed -or $summary.journey.tests_after_apply -ne "passed" -or $summary.journey.operator_note_left -or ($summary.layout.Values -contains "failed")
if ($failed) { throw "Acceptance checks failed; see acceptance.json" }
