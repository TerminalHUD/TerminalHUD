. (Join-Path $PSScriptRoot 'Client.ps1')
. (Join-Path $PSScriptRoot 'HandlerIdentity.ps1')
$script:installed = $false
$script:previous = $null

function Enable-TerminalHUDEscape {
    if ($script:installed) { return }
    if (!(Get-Module PSReadLine)) { Write-Warning 'TerminalHUD: load PSReadLine first; Escape unchanged.'; return }
    $mode = (Get-PSReadLineOption).EditMode
    if ($mode -ne 'Windows') {
        Write-Warning 'TerminalHUD: Escape requires Windows editing mode; Emacs/Vi bindings unchanged.'; return
    }
    $bound = @(Get-PSReadLineKeyHandler -Bound)
    if (@($bound | Where-Object { $_.Key -like 'Escape,*' }).Count) {
        Write-Warning 'TerminalHUD: Escape prefix chords detected; bindings unchanged.'; return
    }
    $identity = Get-TerminalHUDEscapeIdentity
    if (!$identity.supported) {
        Write-Warning 'TerminalHUD: PSReadLine handler identity unavailable; bindings unchanged.'; return
    }
    if ($null -ne $identity.handler -and !$identity.builtin) {
        Write-Warning 'TerminalHUD: custom Escape handler detected; Escape unchanged.'; return
    }
    $script:previous = $identity.builtin
    Set-PSReadLineKeyHandler -Chord Escape -BriefDescription TerminalHUDPassive -ScriptBlock {
        param($key, $arg)
        $line = $null; $cursor = 0
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        if ($line.Length -eq 0 -and (Get-PSReadLineOption).EditMode -eq 'Windows') {
            Invoke-TerminalHUDPassive
        } elseif ($script:previous -eq 'RevertLine') {
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine($key, $arg)
        } elseif ($script:previous -eq 'Abort') {
            [Microsoft.PowerShell.PSConsoleReadLine]::Abort($key, $arg)
        }
    }
    $script:ownedHandler = (Get-TerminalHUDEscapeIdentity).handler
    $script:installed = $true
}
function Disable-TerminalHUDEscape {
    if (!$script:installed) { return }
    $current = Get-TerminalHUDEscapeIdentity
    # Leave a handler installed by somebody else after us intact.
    if ($current.supported -and [object]::ReferenceEquals($current.handler, $script:ownedHandler)) {
        if ($script:previous) { Set-PSReadLineKeyHandler -Chord Escape -Function $script:previous }
        else { Remove-PSReadLineKeyHandler -Chord Escape }
    }
    $script:installed = $false
}
$ExecutionContext.SessionState.Module.OnRemove = { Disable-TerminalHUDEscape }
Export-ModuleMember -Function Enable-TerminalHUDEscape, Disable-TerminalHUDEscape
Enable-TerminalHUDEscape
