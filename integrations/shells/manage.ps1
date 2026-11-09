[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('Preview', 'Install', 'Remove')][string] $Operation,
    [ValidateSet('Install', 'Remove')][string] $PreviewAction = 'Install',
    [Parameter(Mandatory)][ValidateSet('PowerShell', 'WSLBash', 'GitBash')][string] $Shell,
    [Parameter(Mandatory)][string] $ProfilePath,
    [string] $InstallRoot,
    [string] $BashInstallRoot
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Snippet.ps1')
. (Join-Path $PSScriptRoot 'Profile.ps1')
if (!$InstallRoot) {
    $local = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
    if (!$local) { throw 'Current-user Local App Data is unavailable' }
    $InstallRoot = Join-Path $local 'TerminalHUD\install'
}
if ($InstallRoot.Contains("`n") -or $InstallRoot.Contains("`r")) { throw 'Invalid installation path' }
$snippet = New-TerminalHUDSnippet $Shell $InstallRoot $BashInstallRoot
$action = if ($Operation -eq 'Preview') { $PreviewAction } else { $Operation }
$plan = New-TerminalHUDProfilePlan $ProfilePath $snippet $action
$plan | Select-Object action, profile, changed, backup, binding_status, snippet | Format-List
if ($Operation -ne 'Preview') { Invoke-TerminalHUDProfilePlan $plan }
