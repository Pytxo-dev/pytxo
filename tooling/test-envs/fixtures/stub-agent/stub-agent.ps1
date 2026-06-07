$ErrorActionPreference = "Stop"
Write-Host "=== pytxo stub-agent ==="
Write-Host "cwd=$(Get-Location)"
if ($env:PYTXO_CONTEXT_DIR) {
    Write-Host "PYTXO_CONTEXT_DIR=$($env:PYTXO_CONTEXT_DIR)"
    Write-Host "PYTXO_SIGNAL_CORE=$($env:PYTXO_SIGNAL_CORE)"
    $manifest = Join-Path $env:PYTXO_CONTEXT_DIR "manifest.json"
    if (Test-Path $manifest) {
        Write-Host "manifest.json:"
        Get-Content $manifest
    } else {
        Write-Host "manifest.json: (missing)"
        exit 1
    }
} else {
    Write-Host "PYTXO_CONTEXT_DIR=(unset)"
    exit 1
}
Write-Host "stub-agent OK"
