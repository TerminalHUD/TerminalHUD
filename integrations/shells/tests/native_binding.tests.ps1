# Native PSReadLine metadata only; no ReadLine, history, IPC, or desktop input.
$ErrorActionPreference = 'Stop'
Import-Module PSReadLine
Set-PSReadLineOption -HistorySaveStyle SaveNothing -EditMode Windows
$path = Join-Path $PSScriptRoot '..\powershell\TerminalHUD.psm1'
$before = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
$module = Import-Module $path -PassThru
$current = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
if ($current.Count -ne 1 -or $current[0].Function -ne 'TerminalHUDPassive') { throw 'Native Windows-mode binding did not install' }
Remove-Module $module
$after = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
if ($after.Count -ne $before.Count -or $after[0].Function -ne $before[0].Function) { throw 'Native binding did not restore' }
foreach ($mode in @('Emacs', 'Vi')) {
    Set-PSReadLineOption -EditMode $mode
    $before = Get-PSReadLineKeyHandler -Bound | ConvertTo-Json
    $module = Import-Module $path -PassThru
    $after = Get-PSReadLineKeyHandler -Bound | ConvertTo-Json
    if ($after -cne $before) { throw "Native $mode bindings changed" }
    Remove-Module $module
}
Write-Output 'Native PSReadLine binding, restoration, and Emacs/Vi preservation passed'
