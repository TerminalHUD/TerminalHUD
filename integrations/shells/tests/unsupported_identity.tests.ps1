# Isolated fake type with an unknown private layout. No real line editor/input.
param([Parameter(Mandatory)][version] $PSReadLineVersion)
$ErrorActionPreference = 'Stop'
$global:fixtureVersion = $global:fixtureVersion
Add-Type 'namespace Microsoft.PowerShell { public class PSConsoleReadLine { public static void FixtureOnly() {} } }'
function global:Get-Module {
    param($Name)
    if ($Name -eq 'PSReadLine') { return [pscustomobject]@{ Version = $global:fixtureVersion } }
    Microsoft.PowerShell.Core\Get-Module @PSBoundParameters
}
function global:Get-PSReadLineOption { [pscustomobject]@{ EditMode = 'Windows' } }
function global:Get-PSReadLineKeyHandler { param([switch] $Bound) [pscustomobject]@{ Key = 'Escape'; Function = 'RevertLine' } }
function global:Set-PSReadLineKeyHandler { throw 'Unsupported identity must never change bindings' }
function global:Remove-PSReadLineKeyHandler { throw 'Unsupported identity must never remove bindings' }
$module = Import-Module (Join-Path $PSScriptRoot '..\powershell\TerminalHUD.psm1') -PassThru
if ((& $module { Get-TerminalHUDEscapeIdentity }).supported) { throw 'Unsupported identity was accepted' }
Remove-Module $module
Write-Output "Unsupported PSReadLine version/layout preserves bindings: $PSReadLineVersion"
