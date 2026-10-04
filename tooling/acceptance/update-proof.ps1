# In-app update proof on a disposable Windows runner: the public v1.2.1 app,
# unmodified, finds and installs the candidate through its own updater. The
# runner's hosts file sends the feed hosts to a local HTTPS stand-in (trusted
# through a throwaway root certificate) that serves the candidate MSI and its
# real updater signature, so v1.2.1's embedded public key checks the production
# signature. The feed URL itself and Windows UAC consent are not exercised:
# Desktop runs as the runner's administrator so msiexec needs no prompt.
param(
  [Parameter(Mandatory)] [string]$Msi,
  [Parameter(Mandatory)] [string]$Evidence,
  [string]$PreviousUrl = "https://github.com/Pytxo-dev/pytxo-releases/releases/download/v1.2.1/pytxo-desktop-windows-x64.msi"
)
$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force $Evidence | Out-Null
$Evidence = (Resolve-Path $Evidence).Path
$Msi = (Resolve-Path $Msi).Path
if (-not (Test-Path "$Msi.sig")) { throw "The candidate needs its updater signature next to it ($Msi.sig)" }
$summary = [ordered]@{ previous = $PreviousUrl; candidate = (Split-Path $Msi -Leaf); candidate_sha256 = (Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash }

function Get-Installed {
  Get-ChildItem HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall, HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall -ErrorAction SilentlyContinue |
    Get-ItemProperty | Where-Object { $_.DisplayName -like "Pytxo Desktop*" } | ForEach-Object { "$($_.DisplayName) $($_.DisplayVersion)" }
}
function Find-Exe { Get-ChildItem -Recurse -File -Filter pytxo-desktop.exe "C:\Program Files" -ErrorAction SilentlyContinue | Select-Object -First 1 }
$hostsFile = "$env:SystemRoot\System32\drivers\etc\hosts"
$hostsBefore = Get-Content -Raw $hostsFile

# 1. The previous public release, installed as users have it.
$previous = Join-Path $env:RUNNER_TEMP "pytxo-previous.msi"
Invoke-WebRequest -Uri $PreviousUrl -OutFile $previous -UseBasicParsing
$run = Start-Process msiexec.exe -ArgumentList @("/i", "`"$previous`"", "/qn", "/norestart", "/l*v", "`"$Evidence\install-previous.log`"") -Wait -PassThru
if ($run.ExitCode -ne 0) { throw "msiexec /i v1.2.1 exited $($run.ExitCode)" }
$summary.before = @(Get-Installed)
# The candidate's version comes from its MSI, as the feed must advertise it.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $installer, @($Msi, 0))
$view = $database.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $database, @("SELECT Value FROM Property WHERE Property='ProductVersion'"))
$view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null) | Out-Null
$record = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)
$candidateVersion = $record.GetType().InvokeMember("StringData", "GetProperty", $null, $record, 1)
$summary.offered_version = $candidateVersion

# 2. The local feed: a throwaway certificate for the feed hosts, trusted machine-wide.
$cert = New-SelfSignedCertificate -DnsName "github.com", "raw.githubusercontent.com" -CertStoreLocation Cert:\LocalMachine\My -KeyExportPolicy Exportable -NotAfter (Get-Date).AddDays(1)
$passphrase = [guid]::NewGuid().ToString("N")
$pfx = Join-Path $env:RUNNER_TEMP "update-feed.pfx"
Export-PfxCertificate -Cert $cert -FilePath $pfx -Password (ConvertTo-SecureString $passphrase -AsPlainText -Force) | Out-Null
Export-Certificate -Cert $cert -FilePath "$env:RUNNER_TEMP\update-feed.cer" | Out-Null
Import-Certificate -FilePath "$env:RUNNER_TEMP\update-feed.cer" -CertStoreLocation Cert:\LocalMachine\Root | Out-Null
$feedLog = Join-Path $Evidence "feed.log"
$feed = Start-Process node -ArgumentList @((Join-Path $PSScriptRoot "update-feed.mjs"), "--pfx", $pfx, "--passphrase", $passphrase, "--msi", "`"$Msi`"", "--version", $candidateVersion, "--log", "`"$feedLog`"") -PassThru -WindowStyle Hidden
Add-Content -Path $hostsFile -Value "`r`n127.0.0.1 github.com`r`n127.0.0.1 raw.githubusercontent.com"
ipconfig /flushdns | Out-Null
Start-Sleep -Seconds 2

try {
  # 3. Open v1.2.1 and take the update it offers.
  $exe = Find-Exe
  $summary.before_file_version = $exe.VersionInfo.FileVersion
  Start-Process -FilePath $exe.FullName | Out-Null
  python (Join-Path $PSScriptRoot "journey.py") --out $Evidence --mode update --name previous | Out-Host

  # 4. The installed version, and whether the updated app came back by itself.
  $deadline = (Get-Date).AddMinutes(5)
  do { Start-Sleep -Seconds 5; $installed = @(Get-Installed) } while (($installed -join "|") -notlike "*$candidateVersion*" -and (Get-Date) -lt $deadline)
  $summary.after = $installed
  $exe = Find-Exe
  $summary.after_file_version = $exe.VersionInfo.FileVersion
  $deadline = (Get-Date).AddSeconds(90)
  do { Start-Sleep -Seconds 3; $app = Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1 } while (-not $app -and (Get-Date) -lt $deadline)
  $summary.relaunched = [bool]$app
  if ($app) {
    $summary.relaunched_file_version = (Get-Item $app.Path).VersionInfo.FileVersion
    python (Join-Path $PSScriptRoot "journey.py") --out $Evidence --mode probe --name updated | Out-Host
    $summary.updated_shows_onboarding = (Get-Content (Join-Path $Evidence "updated.json") -Raw | ConvertFrom-Json).onboarding_shown
  }
} finally {
  Get-Process pytxo-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
  Stop-Process -Id $feed.Id -Force -ErrorAction SilentlyContinue
  Set-Content -Path $hostsFile -Value $hostsBefore -NoNewline
  ipconfig /flushdns | Out-Null
  Get-ChildItem Cert:\LocalMachine\Root, Cert:\LocalMachine\My | Where-Object { $_.Thumbprint -eq $cert.Thumbprint } | Remove-Item
}
$summary.feed_requests = @(Get-Content $feedLog -ErrorAction SilentlyContinue)
$summary | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $Evidence "update.json")
Get-Content (Join-Path $Evidence "update.json")
if (($summary.after -join "|") -notlike "*$candidateVersion*" -or $summary.after.Count -ne 1) { throw "The in-app update did not leave exactly the candidate installed" }
if (-not ($summary.feed_requests -match "pytxo-update-proof")) { throw "The app never downloaded the offered installer" }
