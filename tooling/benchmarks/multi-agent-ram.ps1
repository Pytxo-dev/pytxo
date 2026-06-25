$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Fixture = Join-Path $Root "tests\fixtures\tiny-monorepo"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pytxo-ram-bench-" + [guid]::NewGuid().ToString("n"))
$Pytxo = if ($env:PYTXO_BIN) { $env:PYTXO_BIN } else { Join-Path $Root "target\debug\pytxo.exe" }

try {
    New-Item -ItemType Directory -Path $WorkDir | Out-Null
    $Repo = Join-Path $WorkDir "repo"
    Copy-Item -Recurse $Fixture $Repo
    Set-Location $Repo
    git init -q
    git config user.email "pytxo@bench.local"
    git config user.name "Pytxo Bench"
    git add .
    git commit -q -m "init"

    @'
max_agents = 3
worktree_dir = ".pytxo/worktrees"
data_dir = ".pytxo/data"

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

    & $Pytxo init
    & $Pytxo trust orbit

    $proc = Start-Process -FilePath $Pytxo -ArgumentList @(
        "run", "--config", "pytxo.toml", "--cmd", "Start-Sleep -Seconds 30"
    ) -PassThru -NoNewWindow
    Start-Sleep -Seconds 2

    $peakKb = 0
    $samples = 0
    while (-not $proc.HasExited -and $samples -lt 10) {
        $proc.Refresh()
        $ws = $proc.WorkingSet64
        $kb = [math]::Floor($ws / 1024)
        if ($kb -gt $peakKb) { $peakKb = $kb }
        $samples++
        Start-Sleep -Seconds 1
    }

    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }

    $peakMb = [math]::Floor($peakKb / 1024)
    Write-Host "=== multi-agent RAM benchmark ==="
    Write-Host "samples=$samples"
    Write-Host "peak_rss_kb=$peakKb"
    Write-Host "peak_rss_mb=$peakMb"
    Write-Host "OK: RAM sample complete"
} finally {
    Remove-Item -Recurse -Force $WorkDir -ErrorAction SilentlyContinue
}
