param(
    [string]$OutputPath
)

# Deterministic control-plane benchmark against the real Pytxo monorepo.
# It measures scheduler output and five task-agent echo runs with max
# parallelism three in a temporary local clone. It does not claim model
# quality, token cost, or competitor outcomes.
#
# Usage:
#   powershell -File tooling/benchmarks/real-repo-control-plane.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Pytxo = if ($env:PYTXO_BIN) {
    [System.IO.Path]::GetFullPath($env:PYTXO_BIN)
} else {
    Join-Path $Root "target\debug\pytxo.exe"
}
if (-not $OutputPath) {
    $OutputPath = Join-Path $Root "tooling\benchmarks\results\control-plane-real-repo.json"
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)

Write-Host "Building the current Pytxo CLI source..."
cargo build -q -p pytxo-cli
if ($LASTEXITCODE -ne 0) {
    throw "Failed to build the current pytxo CLI"
}
if (-not (Test-Path -LiteralPath $Pytxo)) {
    throw "pytxo binary not found: $Pytxo"
}

$tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$WorkDir = Join-Path $tempRoot ("pytxo-real-repo-bench-" + [guid]::NewGuid().ToString("n"))
$resolvedWorkDir = [System.IO.Path]::GetFullPath($WorkDir)
if (-not $resolvedWorkDir.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing benchmark outside temp root: $resolvedWorkDir"
}

function Get-SameWaveCollisions($plan) {
    $collisions = 0
    foreach ($wave in @($plan.waves)) {
        $paths = @()
        foreach ($task in @($wave)) {
            if ($task.paths) {
                $paths += @($task.paths | ForEach-Object { [string]$_ })
            } elseif ($task.task -and $task.task.paths) {
                $paths += @($task.task.paths | ForEach-Object { [string]$_ })
            }
        }
        for ($i = 0; $i -lt $paths.Count; $i++) {
            for ($j = $i + 1; $j -lt $paths.Count; $j++) {
                if ($paths[$i] -eq $paths[$j]) {
                    $collisions++
                }
            }
        }
    }
    return $collisions
}

function Parse-Plan([string]$text) {
    $start = $text.IndexOf("{")
    if ($start -lt 0) {
        throw "dry-run did not emit a JSON plan"
    }
    return ($text.Substring($start) | ConvertFrom-Json)
}

function Invoke-PytxoCaptured([string[]]$Arguments) {
    $previousPreference = $ErrorActionPreference
    $ErrorActionPreference = "SilentlyContinue"
    try {
        $lines = @(& $Pytxo @Arguments 2>&1)
        $exitCode = $LASTEXITCODE
        $text = ($lines | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
        return [pscustomobject]@{
            exit_code = $exitCode
            text = $text
        }
    } finally {
        $ErrorActionPreference = $previousPreference
    }
}

$previousLocation = Get-Location
try {
    New-Item -ItemType Directory -Path $WorkDir | Out-Null
    $env:PYTXO_HOME = Join-Path $WorkDir "home"
    $env:PYTXO_TRUST_STORE = Join-Path $WorkDir "trusted-domains.json"
    New-Item -ItemType Directory -Path $env:PYTXO_HOME | Out-Null
    $Repo = Join-Path $WorkDir "repo"
    $Archive = Join-Path $WorkDir "source.tar"
    git -C $Root archive --format=tar HEAD -o $Archive
    if ($LASTEXITCODE -ne 0) {
        throw "git archive failed"
    }
    New-Item -ItemType Directory -Path $Repo | Out-Null
    tar -xf $Archive -C $Repo
    if ($LASTEXITCODE -ne 0) {
        throw "git archive extraction failed"
    }

    Set-Location $Repo
    git init -q
    git config user.email "pytxo-benchmark@local.invalid"
    git config user.name "Pytxo Benchmark"
    git add -A
    git commit -q -m "source snapshot"

    @'
max_agents = 3
permission_profile = "orbit"
isolation = "worktree"
execution_backend = "subprocess"
fail_fast = true
worktree_dir = ".pytxo/worktrees"
data_dir = ".pytxo/data"

[[agent]]
name = "benchmark"
paths = ["crates/**", "apps/**", "distribution/**"]
cli_adapter = "generic"

[[task]]
id = "desktop-stop-contract"
agent = "benchmark"
paths = ["apps/desktop/src/components/desktop2/OperationsScreen.svelte"]

[[task]]
id = "desktop-stop-evidence"
agent = "benchmark"
paths = ["apps/desktop/src/components/desktop2/OperationsScreen.svelte"]

[[task]]
id = "tauri-ipc"
agent = "benchmark"
paths = ["apps/desktop/src-tauri/src/ipc.rs"]

[[task]]
id = "orchestrate-stop-test"
agent = "benchmark"
paths = ["crates/pytxo-orchestrate/tests/stop_exact.rs"]

[[task]]
id = "release-notes"
agent = "benchmark"
paths = ["distribution/release-notes/v1.0.0.md"]
'@ | Set-Content -LiteralPath "pytxo.toml" -Encoding UTF8

    git add pytxo.toml
    git commit -q -m "benchmark config"

    $dryStopwatch = [Diagnostics.Stopwatch]::StartNew()
    $dryResult = Invoke-PytxoCaptured -Arguments @("run", "--config", "pytxo.toml", "--dry-run")
    $dryRaw = $dryResult.text
    $dryExit = $dryResult.exit_code
    $dryStopwatch.Stop()
    if ($dryExit -ne 0) {
        throw "dry-run failed with exit $dryExit`n$dryRaw"
    }
    $plan = Parse-Plan $dryRaw
    $conflicts = @($plan.conflicts).Count
    $waves = @($plan.waves).Count
    $sameWaveCollisions = Get-SameWaveCollisions $plan

    $trustResult = Invoke-PytxoCaptured -Arguments @("trust", "orbit")
    if ($trustResult.exit_code -ne 0) {
        throw "trust setup failed`n$($trustResult.text)"
    }

    $runStopwatch = [Diagnostics.Stopwatch]::StartNew()
    $runResult = Invoke-PytxoCaptured -Arguments @(
        "run",
        "--config",
        "pytxo.toml",
        "--cmd",
        "echo pytxo-real-repo-benchmark"
    )
    $runRaw = $runResult.text
    $runExit = $runResult.exit_code
    $runStopwatch.Stop()
    if ($runExit -ne 0) {
        throw "five-task control-plane run failed with exit $runExit`n$runRaw"
    }

    $statusResult = Invoke-PytxoCaptured -Arguments @(
        "status",
        "--config",
        "pytxo.toml",
        "--limit",
        "1",
        "--json"
    )
    $statusRaw = $statusResult.text
    $statusExit = $statusResult.exit_code
    if ($statusExit -ne 0) {
        throw "status failed with exit $statusExit`n$statusRaw"
    }
    $statusStart = $statusRaw.IndexOf("{")
    $status = if ($statusStart -ge 0) {
        $statusRaw.Substring($statusStart) | ConvertFrom-Json
    } else {
        $null
    }
    if ($status -and $status.runs) {
        foreach ($statusRun in @($status.runs)) {
            $statusRun.repo_root = "<temporary-source-snapshot>/repo"
        }
    }

    $primaryChanges = @(git status --porcelain)
    $worktrees = @(git worktree list --porcelain | Select-String "^worktree ").Count
    $isolatedWorkspaceDirectory = Join-Path $Repo ".pytxo\worktrees"
    $isolatedWorkspaceDirs = if (Test-Path -LiteralPath $isolatedWorkspaceDirectory) {
        @(Get-ChildItem -LiteralPath $isolatedWorkspaceDirectory -Directory).Count
    } else {
        0
    }
    $null = Invoke-PytxoCaptured -Arguments @(
        "stop",
        "--all",
        "--config",
        "pytxo.toml",
        "--cleanup-worktrees"
    )
    $cpu = try {
        (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name).Trim()
    } catch {
        $env:PROCESSOR_IDENTIFIER
    }

    $result = [ordered]@{
        schema_version = 1
        benchmark = "control-plane-real-repo"
        measured_at_utc = [DateTime]::UtcNow.ToString("o")
        repository = [ordered]@{
            source_commit = (git -C $Root rev-parse HEAD).Trim()
            clone_commit = (git rev-parse HEAD).Trim()
            tracked_files = @(git ls-files).Count
        }
        binary = [ordered]@{
            version = (Invoke-PytxoCaptured -Arguments @("--version")).text.Trim()
            base_commit = (git -C $Root rev-parse HEAD).Trim()
            source_worktree_dirty = -not [string]::IsNullOrWhiteSpace(
                (git -C $Root status --porcelain)
            )
        }
        host = [ordered]@{
            label = "local-windows-host"
            os = [System.Environment]::OSVersion.VersionString
            cpu = $cpu
            rustc = (rustc --version)
        }
        workload = [ordered]@{
            description = "Five deterministic tasks mapped to real v1.0.0 Desktop, Tauri IPC, test, and release-note paths. Two tasks intentionally overlap OperationsScreen.svelte."
            tasks = 5
            agents = 5
            max_parallel_agents = 3
            command = "echo pytxo-real-repo-benchmark"
            model_calls = 0
        }
        dry_run = [ordered]@{
            wall_time_ms = [long]$dryStopwatch.ElapsedMilliseconds
            conflicts_detected = $conflicts
            waves = $waves
            same_wave_path_collisions = $sameWaveCollisions
            plan = $plan
        }
        execution = [ordered]@{
            wall_time_ms = [long]$runStopwatch.ElapsedMilliseconds
            exit_code = $runExit
            registered_worktrees = $worktrees
            isolated_workspace_directories = $isolatedWorkspaceDirs
            primary_checkout_changes = $primaryChanges.Count
            status = $status
        }
        limitations = @(
            "This isolates Pytxo control-plane and worktree overhead; echo tasks do not measure model quality or coding-task completion.",
            "No competitor agent product was invoked, so this result supports no competitor outcome claim.",
            "One path overlap is deliberate and declared in the workload."
        )
    }

    $outputDirectory = Split-Path $OutputPath -Parent
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
    $result | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

    if ($conflicts -lt 1) {
        throw "Expected at least one declared-path conflict"
    }
    if ($waves -lt 2) {
        throw "Expected at least two scheduler waves"
    }
    if ($sameWaveCollisions -ne 0) {
        throw "Race Shield left $sameWaveCollisions same-wave path collisions"
    }
    if ($primaryChanges.Count -ne 0) {
        throw "Blast Shield benchmark changed the primary checkout"
    }

    Write-Host "control_plane_tasks=5"
    Write-Host "control_plane_agents=5"
    Write-Host "control_plane_max_parallel_agents=3"
    Write-Host "control_plane_conflicts_detected=$conflicts"
    Write-Host "control_plane_waves=$waves"
    Write-Host "control_plane_same_wave_collisions=$sameWaveCollisions"
    Write-Host "control_plane_dry_run_ms=$($dryStopwatch.ElapsedMilliseconds)"
    Write-Host "control_plane_execution_ms=$($runStopwatch.ElapsedMilliseconds)"
    Write-Host "control_plane_primary_checkout_changes=$($primaryChanges.Count)"
    Write-Host "control_plane_result=$OutputPath"
} finally {
    Set-Location $previousLocation
    if (Test-Path -LiteralPath $resolvedWorkDir) {
        $verified = [System.IO.Path]::GetFullPath($resolvedWorkDir)
        if (-not $verified.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing cleanup outside temp root: $verified"
        }
        try {
            Remove-Item -LiteralPath $verified -Recurse -Force -ErrorAction Stop
        } catch {
            if (Test-Path -LiteralPath $verified) {
                # PowerShell 5 can fail on Pytxo's intentionally deep projected
                # workspace paths. The verified extended path keeps cleanup
                # bounded to this benchmark's unique temp directory.
                [System.IO.Directory]::Delete("\\?\$verified", $true)
            }
        }
    }
}
