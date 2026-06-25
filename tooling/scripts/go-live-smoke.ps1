# Go-live smoke: Link health, entitlements, run-ledger round-trip.
# Usage:
#   $env:LINK_BASE = "https://link.pytxo.com"; $env:LINK_API_KEY = "..."; .\tooling\scripts\go-live-smoke.ps1
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\..")

$linkBase = if ($env:LINK_BASE) { $env:LINK_BASE } elseif ($env:LINK_BASE_URL) { $env:LINK_BASE_URL } else { "http://127.0.0.1:8787" }
$proxyBase = if ($env:PROXY_BASE) { $env:PROXY_BASE } elseif ($env:PROXY_BASE_URL) { $env:PROXY_BASE_URL } else { "http://127.0.0.1:8790" }
$session = if ($env:PYTXO_ULTRA_SESSION) { $env:PYTXO_ULTRA_SESSION } else { $env:LINK_API_KEY }
$domainId = if ($env:PYTXO_DOMAIN_ID) { $env:PYTXO_DOMAIN_ID } else { "go-live-smoke" }
$runId = "go-live-$(Get-Date -UFormat %s)"

function Invoke-Link {
    param([string]$Method, [string]$Uri, [string]$Body = $null)
    $headers = @{}
    if ($session) { $headers["Authorization"] = "Bearer $session" }
    if ($Body) {
        $headers["Content-Type"] = "application/json"
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $headers -Body $Body
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $headers
}

Write-Host "==> Link health ($linkBase)"
$health = Invoke-RestMethod -Uri "$($linkBase.TrimEnd('/'))/health"
if ($health -ne "ok") { throw "unexpected health: $health" }

Write-Host "==> Proxy health ($proxyBase)"
$proxyHealth = Invoke-RestMethod -Uri "$($proxyBase.TrimEnd('/'))/health"
if ($proxyHealth -ne "ok") { throw "unexpected proxy health: $proxyHealth" }

if ($linkBase -and $session) {
    Write-Host "==> Entitlements status"
    $ent = Invoke-Link -Method GET -Uri "$($linkBase.TrimEnd('/'))/v1/entitlements/status"
    $ent | ConvertTo-Json -Compress | Write-Host

    Write-Host "==> Wallet balance"
    $wallet = Invoke-Link -Method GET -Uri "$($linkBase.TrimEnd('/'))/v1/wallet/balance"
    $wallet | ConvertTo-Json -Compress | Write-Host
} else {
    Write-Host "==> Skipping entitlements/wallet (set LINK_BASE + LINK_API_KEY or PYTXO_ULTRA_SESSION)"
}

Write-Host "==> Run ledger round-trip (idempotent retry simulation)"
$startBody = (@{ domain_id = $domainId; run_id = $runId } | ConvertTo-Json -Compress)
$endBody = (@{
    domain_id = $domainId
    run_id    = $runId
    usage     = @{
        tokens_in_billed = 12
        tokens_in_sent   = 8
        tokens_out       = 4
        saved_tokens     = 4
        cost_micro_usd   = 250
    }
} | ConvertTo-Json -Compress)

foreach ($attempt in 1, 2) {
    Invoke-Link -Method POST -Uri "$($linkBase.TrimEnd('/'))/v1/runs/start" -Body $startBody | Out-Null
    Write-Host "  runs/start attempt $attempt OK"
}
foreach ($attempt in 1, 2) {
    Invoke-Link -Method POST -Uri "$($linkBase.TrimEnd('/'))/v1/runs/end" -Body $endBody | Out-Null
    Write-Host "  runs/end attempt $attempt OK"
}

Write-Host "Go-live smoke OK (run_id=$runId)"
