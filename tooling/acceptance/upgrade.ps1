# Retained-data upgrade on a disposable Windows runner: install the public
# v1.2.1 MSI, use it once with the default data locations, then install the
# candidate over it and check the version, the data and the WebView profile.
param(
  [Parameter(Mandatory)] [string]$Msi,
  [Parameter(Mandatory)] [string]$Evidence,
  [string]$PreviousUrl = "https://github.com/Pytxo-dev/pytxo-releases/releases/download/v1.2.1/pytxo-desktop-windows-x64.msi"
)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "desktop-session.ps1")
New-Item -ItemType Directory -Force $Evidence | Out-Null
$Evidence = (Resolve-Path $Evidence).Path
$summary = [ordered]@{ previous = $PreviousUrl; candidate_sha256 = (Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash }

function Get-Installed {
  Get-ChildItem HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall, HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall, HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall -ErrorAction SilentlyContinue |
    Get-ItemProperty | Where-Object { $_.DisplayName -like "Pytxo Desktop*" }
}
function Install([string]$Path, [string]$Log) {
  $run = Start-Process msiexec.exe -ArgumentList @("/i", "`"$Path`"", "/qn", "/norestart", "/l*v", "`"$Log`"") -Wait -PassThru
  if ($run.ExitCode -ne 0) { throw "msiexec /i $Path exited $($run.ExitCode)" }
}
function Find-Exe {
  Get-ChildItem -Recurse -File -Filter pytxo-desktop.exe "C:\Program Files", "$env:LOCALAPPDATA\Programs" -ErrorAction SilentlyContinue | Select-Object -First 1
}
function Use-Desktop([string]$Name, [string[]]$ProbeArgs) {
  $exe = Find-Exe
  # Default data locations of the standard-user tester (see desktop-session.ps1).
  Set-DesktopBrowserArguments "--remote-debugging-port=0"
  $app = Start-DesktopAsTester $exe.FullName @{} $null $Evidence $Name
  try {
    node (Join-Path $PSScriptRoot "probe.mjs") --cdp $script:DesktopCdpPort --out $Evidence --name $Name @ProbeArgs | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "Probe $Name failed" }
  } finally {
    Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 3
  }
  $exe
}
function Snapshot-Data {
  $dir = Join-Path (Get-DesktopTesterProfile) ".pytxo"
  if (-not (Test-Path $dir)) { return @() }
  Get-ChildItem -Recurse -File $dir | ForEach-Object { [ordered]@{ path = $_.FullName.Substring($dir.Length + 1); bytes = $_.Length } }
}

$previous = Join-Path $env:RUNNER_TEMP "pytxo-previous.msi"
Invoke-WebRequest -Uri $PreviousUrl -OutFile $previous -UseBasicParsing
$summary.previous_sha256 = (Get-FileHash -LiteralPath $previous -Algorithm SHA256).Hash
Install $previous (Join-Path $Evidence "install-previous.log")
$summary.before = [ordered]@{ installed = @(Get-Installed | ForEach-Object { "$($_.DisplayName) $($_.DisplayVersion)" }) }
$exe = Use-Desktop "previous" @("--set-marker", "v1.2.1", "--onboard")
$summary.before.file_version = $exe.VersionInfo.FileVersion
$before = @(Snapshot-Data)
$summary.before.data_files = $before.Count

Install $Msi (Join-Path $Evidence "install-candidate.log")
$summary.after = [ordered]@{ installed = @(Get-Installed | ForEach-Object { "$($_.DisplayName) $($_.DisplayVersion)" }) }
$exe = Use-Desktop "candidate" @()
$summary.after.file_version = $exe.VersionInfo.FileVersion
$after = @(Snapshot-Data)
$summary.after.data_files = $after.Count
$missing = @($before | Where-Object { $path = $_.path; -not ($after | Where-Object { $_.path -eq $path }) } | ForEach-Object { $_.path })
$summary.after.missing_data_files = $missing
$probe = Get-Content (Join-Path $Evidence "candidate.json") -Raw | ConvertFrom-Json
$summary.after.webview_marker = $probe.markerBefore
$summary.after.headings = $probe.headings

$summary | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $Evidence "upgrade.json")
Get-Content (Join-Path $Evidence "upgrade.json")
if ($summary.after.installed.Count -ne 1 -or $summary.after.installed[0] -notlike "*1.2.2*") { throw "Upgrade did not leave exactly the candidate installed" }
if ($missing.Count) { throw "Upgrade lost data files: $($missing -join ', ')" }
if ($probe.markerBefore -ne "v1.2.1") { throw "WebView storage was not retained across the upgrade" }
