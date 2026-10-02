# Extracts an MSI (administrative image, nothing installed) and starts Pytxo
# Desktop with isolated Pytxo state, trust store and WebView2 profile, plus a
# local CDP port for capture.mjs. Writes launch.json with both hashes.
param(
  [Parameter(Mandatory)] [string]$Msi,
  [Parameter(Mandatory)] [string]$Root,
  [int]$Port = 9340
)
$ErrorActionPreference = "Stop"
if (Test-Path $Root) { throw "Refusing to reuse existing evidence root $Root" }
New-Item -ItemType Directory -Force "$Root\payload", "$Root\home", "$Root\webview" | Out-Null
$msiHash = (Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash
$extract = Start-Process msiexec.exe -ArgumentList @("/a", "`"$Msi`"", "/qn", "TARGETDIR=`"$Root\payload`"", "/L*v", "`"$Root\msi-admin.log`"") -Wait -PassThru
if ($extract.ExitCode -ne 0) { throw "msiexec /a exited $($extract.ExitCode)" }
$exe = Get-ChildItem -Recurse -LiteralPath "$Root\payload" -Filter pytxo-desktop.exe | Select-Object -First 1
$env:PYTXO_HOME = "$Root\home"
$env:PYTXO_TRUST_STORE = "$Root\home\trusted-domains.json"
$env:WEBVIEW2_USER_DATA_FOLDER = "$Root\webview"
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port"
$app = Start-Process -FilePath $exe.FullName -WorkingDirectory "$Root\home" -PassThru
[ordered]@{
  msi = $Msi
  msi_sha256 = $msiHash
  exe = $exe.FullName
  exe_sha256 = (Get-FileHash -LiteralPath $exe.FullName -Algorithm SHA256).Hash
  pid = $app.Id
  started = (Get-Date).ToString("o")
  cdp_port = $Port
} | ConvertTo-Json | Set-Content -Encoding utf8 "$Root\launch.json"
Get-Content "$Root\launch.json"
