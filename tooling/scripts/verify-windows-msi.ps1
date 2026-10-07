#Requires -Version 7.0
<#
Extract an MSI into a fresh evidence directory and check every packaged PE for
unprovided MSVC runtime dependencies. This does not install or launch the app.
The extraction and report are retained for inspection, including on failure.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$InstallerPath,
    [string]$EvidenceDirectory = (Join-Path ([IO.Path]::GetTempPath()) ('pytxo-msi-check-' + [guid]::NewGuid()))
)

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'MSI extraction requires Windows.' }
$installer = Get-Item -LiteralPath $InstallerPath
if ($installer.PSIsContainer -or $installer.Extension -ine '.msi') {
    throw 'InstallerPath must name an MSI file.'
}
$installerHash = (Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$evidencePath = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $evidencePath) { throw "Evidence directory already exists: $evidencePath" }
New-Item -ItemType Directory -Path $evidencePath | Out-Null
$payloadPath = Join-Path $evidencePath 'payload'
$logPath = Join-Path $evidencePath 'extract.log'
$msiArguments = @('/a', ('"' + $installer.FullName + '"'), '/qn',
    ('TARGETDIR="' + $payloadPath + '"'), '/l*v', ('"' + $logPath + '"'))
$extraction = Start-Process -FilePath "$env:SystemRoot\System32\msiexec.exe" `
    -ArgumentList $msiArguments -WindowStyle Hidden -Wait -PassThru
if ($extraction.ExitCode -ne 0) {
    throw "MSI administrative extraction failed with exit $($extraction.ExitCode); see $logPath"
}
$binaries = @(Get-ChildItem -LiteralPath $payloadPath -File -Recurse |
    Where-Object { $_.Extension -in @('.exe', '.dll') } | Sort-Object FullName)
if ($binaries.Count -eq 0) { throw 'The MSI contains no executable or DLL payload to inspect.' }
$checker = Join-Path $PSScriptRoot 'verify-windows-runtime.mjs'
$inspection = & node $checker @($binaries.FullName)
$inspectionExit = $LASTEXITCODE
$inspection | Set-Content -LiteralPath (Join-Path $evidencePath 'runtime-imports.json') -Encoding utf8
if ($inspectionExit -ne 0) {
    throw "Packaged runtime dependency check failed; see $evidencePath"
}
if ((Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $installerHash) {
    throw 'The MSI changed during inspection.'
}
$report = [ordered]@{
    at = [DateTime]::UtcNow.ToString('o')
    installer = $installer.FullName
    installer_sha256 = $installerHash
    extracted_binary_count = $binaries.Count
    runtime_imports = Join-Path $evidencePath 'runtime-imports.json'
    result = 'passed'
    scope = 'All extracted EXE/DLL files have no versioned MSVC runtime imports; installation and launch require separate checks.'
}
$report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidencePath 'result.json') -Encoding utf8
$report | ConvertTo-Json -Depth 5
