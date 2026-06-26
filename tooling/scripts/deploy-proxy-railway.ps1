# Deploy pytxo-proxy to Railway (run after: railway login && railway link)
# Usage: .\tooling\scripts\deploy-proxy-railway.ps1

param(
    [switch]$SkipDeploy
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path "$PSScriptRoot\..\.."
Set-Location $repoRoot

$railway = Get-Command railway -ErrorAction SilentlyContinue
if (-not $railway) {
    throw "Railway CLI not found. Install: npm i -g @railway/cli"
}

Write-Host "Checking Railway auth..." -ForegroundColor Cyan
railway whoami 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Not logged in. Run: railway login"
}

$services = railway service list 2>&1 | Out-String
if ($services -notmatch "pytxo-proxy") {
    Write-Host "Creating pytxo-proxy service (empty, deploy via railpack)..." -ForegroundColor Cyan
    railway add --service pytxo-proxy --json | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "railway add failed - run: railway login" }
}

Write-Host "Configuring pytxo-proxy variables..." -ForegroundColor Cyan
$linkKey = (railway variables --json -s pytxo-link 2>&1 | ConvertFrom-Json).LINK_API_KEY
if (-not $linkKey) {
    throw "Could not read LINK_API_KEY from pytxo-link. Set it manually on pytxo-proxy."
}

railway variables set `
    PROXY_REQUIRE_AUTH=1 `
    LINK_BASE_URL=https://link.pytxo.com `
    LINK_API_KEY=$linkKey `
    RAILPACK_CONFIG_FILE=services/pytxo-proxy/railpack.json `
    -s pytxo-proxy

$providerSet = $false
if ($env:DEEPSEEK_API_KEY) {
    railway variables set "DEEPSEEK_API_KEY=$env:DEEPSEEK_API_KEY" -s pytxo-proxy
    $providerSet = $true
    Write-Host "Set DEEPSEEK_API_KEY from environment." -ForegroundColor Green
}
if ($env:ANTHROPIC_API_KEY) {
    railway variables set "ANTHROPIC_API_KEY=$env:ANTHROPIC_API_KEY" -s pytxo-proxy
    $providerSet = $true
}

if (-not $providerSet) {
    Write-Host ""
    Write-Host "WARNING: DEEPSEEK_API_KEY not set in this shell." -ForegroundColor Yellow
    Write-Host "Set DEEPSEEK_API_KEY on pytxo-proxy in Railway dashboard (required for Ultra default):"
    Write-Host "  Railway -> pytxo-proxy -> Variables -> DEEPSEEK_API_KEY"
    Write-Host "  Optional later: ANTHROPIC_API_KEY, OPENAI_API_KEY, GOOGLE_API_KEY"
    Write-Host ""
}

if ($SkipDeploy) {
    Write-Host "Skipped deploy (-SkipDeploy). Run: railway up -s pytxo-proxy"
    exit 0
}

Write-Host "Deploying pytxo-proxy..." -ForegroundColor Cyan
railway up -s pytxo-proxy

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Green
Write-Host "  1. Railway dashboard -> pytxo-proxy -> Networking -> add custom domain proxy.pytxo.com"
Write-Host "  2. CNAME proxy + TXT _railway-verify.proxy at your DNS host (see Railway modal)"
Write-Host "  3. Verify: curl https://proxy.pytxo.com/health  (providers_configured should include deepseek)"
Write-Host "  4. Run: .\tooling\scripts\go-live-smoke.ps1"
