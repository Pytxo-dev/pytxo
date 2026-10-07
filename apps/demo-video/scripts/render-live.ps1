# Renders PytxoLive in short chunks (long single renders degrade on a busy
# machine), renders the mixed audio once, then joins and muxes with ffmpeg.
param([string]$Out = "out/pytxo-live.mp4", [int]$Chunk = 600, [int]$Concurrency = 2, [switch]$Fresh)
# Remotion writes progress and warnings to stderr; check outputs instead.
$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")
$total = [int]((npx remotion compositions src/index.ts 2>$null | Select-String "PytxoLive\s+\d+\s+\S+\s+(\d+)").Matches[0].Groups[1].Value)
$dir = "out/live-chunks"
if ($Fresh -and (Test-Path $dir)) { Remove-Item -Recurse -Force $dir }
New-Item -ItemType Directory -Force $dir | Out-Null
$list = @()
for ($from = 0; $from -lt $total; $from += $Chunk) {
  $to = [Math]::Min($from + $Chunk, $total) - 1
  $file = "$dir/part-$from.mp4"
  # Finished chunks are kept, so a run interrupted by low memory resumes.
  if (-not (Test-Path $file)) { npx remotion render src/index.ts PytxoLive "$dir/tmp-$from.mp4" --frames="$from-$to" --codec=h264 --crf=17 --pixel-format=yuv420p --color-space=bt709 --muted --concurrency=$Concurrency --overwrite *> "$dir/part-$from.log"; if ($LASTEXITCODE -eq 0) { Move-Item -Force "$dir/tmp-$from.mp4" $file } }
  if (-not (Test-Path $file)) { throw "Chunk $from-$to failed; see $dir/part-$from.log" }
  $list += "file 'part-$from.mp4'"
  Write-Host "frames $from-$to of $total"
}
Set-Content -Encoding ascii "$dir/list.txt" $list
npx remotion render src/index.ts PytxoLive "$dir/audio.wav" --codec=wav --concurrency=$Concurrency --overwrite *> "$dir/audio.log"
ffmpeg -v error -y -f concat -safe 0 -i "$dir/list.txt" -i "$dir/audio.wav" -map 0:v -map 1:a -c:v copy -af "loudnorm=I=-16:TP=-1.5:LRA=11:linear=true" -c:a aac -b:a 256k -ar 48000 -shortest -movflags +faststart $Out
Write-Host "wrote $Out"
