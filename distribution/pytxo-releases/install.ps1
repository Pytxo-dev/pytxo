# Install pytxo CLI from GitHub Releases into %LOCALAPPDATA%\Programs\pytxo
$ErrorActionPreference = "Stop"

$Repo = if ($env:PYTXO_REPO) { $env:PYTXO_REPO } else { "Pytxo-dev/pytxo-releases" }
$InstallDir = if ($env:PYTXO_INSTALL_DIR) { $env:PYTXO_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\pytxo" }
$Asset = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "pytxo-windows-arm64.exe" } else { "pytxo-windows-x64.exe" }

function Get-LatestVersion {
    if ($env:PYTXO_VERSION) { return $env:PYTXO_VERSION }
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

$Version = Get-LatestVersion
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$Dest = Join-Path $InstallDir "pytxo.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Write-Host "Installing pytxo $Version ($Asset) -> $Dest"
Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH (restart shell)."
}

& $Dest doctor
Write-Host "Done."
