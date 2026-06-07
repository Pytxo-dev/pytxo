# Shared pytxo binary resolution for test-env scripts.
# Dot-source: . (Join-Path $PSScriptRoot "..\lib\Resolve-PytxoBin.ps1")

function Get-PytxoRepoRoot {
    # tooling/test-envs/{lib|recipes|scaffold.ps1} -> repo root is three levels up from test-envs
    $testEnvRoot = if ($PSScriptRoot -match '[\\/]lib$') {
        Split-Path $PSScriptRoot -Parent
    } elseif ($PSScriptRoot -match '[\\/]recipes$') {
        Split-Path $PSScriptRoot -Parent
    } else {
        $PSScriptRoot
    }
    Split-Path (Split-Path $testEnvRoot -Parent) -Parent
}

function Resolve-PytxoBin {
    if ($env:PYTXO_BIN -and (Test-Path -LiteralPath $env:PYTXO_BIN)) {
        return (Resolve-Path -LiteralPath $env:PYTXO_BIN).Path
    }

    $onPath = Get-Command pytxo -ErrorAction SilentlyContinue
    if ($onPath) {
        return $onPath.Source
    }

    try {
        $npmBin = (& npm bin -g 2>$null).Trim()
        if ($npmBin) {
            foreach ($name in @("pytxo.cmd", "pytxo.ps1", "pytxo")) {
                $candidate = Join-Path $npmBin $name
                if (Test-Path -LiteralPath $candidate) {
                    return (Resolve-Path -LiteralPath $candidate).Path
                }
            }
        }
    } catch {
        # npm not installed — fall through to local build paths
    }

    $root = Get-PytxoRepoRoot
    foreach ($rel in @("target\release\pytxo.exe", "target\debug\pytxo.exe")) {
        $candidate = Join-Path $root $rel
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    throw @"
pytxo not found. Install one of:
  npm i -g pytxo
  cargo build -p pytxo-cli --release   (from repo root)
Or set PYTXO_BIN to the full path of pytxo.exe
"@
}
