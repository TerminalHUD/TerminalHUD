# Fake line editor: do not load real PSReadLine, read input, or send real IPC.
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Collections.Generic;
using System.Management.Automation;
[assembly: System.Reflection.AssemblyVersion("2.4.5.0")]
namespace Microsoft.PowerShell {
    static class Keys {
        public static readonly ConsoleKeyInfo Escape = new ConsoleKeyInfo((char)27, ConsoleKey.Escape, false, false, false);
    }
    public class PSConsoleReadLine {
        private class KeyHandler {
            public Action<ConsoleKeyInfo?, object> Action;
            public ScriptBlock ScriptBlock;
        }
        private static readonly PSConsoleReadLine _singleton = new PSConsoleReadLine();
        private Dictionary<ConsoleKeyInfo, KeyHandler> _dispatchTable = new Dictionary<ConsoleKeyInfo, KeyHandler>();
        public static string Line = "";
        public static int Reverts = 0;
        public static int Aborts = 0;
        public static void GetBufferState(ref string line, ref int cursor) { line = Line; cursor = Line.Length; }
        public static void RevertLine(ConsoleKeyInfo? key, object arg) { Reverts++; }
        public static void Abort(ConsoleKeyInfo? key, object arg) { Aborts++; }
        public static void SetBuiltin(string name) {
            Action<ConsoleKeyInfo?, object> action;
            if (name == "RevertLine") action = RevertLine; else action = Abort;
            _singleton._dispatchTable[Keys.Escape] = new KeyHandler { Action = action, ScriptBlock = null };
        }
        public static void SetCustom(ScriptBlock block) {
            _singleton._dispatchTable[Keys.Escape] = new KeyHandler { Action = (key, arg) => {}, ScriptBlock = block };
        }
        public static void RemoveEscape() { _singleton._dispatchTable.Remove(Keys.Escape); }
    }
}
'@
$global:mode = 'Windows'
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'RevertLine' })
$global:handler = $null
$global:requests = 0
[Microsoft.PowerShell.PSConsoleReadLine]::SetBuiltin('RevertLine')
function global:Get-Module { param($Name) if ($Name -eq 'PSReadLine') { return [pscustomobject]@{ Version = [version]'2.4.5' } }; Microsoft.PowerShell.Core\Get-Module @PSBoundParameters }
function global:Get-PSReadLineOption { [pscustomobject]@{ EditMode = $global:mode } }
function global:Get-PSReadLineKeyHandler { param([switch] $Bound) $global:bound }
function global:Set-PSReadLineKeyHandler {
    param($Chord, $BriefDescription, [scriptblock] $ScriptBlock, $Function)
    if ($ScriptBlock) { [Microsoft.PowerShell.PSConsoleReadLine]::SetCustom($ScriptBlock) }
    else { [Microsoft.PowerShell.PSConsoleReadLine]::SetBuiltin($Function) }
    $global:handler = $ScriptBlock
    $name = if ($ScriptBlock) { $BriefDescription } else { $Function }
    $global:bound = @([pscustomobject]@{ Key = $Chord; Function = $name })
}
function global:Remove-PSReadLineKeyHandler { param($Chord) [Microsoft.PowerShell.PSConsoleReadLine]::RemoveEscape(); $global:bound = @(); $global:handler = $null }
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
    [Microsoft.PowerShell.PSConsoleReadLine]::SetCustom({})
    $module = Import-Module $path -PassThru
    Assert ($global:bound[0].Function -eq $binding.Function) 'Custom handler/prefix changed'
    Remove-Module $module
}
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'Abort' })
[Microsoft.PowerShell.PSConsoleReadLine]::SetBuiltin('Abort')
$module = Import-Module $path -PassThru
& $module { function script:Invoke-TerminalHUDPassive { $global:requests++ } }
[Microsoft.PowerShell.PSConsoleReadLine]::Line = 'text'
& $global:handler $null $null
Assert ([Microsoft.PowerShell.PSConsoleReadLine]::Aborts -eq 1) 'Nonempty Abort was not preserved'
# A later user binding must survive module removal.
$global:bound = @([pscustomobject]@{ Key = 'Escape'; Function = 'LaterUserHandler' })
[Microsoft.PowerShell.PSConsoleReadLine]::SetCustom({})
Remove-Module $module
Assert ($global:bound[0].Function -eq 'LaterUserHandler') 'Unload overwrote later user handler'
Write-Output 'Fake PSReadLine empty/nonempty/multiline/mode/custom-handler/unload tests passed'

$hostName = if ($PSVersionTable.PSEdition -eq 'Core') { 'pwsh.exe' } else { 'powershell.exe' }
foreach ($version in @('2.4.5', '3.0.0')) {
    & (Join-Path $PSHOME $hostName) -NoLogo -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'unsupported_identity.tests.ps1') -PSReadLineVersion $version
    if ($LASTEXITCODE -ne 0) { throw "Unsupported identity regression failed: $version" }
}
