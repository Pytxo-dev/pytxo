$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Fixture = Join-Path $Root "tests\fixtures\tiny-monorepo"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pytxo-overlay-bench-" + [guid]::NewGuid().ToString("n"))
$Pytxo = if ($env:PYTXO_BIN) { $env:PYTXO_BIN } else { Join-Path $Root "target\debug\pytxo.exe" }

function Format-Bytes([long]$Bytes) {
    if ($Bytes -ge 1GB) { return "{0:N2} GB" -f ($Bytes / 1GB) }
    if ($Bytes -ge 1MB) { return "{0:N2} MB" -f ($Bytes / 1MB) }
    if ($Bytes -ge 1KB) { return "{0:N2} KB" -f ($Bytes / 1KB) }
    return "$Bytes B"
}

function Get-DirBytes([string]$Path) {
    if (-not (Test-Path $Path)) { return 0 }
    return (Get-ChildItem -Path $Path -Recurse -File -ErrorAction SilentlyContinue |
        Measure-Object -Property Length -Sum).Sum
}

function Setup-Repo {
    New-Item -ItemType Directory -Path $WorkDir | Out-Null
    $script:Repo = Join-Path $WorkDir "repo"
    Copy-Item -Recurse $Fixture $Repo
    Set-Location $Repo
    git init -q
    git config user.email "pytxo@bench.local"
    git config user.name "Pytxo Bench"
    git add .
    git commit -q -m "init"
}

function Bench-Worktree {
    $wtBase = Join-Path $Repo ".pytxo\worktrees"
    New-Item -ItemType Directory -Path $wtBase -Force | Out-Null
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($i in 0, 1, 2) {
        git worktree add -q (Join-Path $wtBase "agent-$i") HEAD
    }
    $sw.Stop()
    $disk = Get-DirBytes $wtBase
    Write-Host "worktree_cold_start_ms=$($sw.ElapsedMilliseconds)"
    Write-Host "worktree_disk_bytes=$disk"
    Write-Host "worktree_disk_human=$(Format-Bytes $disk)"
}

function Bench-Overlay {
    @'
max_agents = 3
worktree_dir = ".pytxo/worktrees"
data_dir = ".pytxo/data"
isolation = "overlay"

[[task]]
id = "task-a"
agent = "builder"
paths = ["src/a.ts"]

[[task]]
id = "task-b"
agent = "builder"
paths = ["package.json"]

[[task]]
id = "task-c"
agent = "builder"
paths = ["package.json"]
'@ | Set-Content -Path pytxo.toml -Encoding utf8

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Pytxo init 2>$null
    & $Pytxo trust orbit 2>$null
    & $Pytxo run --config pytxo.toml --dry-run | Out-Null
    $sw.Stop()
    $wtBase = Join-Path $Repo ".pytxo\worktrees"
    $disk = Get-DirBytes $wtBase
    Write-Host "overlay_cold_start_ms=$($sw.ElapsedMilliseconds)"
    Write-Host "overlay_disk_bytes=$disk"
    Write-Host "overlay_disk_human=$(Format-Bytes $disk)"
}

Write-Host "=== Pytxo overlay vs worktree benchmark (tiny-monorepo) ==="
Write-Host "fixture: $Fixture"
Write-Host "pytxo:   $Pytxo"
if (-not (Test-Path $Pytxo)) {
    Write-Error "build pytxo first: cargo build -p pytxo-cli"
}

try {
    Setup-Repo
    Write-Host ""
    Write-Host "--- worktree isolation (3 git worktrees) ---"
    Bench-Worktree
    Remove-Item -Recurse -Force (Join-Path $Repo ".pytxo\worktrees") -ErrorAction SilentlyContinue
    Write-Host ""
    Write-Host "--- overlay isolation (dry-run planner path) ---"
    Bench-Overlay
    Write-Host ""
    Write-Host "=== summary (paste into competitive-benchmarks.md pinned table) ==="
} finally {
    Remove-Item -Recurse -Force $WorkDir -ErrorAction SilentlyContinue
}
