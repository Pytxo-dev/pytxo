# Mission-loop dogfood scaffold
# Creates tooling/benchmarks/mission-loop/{solo,manual,pytxo}/ for five missions.

$Root = Join-Path $PSScriptRoot "mission-loop"
$Missions = @(
  "auth-cross-cutting",
  "db-migration-api",
  "plugin-tests-docs",
  "large-refactor",
  "three-unrelated-bugs"
)
$Modes = @("solo", "manual", "pytxo")

foreach ($mode in $Modes) {
  foreach ($m in $Missions) {
    $dir = Join-Path $Root (Join-Path $mode $m)
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $readme = Join-Path $dir "NOTES.md"
    if (-not (Test-Path $readme)) {
      @"
# $mode / $m

- setup_minutes:
- wall_clock:
- cost:
- interventions:
- integration_failures:
- tests_passed:
- review_minutes:
- success:
- notes:
"@ | Set-Content -Path $readme -Encoding utf8
    }
  }
}

Write-Host "Scaffold ready under $Root"
Write-Host "Fill NOTES.md after each dogfood run. Keep plan/status JSON beside NOTES when using pytxo."
