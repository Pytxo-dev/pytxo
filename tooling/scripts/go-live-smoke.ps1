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
$linkHealthRaw = (Invoke-WebRequest -Uri "$($linkBase.TrimEnd('/'))/health" -UseBasicParsing).Content
if ($linkHealthRaw.Trim() -ne "ok" -and $linkHealthRaw -notmatch '"status"\s*:\s*"ok"') {
    throw "unexpected link health: $linkHealthRaw"
}

Write-Host "==> Proxy health ($proxyBase)"
$proxyHealthRaw = (Invoke-WebRequest -Uri "$($proxyBase.TrimEnd('/'))/health" -UseBasicParsing).Content
if ($proxyHealthRaw.Trim() -ne "ok" -and $proxyHealthRaw -notmatch '"status"\s*:\s*"ok"') {
    throw "unexpected proxy health: $proxyHealthRaw"
}
Write-Host "  $proxyHealthRaw"
if ($proxyHealthRaw -notmatch 'deepseek') {
    Write-Host "  WARNING: deepseek not in providers_configured - set DEEPSEEK_API_KEY on pytxo-proxy" -ForegroundColor Yellow
}

if ($linkBase -and $session) {
    Write-Host "==> Entitlements status"
    $ent = Invoke-Link -Method GET -Uri "$($linkBase.TrimEnd('/'))/v1/entitlements/status"
    $ent | ConvertTo-Json -Compress | Write-Host

    Write-Host "==> Wallet balance"
    try {
        $wallet = Invoke-Link -Method GET -Uri "$($linkBase.TrimEnd('/'))/v1/wallet/balance"
        $wallet | ConvertTo-Json -Compress | Write-Host
    } catch {
        Write-Host "  WARNING: wallet/balance failed ($($_.Exception.Message)) - check Link DB migrations" -ForegroundColor Yellow
    }
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

$adminKey = $env:LINK_ADMIN_KEY
$orgId = $env:PYTXO_ORG_ID
if ($adminKey -and $orgId) {
    $adminHeaders = @{ Authorization = "Bearer $adminKey" }
    Write-Host "==> Enterprise org seats"
    (Invoke-RestMethod -Uri "$($linkBase.TrimEnd('/'))/v1/orgs/$orgId/seats" -Headers $adminHeaders) | ConvertTo-Json -Compress | Write-Host
    Write-Host "==> Enterprise org policy (GET)"
    (Invoke-RestMethod -Uri "$($linkBase.TrimEnd('/'))/v1/orgs/$orgId/policy" -Headers $adminHeaders) | ConvertTo-Json -Compress | Write-Host
    Write-Host "==> Enterprise org audit (recent)"
    (Invoke-RestMethod -Uri "$($linkBase.TrimEnd('/'))/v1/orgs/$orgId/audit?limit=5" -Headers $adminHeaders) | ConvertTo-Json -Compress | Write-Host
} elseif ($adminKey) {
    Write-Host "==> Skipping org seats/policy (set PYTXO_ORG_ID for Enterprise smoke)"
}

Write-Host "Go-live smoke OK (run_id=$runId)"
