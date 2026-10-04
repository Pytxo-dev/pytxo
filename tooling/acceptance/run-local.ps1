# Local native journey on a developer machine: the given Desktop build runs the
# mixed-agent journey with stand-in agent CLIs, isolated from the machine's own
# Pytxo data (PYTXO_HOME) and WebView profile. Real agent CLIs are never called.
# The journey drives the real pointer and keyboard, so leave the machine alone
# while it runs. -Film records 60 fps without the pointer for the demo edit.
param(
  [Parameter(Mandatory)] [string]$Exe,
  [Parameter(Mandatory)] [string]$Evidence,
  [string]$Python = "python",
  [switch]$Film
)
$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "../..")
$Exe = (Resolve-Path $Exe).Path
if (Get-Process pytxo-desktop -ErrorAction SilentlyContinue) { throw "Close Pytxo Desktop first; the journey finds its window by title." }
if (Test-Path $Evidence) { Remove-Item -Recurse -Force $Evidence }
New-Item -ItemType Directory -Force $Evidence | Out-Null
$Evidence = (Resolve-Path $Evidence).Path
$work = Join-Path $Evidence "sandbox"
$bin = Join-Path $work "bin"
New-Item -ItemType Directory -Force $bin, "$work\home", "$work\webview" | Out-Null

Copy-Item (Join-Path $PSScriptRoot "stand-in-agent.mjs") $bin
Copy-Item (Join-Path $PSScriptRoot "replay-20261002.json") (Join-Path $bin "replay.json")
foreach ($cli in "codex", "claude", "cursor-agent", "opencode", "agy") {
  Set-Content -Encoding ascii (Join-Path $bin "$cli.cmd") "@node `"%~dp0stand-in-agent.mjs`" $cli %*"
}

$fixture = Join-Path $work "fleet\taskboard"
& (Join-Path $root "docs\demo\fleet\setup.ps1") -Path $fixture
$baselineTests = (& npm --prefix $fixture test 2>&1 | Out-String)

$saved = @{ PATH = $env:PATH; PYTXO_HOME = $env:PYTXO_HOME; WEBVIEW2_USER_DATA_FOLDER = $env:WEBVIEW2_USER_DATA_FOLDER }
$env:PATH = "$bin;$env:PATH"
$env:PYTXO_HOME = "$work\home"
$env:WEBVIEW2_USER_DATA_FOLDER = "$work\webview"
$app = Start-Process -FilePath $Exe -WorkingDirectory "$work\home" -PassThru
try {
  $journey = @((Join-Path $PSScriptRoot "journey.py"), "--out", (Join-Path $Evidence "journey"), "--mode", "full", "--repo", $fixture,
    "--mission", (Join-Path $root "docs\demo\fleet\mission.txt"), "--team", "Claude Code,Cursor Agent,OpenCode,Antigravity")
  if ($Film) { $journey += "--film" }
  & $Python @journey
  $journeyExit = $LASTEXITCODE
} finally {
  if (-not $app.HasExited) { Stop-Process -Id $app.Id -Force; $app.WaitForExit(15000) | Out-Null }
  foreach ($name in $saved.Keys) { Set-Item "env:$name" $saved[$name] }
}

node (Join-Path $PSScriptRoot "result-app.mjs") --out (Join-Path $Evidence "journey") --repo $fixture
$changed = @(git -C $fixture status --porcelain | ForEach-Object { $_.Substring(3).Trim('"') }) | Sort-Object
$receipt = Get-Content -Raw (Join-Path $Evidence "journey\receipt.json") | ConvertFrom-Json
$tests = & npm --prefix $fixture test 2>&1 | Out-String
$recorded = @((Get-Content (Join-Path $PSScriptRoot "replay-20261002.json") -Raw | ConvertFrom-Json).tasks.PSObject.Properties.Value | ForEach-Object { $_.files.PSObject.Properties })
$summary = [ordered]@{
  exe = $Exe
  exe_sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash
  journey = $receipt.result
  journey_exit = $journeyExit
  changed_files = $changed
  reviewed_files = @($receipt.checks.review.files) | Sort-Object
  tests_after_apply = [bool]($LASTEXITCODE -eq 0)
  replayed_files = "{0}/{1}" -f @($recorded | Where-Object { (Get-Content -Raw -LiteralPath (Join-Path $fixture $_.Name) -ErrorAction SilentlyContinue) -ceq $_.Value }).Count, $recorded.Count
}
Set-Content -Encoding utf8 (Join-Path $Evidence "fixture-tests.txt") "--- before ---`n$baselineTests`n--- after Apply ---`n$tests"
git -C $fixture diff > (Join-Path $Evidence "fixture-applied.diff")
$summary | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $Evidence "summary.json")
$summary | ConvertTo-Json -Depth 6
if ($receipt.result -ne "passed") { exit 1 }
