# Install pytxo CLI from GitHub Releases into %LOCALAPPDATA%\Programs\pytxo
$ErrorActionPreference = "Stop"

$Repo = if ($env:PYTXO_REPO) { $env:PYTXO_REPO } else { "Pytxo-dev/pytxo-releases" }
$InstallDir = if ($env:PYTXO_INSTALL_DIR) { $env:PYTXO_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\pytxo" }
$Architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Architecture -ne "X64") {
    throw "Unsupported Pytxo Windows release architecture: $Architecture (v1.2.1 provides Windows x64)"
}
$Asset = "pytxo-windows-x64.exe"

function Get-LatestVersion {
    if ($env:PYTXO_VERSION) { return $env:PYTXO_VERSION }
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

$Version = Get-LatestVersion
if (-not $Version) { throw "Could not resolve a Pytxo release version" }
if (-not $Version.StartsWith("v")) { $Version = "v$Version" }
$ExpectedVersion = $Version.Substring(1)
$ReleaseBase = if ($env:PYTXO_RELEASE_BASE_URL) { $env:PYTXO_RELEASE_BASE_URL.TrimEnd('/') } else { "https://github.com/$Repo/releases/download" }
$Url = "$ReleaseBase/$Version/$Asset"
$ChecksumUrl = "$ReleaseBase/$Version/SHA256SUMS.txt"
$Dest = Join-Path $InstallDir "pytxo.exe"
$TempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$TempDir = Join-Path $TempRoot ("pytxo-install-" + [Guid]::NewGuid().ToString("N"))
$TempBinary = Join-Path $TempDir "pytxo.exe"
$TempChecksums = Join-Path $TempDir "SHA256SUMS.txt"

try {
    New-Item -ItemType Directory -Path $TempDir | Out-Null
    Write-Host "Downloading pytxo $Version ($Asset) for verification"
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $TempChecksums -UseBasicParsing
    Invoke-WebRequest -Uri $Url -OutFile $TempBinary -UseBasicParsing

    $ExpectedHash = $null
    foreach ($Line in Get-Content -LiteralPath $TempChecksums) {
        $Parts = $Line -split '\s+', 2
        if ($Parts.Count -eq 2 -and $Parts[1].TrimStart('*') -eq $Asset -and $Parts[0] -match '^[0-9a-fA-F]{64}$') {
            $ExpectedHash = $Parts[0].ToLowerInvariant()
            break
        }
    }
    if (-not $ExpectedHash) { throw "SHA256SUMS.txt has no exact entry for $Asset" }
    $ActualHash = (Get-FileHash -LiteralPath $TempBinary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualHash -ne $ExpectedHash) { throw "SHA-256 mismatch for $Asset" }

    $VersionOutput = (& $TempBinary --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $VersionOutput -notmatch ('^pytxo\s+v?' + [Regex]::Escape($ExpectedVersion) + '$')) {
        throw "Downloaded binary reported unexpected version: $VersionOutput"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $Backup = "$Dest.previous"
    if (Test-Path -LiteralPath $Backup) { Remove-Item -LiteralPath $Backup -Force }
    if (Test-Path -LiteralPath $Dest) { Move-Item -LiteralPath $Dest -Destination $Backup }
    try {
        Move-Item -LiteralPath $TempBinary -Destination $Dest
        if (Test-Path -LiteralPath $Backup) { Remove-Item -LiteralPath $Backup -Force }
    }
    catch {
        if (-not (Test-Path -LiteralPath $Dest) -and (Test-Path -LiteralPath $Backup)) {
            Move-Item -LiteralPath $Backup -Destination $Dest
        }
        throw
    }
}
finally {
    $ResolvedTemp = [IO.Path]::GetFullPath($TempDir)
    if ($ResolvedTemp.StartsWith($TempRoot, [StringComparison]::OrdinalIgnoreCase) -and (Test-Path -LiteralPath $ResolvedTemp)) {
        Remove-Item -LiteralPath $ResolvedTemp -Recurse -Force
    }
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH (restart shell)."
}

$InstalledVersion = (& $Dest --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw "Installed Pytxo version check failed" }
Write-Host "Installed $InstalledVersion. Run 'pytxo doctor' inside a repository."
