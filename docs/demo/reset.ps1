param([switch]$ConfirmDisposableReset)
$ErrorActionPreference = 'Stop'
$fixtureRoot = 'C:\pytxo-disposable-demo-2026-09-14-taskboard'
$resolved = (Resolve-Path -LiteralPath $fixtureRoot).Path
if ($resolved -cne $fixtureRoot) { throw 'Unexpected fixture root' }
if ((Get-Item -LiteralPath $resolved).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Refusing a linked fixture' }
$sentinel = Join-Path $resolved '.pytxo-demo-sentinel'
if ((Get-Content -LiteralPath $sentinel -Raw).Trim() -cne 'pytxo-disposable-taskboard-2026-09-14') { throw 'Wrong fixture sentinel' }
$gitRoot = (git -C $resolved rev-parse --show-toplevel).Replace('/', '\').Trim()
if ($LASTEXITCODE -or $gitRoot -cne $fixtureRoot) { throw 'Wrong Git root' }
$allowed = @('src/model.mjs', 'src/app.js', 'src/index.html', 'src/style.css', 'test/model.test.mjs')
foreach ($relative in $allowed) {
  $path = Join-Path $resolved $relative
  foreach ($entry in @($path, (Split-Path $path))) {
    if (Test-Path -LiteralPath $entry) {
      if ((Get-Item -LiteralPath $entry).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Refusing linked path: $entry" }
    }
  }
}
if (!$ConfirmDisposableReset) { Write-Output 'Validated target. Stop its agents/server, retain evidence, then rerun with -ConfirmDisposableReset.'; return }
# Recoverable tracked-file restore only; never delete the repository, ledger or untracked files.
$backup = Join-Path (Split-Path $resolved) ('pytxo-demo-reset-backup-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $backup | Out-Null
foreach ($relative in $allowed) {
  $source = Join-Path $resolved $relative
  if (Test-Path -LiteralPath $source) {
    $destination = Join-Path $backup $relative
    New-Item -ItemType Directory -Path (Split-Path $destination) -Force | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination
  }
}
git -C $resolved restore --source=demo-initial --worktree -- $allowed
if ($LASTEXITCODE) { throw "Restore failed; previous files preserved at $backup" }
Write-Output "Restored five fixture source/test files. Backup: $backup. Ledger and untracked files preserved; use a fresh run."
