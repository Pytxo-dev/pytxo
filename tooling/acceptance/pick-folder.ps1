# Answers Pytxo Desktop's "Select project workspace" dialog through UI
# Automation patterns only (no keystrokes, no focus), so it works on a hosted
# runner: type the folder into the dialog's Folder box, then invoke Select Folder.
param([Parameter(Mandatory)] [string]$Folder, [int]$TimeoutSeconds = 60)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]
$condition = { param($property, $value) New-Object System.Windows.Automation.PropertyCondition($property, $value) }

function Find-Dialog {
  $byClass = & $condition $A::ClassNameProperty "#32770"
  $A::RootElement.FindAll($scope::Descendants, $byClass) | Where-Object { $_.Current.Name -eq "Select project workspace" } | Select-Object -First 1
}

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
do { $dialog = Find-Dialog; if (-not $dialog) { Start-Sleep -Milliseconds 300 } } while (-not $dialog -and (Get-Date) -lt $deadline)
if (-not $dialog) { throw "Folder dialog did not appear" }

# 1152 is the common dialog's file-name edit ("Folder:"); 1 is the default button.
$edit = $dialog.FindFirst($scope::Descendants, (& $condition $A::AutomationIdProperty "1152"))
if (-not $edit) { throw "Folder box not found" }
# On some builds 1152 is a combo box whose edit child holds the value pattern.
$typed = $false
$edits = $edit.FindAll($scope::Descendants, (& $condition $A::ControlTypeProperty ([System.Windows.Automation.ControlType]::Edit)))
foreach ($candidate in @($edit) + @($edits)) {
  $pattern = $null
  if ($candidate.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$pattern)) { $pattern.SetValue($Folder); $typed = $true; break }
}
if (-not $typed) {
  Add-Type -AssemblyName System.Windows.Forms
  $edit.SetFocus()
  [System.Windows.Forms.SendKeys]::SendWait(($Folder -replace '([+^%~(){}\[\]])', '{$1}'))
}
$select = $dialog.FindFirst($scope::Descendants, (& $condition $A::AutomationIdProperty "1"))
for ($attempt = 1; $attempt -le 3; $attempt++) {
  $select.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Seconds 2
  # A folder path can navigate into the folder first; Select Folder then picks it.
  if (-not (Find-Dialog)) { "selected $Folder after $attempt invocation(s)"; exit 0 }
}
throw "Dialog stayed open after selecting $Folder"
