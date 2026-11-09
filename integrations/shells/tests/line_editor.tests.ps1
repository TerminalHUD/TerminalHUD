# Fake line editor: do not load real PSReadLine, read input, or send real IPC.
$ErrorActionPreference = 'Stop'
Add-Type @'
namespace Microsoft.PowerShell {
    public static class PSConsoleReadLine {
        public static string Line = "";
        public static int Reverts = 0;
        public static int Aborts = 0;
        public static void GetBufferState(ref string line, ref int cursor) { line = Line; cursor = Line.Length; }
        public static void RevertLine(object key, object arg) { Reverts++; }
        public static void Abort(object key, object arg) { Aborts++; }
    }
}
'@
$global:mode = 'Windows'
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'RevertLine' })
$global:handler = $null
$global:requests = 0
function global:Get-Module { param($Name) if ($Name -eq 'PSReadLine') { return $true }; Microsoft.PowerShell.Core\Get-Module @PSBoundParameters }
function global:Get-PSReadLineOption { [pscustomobject]@{ EditMode = $global:mode } }
function global:Get-PSReadLineKeyHandler { param([switch] $Bound) $global:bound }
function global:Set-PSReadLineKeyHandler {
    param($Chord, $BriefDescription, [scriptblock] $ScriptBlock, $Function)
    $global:handler = $ScriptBlock
    $name = if ($ScriptBlock) { $BriefDescription } else { $Function }
    $global:bound = @([pscustomobject]@{ Key = $Chord; Function = $name })
}
function global:Remove-PSReadLineKeyHandler { param($Chord) $global:bound = @(); $global:handler = $null }
function Assert($Condition, [string] $Message) { if (!$Condition) { throw $Message } }
$path = Join-Path $PSScriptRoot '..\powershell\TerminalHUD.psm1'
$module = Import-Module $path -PassThru
& $module { function script:Invoke-TerminalHUDPassive { $global:requests++ } }
& $global:handler $null $null
Assert ($global:requests -eq 1) 'Empty prompt did not request Passive'
[Microsoft.PowerShell.PSConsoleReadLine]::Line = 'unfinished input'
& $global:handler $null $null
Assert ($global:requests -eq 1 -and [Microsoft.PowerShell.PSConsoleReadLine]::Reverts -eq 1) 'Nonempty line did not retain RevertLine'
[Microsoft.PowerShell.PSConsoleReadLine]::Line = "first`n"
& $global:handler $null $null
Assert ($global:requests -eq 1) 'Multiline buffer requested Passive'
Remove-Module $module
Assert ($global:bound[0].Function -eq 'RevertLine') 'Unload did not restore old Escape handler'
foreach ($mode in @('Emacs', 'Vi')) {
    $global:mode = $mode
    $module = Import-Module $path -PassThru
    Assert ($global:bound[0].Function -eq 'RevertLine') 'Unsupported mode changed bindings'
    Remove-Module $module
}
$global:mode = 'Windows'
foreach ($binding in @(
    [pscustomobject]@{ Key = 'Escape'; Function = 'CustomUserEscape' },
    [pscustomobject]@{ Key = 'Escape,b'; Function = 'BackwardWord' }
)) {
    $global:bound = @($binding)
    $module = Import-Module $path -PassThru
    Assert ($global:bound[0].Function -eq $binding.Function) 'Custom handler/prefix changed'
    Remove-Module $module
}
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'Abort' })
$module = Import-Module $path -PassThru
& $module { function script:Invoke-TerminalHUDPassive { $global:requests++ } }
[Microsoft.PowerShell.PSConsoleReadLine]::Line = 'text'
& $global:handler $null $null
Assert ([Microsoft.PowerShell.PSConsoleReadLine]::Aborts -eq 1) 'Nonempty Abort was not preserved'
# A later user binding must survive module removal.
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'LaterUserHandler' })
Remove-Module $module
Assert ($global:bound[0].Function -eq 'LaterUserHandler') 'Unload overwrote later user handler'
Write-Output 'Fake PSReadLine empty/nonempty/multiline/mode/custom-handler/unload tests passed'
