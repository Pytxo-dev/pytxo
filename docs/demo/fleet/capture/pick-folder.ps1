# Answers Pytxo Desktop's own "Select project workspace" dialog: types the
# folder into the address bar (Alt+D), then confirms the dialog (IDOK).
# Window text, not screen coordinates, selects the dialog.
param([Parameter(Mandatory)] [string]$Folder, [int]$TimeoutSeconds = 20)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type 'using System; using System.Runtime.InteropServices; public static class FolderDialog { [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, int m, IntPtr w, IntPtr l); }'
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]
$byClass = New-Object System.Windows.Automation.PropertyCondition($A::ClassNameProperty, "#32770")
$byId = { param($id) New-Object System.Windows.Automation.PropertyCondition($A::AutomationIdProperty, $id) }

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
do {
  $dialog = $A::RootElement.FindAll($scope::Descendants, $byClass) | Where-Object { $_.Current.Name -eq "Select project workspace" } | Select-Object -First 1
  if (-not $dialog) { Start-Sleep -Milliseconds 300 }
} while (-not $dialog -and (Get-Date) -lt $deadline)
if (-not $dialog) { throw "Folder dialog did not appear" }

$shell = New-Object -ComObject WScript.Shell
if (-not $shell.AppActivate("Select project workspace")) { throw "Could not focus the folder dialog" }
Start-Sleep -Milliseconds 400
$shell.SendKeys("%d")
Start-Sleep -Milliseconds 400
$shell.SendKeys("$Folder{ENTER}")
Start-Sleep -Seconds 2
$address = $dialog.FindFirst($scope::Descendants, (& $byId "1001")).Current.Name
if ($address -notlike "*$Folder*") { throw "Dialog did not navigate to $Folder ($address)" }
$select = $dialog.FindFirst($scope::Descendants, (& $byId "1"))
# WM_COMMAND IDOK from the Select Folder button.
[void][FolderDialog]::SendMessage([IntPtr]$dialog.Current.NativeWindowHandle, 0x0111, [IntPtr]1, [IntPtr]$select.Current.NativeWindowHandle)
"selected $Folder"
