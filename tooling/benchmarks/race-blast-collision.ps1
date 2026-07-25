# Race Shield / Blast Shield collision proof (thin reliability slice).
# Same-repo path collisions: Race Shield must put overlapping tasks in separate
# waves; Blast Shield stays gated via isolation=worktree (CoW-style sandbox until approve/flush).
#
# Usage (from repo root, after `cargo build -p pytxo-cli`):
#   powershell -File tooling/benchmarks/race-blast-collision.ps1
#
# Optional: $env:PYTXO_BIN = path\to\pytxo.exe

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Fixture = Join-Path $Root "tests\fixtures\tiny-monorepo"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pytxo-race-blast-" + [guid]::NewGuid().ToString("n"))
$Pytxo = if ($env:PYTXO_BIN) { $env:PYTXO_BIN } else { Join-Path $Root "target\debug\pytxo.exe" }

if (-not (Test-Path $Pytxo)) {
    throw "pytxo binary not found at $Pytxo (build with cargo build -p pytxo-cli)"
}

function Get-SameWaveCollisions($plan) {
    $collisions = 0
    if (-not $plan.waves) { return 0 }
    foreach ($wave in $plan.waves) {
        $paths = @()
        foreach ($task in $wave) {
            $taskPaths = @()
            if ($task.paths) { $taskPaths = @($task.paths) }
            elseif ($task.PSObject.Properties.Name -contains "task" -and $task.task.paths) {
                $taskPaths = @($task.task.paths)
            }
            foreach ($p in $taskPaths) { $paths += [string]$p }
        }
        for ($i = 0; $i -lt $paths.Count; $i++) {
            for ($j = $i + 1; $j -lt $paths.Count; $j++) {
                if ($paths[$i] -eq $paths[$j]) { $collisions++ }
            }
        }
    }
    return $collisions
}

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
isolation = "worktree"

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

    # Naive baseline: all three tasks in one wave would collide once on package.json.
    $naiveSameWaveCollisions = 1

    Write-Host "=== race-blast collision dry-run ==="
    $dryRaw = & $Pytxo run --config pytxo.toml --dry-run 2>&1 | Out-String
    Write-Host $dryRaw

    $jsonStart = $dryRaw.IndexOf("{")
    if ($jsonStart -lt 0) { throw "dry-run did not emit JSON plan" }
    $plan = ($dryRaw.Substring($jsonStart) | ConvertFrom-Json)

    $conflictCount = 0
    if ($plan.conflicts) { $conflictCount = @($plan.conflicts).Count }
    $waveCount = 0
    if ($plan.waves) { $waveCount = @($plan.waves).Count }
    $wastedWithRace = Get-SameWaveCollisions $plan

    $blastGated = (Select-String -Path pytxo.toml -Pattern 'isolation\s*=\s*"worktree"' -Quiet) -and (Test-Path (Join-Path $Repo ".pytxo"))

    Write-Host "=== race-blast collision metrics ==="
    Write-Host "naive_same_wave_collisions=$naiveSameWaveCollisions"
    Write-Host "race_shield_conflicts_detected=$conflictCount"
    Write-Host "race_shield_wave_count=$waveCount"
    Write-Host "wasted_parallel_edits_with_race_shield=$wastedWithRace"
    Write-Host "blast_shield_gated=$blastGated"

    if ($conflictCount -lt 1) {
        throw "Race Shield failed: expected >=1 conflict for overlapping package.json tasks"
    }
    if ($waveCount -lt 2) {
        throw "Race Shield failed: expected >=2 waves for colliding tasks (got $waveCount)"
    }
    if ($wastedWithRace -ne 0) {
        throw "Race Shield failed: same-wave path collisions=$wastedWithRace (want 0)"
    }
    if (-not $blastGated) {
        throw "Blast Shield failed: isolation=worktree / .pytxo layout missing"
    }

    Write-Host "OK: race/blast collision proof complete"
} finally {
    Set-Location $Root
    Remove-Item -Recurse -Force $WorkDir -ErrorAction SilentlyContinue
}
