$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Fixture = Join-Path $Root "tests\fixtures\tiny-monorepo"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pytxo-bench-" + [guid]::NewGuid().ToString("n"))
$Pytxo = if ($env:PYTXO_BIN) { $env:PYTXO_BIN } else { Join-Path $Root "target\debug\pytxo.exe" }
. (Join-Path $Root "tooling\test-envs\lib\Resolve-PytxoBin.ps1")

New-Item -ItemType Directory -Path $WorkDir | Out-Null
$Repo = Join-Path $WorkDir "repo"
Copy-Item -Recurse $Fixture $Repo
Set-Location $Repo

git init -q
git config user.email "pytxo@bench.local"
git config user.name "Pytxo Bench"
Invoke-PytxoChecked $Pytxo init

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

# The benchmark exercises reviewed Apply, whose trust contract requires a clean
# committed baseline. Include Pytxo's generated ignore rules and benchmark
# config in that baseline so harness setup is not mistaken for operator drift.
git add .
git commit -q -m "init"

Write-Host "=== dry-run plan ==="
Invoke-PytxoChecked $Pytxo run --config pytxo.toml --dry-run

Write-Host "=== execute ==="
Invoke-PytxoChecked $Pytxo trust orbit
Invoke-PytxoChecked $Pytxo run --config pytxo.toml --cmd "echo pytxo-agent"

Write-Host "=== status ==="
Invoke-PytxoChecked $Pytxo status

Write-Host "OK: benchmark complete"
