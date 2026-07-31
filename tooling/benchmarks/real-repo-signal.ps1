param(
    [string]$RepoPath,
    [int]$MinimumBytes = 1024,
    [string]$OutputPath,
    [string]$SourceRevision,
    [string]$RepositoryName
)

# Signal Core benchmark over tracked production source in a real repository.
# The default corpus is Pytxo itself. It excludes tests, generated output,
# dependencies, fixtures, and docs so the result cannot be inflated by copies.
#
# Usage:
#   powershell -File tooling/benchmarks/real-repo-signal.ps1
#   powershell -File tooling/benchmarks/real-repo-signal.ps1 `
#     -RepoPath C:\src\another-repo -OutputPath result.json

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
if (-not $RepoPath) {
    $RepoPath = $Root
}
$RepoPath = [System.IO.Path]::GetFullPath($RepoPath)
if (-not $RepositoryName) {
    $RepositoryName = Split-Path $RepoPath -Leaf
}

if (-not $OutputPath) {
    $OutputPath = Join-Path $Root "tooling\benchmarks\results\signal-real-repo.json"
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)

if (-not (Test-Path -LiteralPath (Join-Path $RepoPath ".git"))) {
    throw "RepoPath must be a git checkout: $RepoPath"
}

$reportName = if ($IsWindows -or $env:OS -eq "Windows_NT") {
    "scaffold_report.exe"
} else {
    "scaffold_report"
}
$reporter = Join-Path $Root "target\debug\examples\$reportName"

Write-Host "Building Signal Core benchmark helper..."
cargo build -q -p pytxo-signal --example scaffold_report
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $reporter)) {
    throw "Failed to build scaffold_report"
}

$supported = "\.(rs|ts|tsx|js|jsx|mjs|cjs|py|pyi|go|java|c|h|cpp|hpp|cc|cxx|rb)$"
$productionRoots = "^(crates|services|packages|apps/(desktop/src|desktop/src-tauri|web/src|web/scripts)|tooling)/"
$excluded = "(/tests?/|/e2e/|/fixtures?/|/vendor/|/node_modules/|/dist/|/target/|\.test\.|\.spec\.|\.stories\.)"

$tracked = @(git -C $RepoPath ls-files)
if ($LASTEXITCODE -ne 0) {
    throw "git ls-files failed for $RepoPath"
}

$candidates = @()
foreach ($relative in $tracked) {
    $normalized = $relative.Replace("\", "/")
    if ($normalized -notmatch $productionRoots) { continue }
    if ($normalized -notmatch $supported) { continue }
    if ($normalized -match $excluded) { continue }

    $full = Join-Path $RepoPath $relative
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { continue }
    $length = (Get-Item -LiteralPath $full).Length
    if ($length -lt $MinimumBytes) { continue }

    $candidates += [pscustomobject]@{
        relative = $normalized
        full = $full
        bytes = [long]$length
    }
}

if ($candidates.Count -eq 0) {
    throw "No supported production files matched the corpus rules"
}

Write-Host "Scaffolding $($candidates.Count) tracked production files..."
$rows = @()
foreach ($candidate in $candidates) {
    $raw = & $reporter $candidate.full low --stats-only
    if ($LASTEXITCODE -ne 0) {
        throw "scaffold_report failed for $($candidate.relative)"
    }
    $parsed = $raw | ConvertFrom-Json
    $rows += [pscustomobject]@{
        path = $candidate.relative
        language = [string]$parsed.stats.language
        original_bytes = [long]$parsed.stats.original_bytes
        scaffolded_bytes = [long]$parsed.stats.scaffolded_bytes
        reduction_pct = [math]::Round([double]$parsed.stats.token_reduction_pct, 4)
        fallback_raw = [bool]$parsed.fallback_raw
    }
}

$originalTotal = [long](($rows | Measure-Object -Property original_bytes -Sum).Sum)
$scaffoldedTotal = [long](($rows | Measure-Object -Property scaffolded_bytes -Sum).Sum)
$weightedReduction = if ($originalTotal -eq 0) {
    0
} else {
    (($originalTotal - $scaffoldedTotal) / $originalTotal) * 100
}

$sortedReductions = @($rows | ForEach-Object { [double]$_.reduction_pct } | Sort-Object)
$medianIndex = [math]::Floor($sortedReductions.Count / 2)
$median = if ($sortedReductions.Count % 2 -eq 0) {
    ($sortedReductions[$medianIndex - 1] + $sortedReductions[$medianIndex]) / 2
} else {
    $sortedReductions[$medianIndex]
}

$languages = @(
    $rows |
        Group-Object language |
        ForEach-Object {
            $languageOriginal = [long](($_.Group | Measure-Object -Property original_bytes -Sum).Sum)
            $languageScaffolded = [long](($_.Group | Measure-Object -Property scaffolded_bytes -Sum).Sum)
            [pscustomobject]@{
                language = $_.Name
                files = $_.Count
                original_bytes = $languageOriginal
                scaffolded_bytes = $languageScaffolded
                weighted_reduction_pct = [math]::Round(
                    (($languageOriginal - $languageScaffolded) / $languageOriginal) * 100,
                    4
                )
            }
        } |
        Sort-Object language
)

$dirty = -not [string]::IsNullOrWhiteSpace((git -C $RepoPath status --porcelain))
$cpu = try {
    (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name).Trim()
} catch {
    $env:PROCESSOR_IDENTIFIER
}

$result = [ordered]@{
    schema_version = 1
    benchmark = "signal-real-repo"
    measured_at_utc = [DateTime]::UtcNow.ToString("o")
    repository = [ordered]@{
        name = $RepositoryName
        commit = if ($SourceRevision) {
            $SourceRevision
        } else {
            (git -C $RepoPath rev-parse HEAD).Trim()
        }
        worktree_dirty = $dirty
    }
    host = [ordered]@{
        label = "local-windows-host"
        os = [System.Environment]::OSVersion.VersionString
        cpu = $cpu
        rustc = (rustc --version)
    }
    corpus = [ordered]@{
        rule = "Tracked production source under crates, services, packages, Desktop/Web source, and tooling; supported Signal grammars only; tests, fixtures, stories, specs, vendor, dependencies, and generated output excluded."
        minimum_bytes = $MinimumBytes
        files = $rows.Count
        original_bytes = $originalTotal
        scaffolded_bytes = $scaffoldedTotal
    }
    summary = [ordered]@{
        weighted_reduction_pct = [math]::Round($weightedReduction, 4)
        median_file_reduction_pct = [math]::Round($median, 4)
        fallback_files = @($rows | Where-Object fallback_raw).Count
    }
    languages = $languages
    files = @($rows | Sort-Object path)
}

$outputDirectory = Split-Path $OutputPath -Parent
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "signal_files=$($rows.Count)"
Write-Host "signal_original_bytes=$originalTotal"
Write-Host "signal_scaffolded_bytes=$scaffoldedTotal"
Write-Host "signal_weighted_reduction_pct=$([math]::Round($weightedReduction, 2))"
Write-Host "signal_median_file_reduction_pct=$([math]::Round($median, 2))"
Write-Host "signal_result=$OutputPath"
