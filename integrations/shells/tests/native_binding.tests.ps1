# Native PSReadLine metadata only; no ReadLine, history, IPC, or desktop input.
$ErrorActionPreference = 'Stop'
Import-Module PSReadLine
Set-PSReadLineOption -HistorySaveStyle SaveNothing -EditMode Windows
. (Join-Path $PSScriptRoot '..\powershell\HandlerIdentity.ps1')
$path = Join-Path $PSScriptRoot '..\powershell\TerminalHUD.psm1'
Write-Output ('Native PSReadLine version: ' + (Get-Module PSReadLine).Version)
$probe = Get-TerminalHUDEscapeIdentity
if (!$probe.supported) { throw 'Handler identity precondition failed' }
$before = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
$module = Import-Module $path -PassThru
$current = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
if ($current.Count -ne 1 -or $current[0].Function -ne 'TerminalHUDPassive') { throw 'Native Windows-mode binding did not install' }
Remove-Module $module
$after = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
if ($after.Count -ne $before.Count -or $after[0].Function -ne $before[0].Function) { throw 'Native binding did not restore' }
# Display labels are user controlled. Colliding names must remain custom.
foreach ($label in @('RevertLine', 'Abort')) {
    Set-PSReadLineKeyHandler -Chord Escape -BriefDescription $label -ScriptBlock { }
    $beforeIdentity = Get-TerminalHUDEscapeIdentity
    if (!$beforeIdentity.supported -or $null -eq $beforeIdentity.handler -or $beforeIdentity.builtin) { throw 'Custom identity precondition failed' }
    $module = Import-Module $path -PassThru
    $afterIdentity = Get-TerminalHUDEscapeIdentity
    if (![object]::ReferenceEquals($beforeIdentity.handler, $afterIdentity.handler)) { throw "Custom handler labeled $label was overwritten" }
    Remove-Module $module
    if (![object]::ReferenceEquals($beforeIdentity.handler, (Get-TerminalHUDEscapeIdentity).handler)) { throw 'Unload changed declined custom handler' }
}
# Action.Method/Target describe the last multicast invocation only. Preserve
# composite actions even when their final invocation is a known built-in.
$type = [Microsoft.PowerShell.PSConsoleReadLine]
$delegateType = [Action[Nullable[ConsoleKeyInfo],object]]
foreach ($last in @('RevertLine', 'Abort')) {
    $first = if ($last -eq 'RevertLine') { 'Abort' } else { 'RevertLine' }
    $prefix = $type.GetMethod($first, [type[]]@([Nullable[ConsoleKeyInfo]], [object])).CreateDelegate($delegateType)
    $suffix = $type.GetMethod($last, [type[]]@([Nullable[ConsoleKeyInfo]], [object])).CreateDelegate($delegateType)
    $composite = [Delegate]::Combine($prefix, $suffix)
    [Microsoft.PowerShell.PSConsoleReadLine]::SetKeyHandler([string[]]@('Escape'), $composite, $last, 'Composite fixture; never invoked')
    $beforeIdentity = Get-TerminalHUDEscapeIdentity
    if (!$beforeIdentity.supported -or $null -eq $beforeIdentity.handler -or $beforeIdentity.builtin) { throw 'Multicast action was mistaken for a built-in' }
    $rawAction = $beforeIdentity.handler.GetType().GetField('Action').GetValue($beforeIdentity.handler)
    if ($rawAction.GetInvocationList().Length -ne 2 -or $rawAction.Method.Name -cne $last) { throw 'Multicast fixture precondition failed' }
    $module = Import-Module $path -PassThru
    if (![object]::ReferenceEquals($beforeIdentity.handler, (Get-TerminalHUDEscapeIdentity).handler)) { throw 'Multicast handler was overwritten' }
    Remove-Module $module
    if (![object]::ReferenceEquals($beforeIdentity.handler, (Get-TerminalHUDEscapeIdentity).handler)) { throw 'Unload changed declined multicast handler' }
}
Set-PSReadLineKeyHandler -Chord Escape -Function RevertLine
$module = Import-Module $path -PassThru
Set-PSReadLineKeyHandler -Chord Escape -BriefDescription TerminalHUDPassive -ScriptBlock { }
$replacement = Get-TerminalHUDEscapeIdentity
Remove-Module $module
if (![object]::ReferenceEquals($replacement.handler, (Get-TerminalHUDEscapeIdentity).handler)) { throw 'Unload overwrote later user handler with colliding owned label' }
foreach ($mode in @('Emacs', 'Vi')) {
    Set-PSReadLineOption -EditMode $mode
    $before = Get-PSReadLineKeyHandler -Bound | ConvertTo-Json
    $module = Import-Module $path -PassThru
    $after = Get-PSReadLineKeyHandler -Bound | ConvertTo-Json
    if ($after -cne $before) { throw "Native $mode bindings changed" }
    Remove-Module $module
}
Write-Output 'Native PSReadLine binding, restoration, colliding-label/multicast identity and Emacs/Vi preservation passed'
