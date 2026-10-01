$ErrorActionPreference = 'Stop'
# Deliberately not parameterized: no caller-supplied path can become a reset target.
$fixtureRoot = 'C:\pytxo-disposable-demo-2026-09-14-taskboard'
if (Test-Path -LiteralPath $fixtureRoot) { throw "Already exists; preserve it: $fixtureRoot" }
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$template = Join-Path $PSScriptRoot 'fixture'
Get-ChildItem -LiteralPath $template -Force | Copy-Item -Destination $fixtureRoot -Recurse
git -C $fixtureRoot init -b demo/baseline
if ($LASTEXITCODE) { throw 'Fixture Git init failed' }
git -C $fixtureRoot add -- .gitignore .pytxo-demo-sentinel package.json server.mjs pytxo.toml src test
if ($LASTEXITCODE) { throw 'Fixture staging failed' }
git -C $fixtureRoot -c user.name='Pytxo Demo' -c user.email='demo@localhost' commit -m 'fixture: initial offline task board'
if ($LASTEXITCODE) { throw 'Fixture baseline commit failed' }
git -C $fixtureRoot tag demo-initial
Write-Output "Prepared disposable fixture: $fixtureRoot"
