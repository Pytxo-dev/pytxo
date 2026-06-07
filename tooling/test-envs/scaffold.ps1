param(
    [ValidateSet("echo", "context", "antigravity", "claude", "codex")]
    [string]$Profile = "echo"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\Resolve-PytxoBin.ps1")

$Root = Get-PytxoRepoRoot
$Fixture = Join-Path $PSScriptRoot "fixtures\base-repo"
$Configs = Join-Path $PSScriptRoot "fixtures\configs"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pytxo-testenv-" + [guid]::NewGuid().ToString("n"))

$configMap = @{
    echo         = "pytxo.echo.toml"
    context      = "pytxo.echo.toml"
    antigravity  = "pytxo.antigravity.toml"
    claude       = "pytxo.claude.toml"
    codex        = "pytxo.codex.toml"
}

$Pytxo = Resolve-PytxoBin
$env:PYTXO_BIN = $Pytxo

New-Item -ItemType Directory -Path $WorkDir | Out-Null
$Repo = Join-Path $WorkDir "repo"
Copy-Item -Recurse $Fixture $Repo
Set-Location $Repo

git init -q
git config user.email "pytxo@test.local"
git config user.name "Pytxo Test"
git add .
git commit -q -m "init"

$configFile = Join-Path $Configs $configMap[$Profile]
Copy-Item $configFile (Join-Path $Repo "pytxo.toml")

& $Pytxo init
& $Pytxo doctor

$env:PYTXO_TEST_REPO = $Repo
Write-Host ""
Write-Host "=== scaffold complete ==="
Write-Host "Profile:   $Profile"
Write-Host "Repo:      $Repo"
Write-Host "PYTXO_BIN: $Pytxo"
Write-Host ""
Write-Host "Next (same PowerShell session):"
Write-Host "  cd `$env:PYTXO_TEST_REPO"
Write-Host "  & '$Pytxo' run --config pytxo.toml --dry-run"
$recipes = Join-Path $PSScriptRoot "recipes"
switch ($Profile) {
    "echo" { Write-Host "  & '$recipes\smoke-echo.ps1'" }
    "context" { Write-Host "  & '$recipes\smoke-context.ps1'" }
    "antigravity" { Write-Host "  & '$recipes\live-antigravity.ps1'" }
    default { Write-Host "  & '$Pytxo' run --config pytxo.toml --cmd `"echo pytxo-agent`"" }
}
