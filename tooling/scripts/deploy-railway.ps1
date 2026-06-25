# Deploy Pytxo Link + Cloud Sandbox to Railway (run after: railway login)
# Usage: .\tooling\scripts\deploy-railway.ps1
#
# Railway auto-names projects. Link first: railway link -p <your-project>
# First-time setup is mostly via the Railway dashboard — see distribution/railway/README.md.
# This script generates API keys and triggers deploys when the Railway CLI is linked.

param(
    [string]$ProjectName = ""
)

$ErrorActionPreference = "Stop"

$railway = Get-Command railway -ErrorAction SilentlyContinue
if (-not $railway) {
    throw "Railway CLI not found. Install: npm i -g @railway/cli  OR  https://docs.railway.com/guides/cli"
}

$linkApi = [guid]::NewGuid().ToString("N")
$linkAdmin = [guid]::NewGuid().ToString("N")
$cloudApi = [guid]::NewGuid().ToString("N")

$proxyApi = [guid]::NewGuid().ToString("N")

Write-Host ""
Write-Host "=== Generated API keys (copy into Railway + Vercel) ===" -ForegroundColor Cyan
Write-Host "  LINK_API_KEY=$linkApi"
Write-Host "  LINK_ADMIN_KEY=$linkAdmin"
Write-Host "  CLOUD_API_KEY=$cloudApi"
Write-Host "  PROXY_API_KEY=$proxyApi"
Write-Host ""
Write-Host "Vercel (pytxo.com project):"
Write-Host "  LINK_ADMIN_URL=https://link.pytxo.com"
Write-Host "  LINK_ADMIN_KEY=$linkAdmin"
Write-Host ""

$repoRoot = Resolve-Path "$PSScriptRoot\..\.."
Set-Location $repoRoot

Write-Host "Ensure you are logged in: railway login" -ForegroundColor Yellow
if ($ProjectName) {
    Write-Host "Link this repo (once): railway link -p $ProjectName"
} else {
    Write-Host "Link this repo (once): railway link   # pick your project from the list"
}
Write-Host ""
Write-Host "Dashboard checklist (distribution/railway/README.md):"
Write-Host "  - PostgreSQL plugin"
Write-Host "  - pytxo-link service (Dockerfile services/pytxo-link/Dockerfile, config services/pytxo-link/railway.toml)"
Write-Host "  - pytxo-cloud-sandbox service (Dockerfile services/pytxo-cloud-sandbox/Dockerfile)"
Write-Host "  - pytxo-proxy service (Dockerfile services/pytxo-proxy/Dockerfile, config services/pytxo-proxy/railway.toml)"
Write-Host "  - DATABASE_URL=`${{Postgres.DATABASE_URL}} on pytxo-link"
Write-Host "  - Custom domains: link.pytxo.com, cloud.pytxo.com, proxy.pytxo.com"
Write-Host ""

$confirm = Read-Host "Railway project linked and services created? Deploy now? [y/N]"
if ($confirm -notmatch '^[Yy]') {
    Write-Host "Skipped deploy. Set secrets in Railway, then run: railway up -s pytxo-link && railway up -s pytxo-cloud-sandbox"
    exit 0
}

Write-Host "Deploying pytxo-link..."
railway up -s pytxo-link

Write-Host "Deploying pytxo-cloud-sandbox..."
railway up -s pytxo-cloud-sandbox

Write-Host "Deploying pytxo-proxy..."
railway up -s pytxo-proxy

Write-Host ""
Write-Host "Done. Verify:"
Write-Host "  curl https://link.pytxo.com/health"
Write-Host "  curl https://cloud.pytxo.com/health"
Write-Host "  curl https://proxy.pytxo.com/health"
