[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\..\.."))
$sessionsRoot = Join-Path $repositoryRoot "target\release-demo"
$sessionName = "commit-boundary-{0}" -f (Get-Date -Format "yyyyMMdd-HHmmss-fff")
$destination = Join-Path $sessionsRoot $sessionName
$demoHome = Join-Path $sessionsRoot ("{0}-home" -f $sessionName)
$demoWebViewHome = Join-Path $demoHome "webview2"
$source = Join-Path $repositoryRoot "examples\pytxo-first-mission"

New-Item -ItemType Directory -Path $destination -Force | Out-Null
New-Item -ItemType Directory -Path $demoHome -Force | Out-Null
$env:PYTXO_HOME = $demoHome
$env:WEBVIEW2_USER_DATA_FOLDER = $demoWebViewHome
Get-ChildItem -LiteralPath $source -Force | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $destination -Recurse -Force
}
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "pytxo.demo.toml") -Destination (Join-Path $destination "pytxo.toml") -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "demo-agent.mjs") -Destination (Join-Path $destination "demo-agent.mjs") -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "demo.gitignore") -Destination (Join-Path $destination ".gitignore") -Force

& git -C $destination init -q
if ($LASTEXITCODE -ne 0) { throw "Could not initialize the demo repository." }
& git -C $destination branch -M main
if ($LASTEXITCODE -ne 0) { throw "Could not select the demo main branch." }
& git -C $destination add .
if ($LASTEXITCODE -ne 0) { throw "Could not stage the demo baseline." }
& git -C $destination -c user.name="Pytxo Demo" -c user.email="demo@pytxo.local" -c commit.gpgsign=false commit -q -m "Create deterministic Pytxo release demo"
if ($LASTEXITCODE -ne 0) { throw "Could not commit the demo baseline." }

Push-Location $destination
try {
    & npm test | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "The demo baseline tests failed." }
} finally {
    Pop-Location
}

Write-Output $destination
