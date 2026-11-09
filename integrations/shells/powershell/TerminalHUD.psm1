. (Join-Path $PSScriptRoot 'Client.ps1')
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
    $old = @($bound | Where-Object { $_.Key -eq 'Escape' })
    # Public PSReadLine handler metadata does not expose custom ScriptBlocks.
    # Preserve custom behavior by declining installation when it cannot be saved.
    if ($old.Count -gt 1 -or ($old.Count -eq 1 -and $old[0].Function -notin @('RevertLine', 'Abort'))) {
        Write-Warning 'TerminalHUD: custom Escape handler detected; Escape unchanged.'; return
    }
    $script:previous = if ($old.Count) { $old[0].Function } else { $null }
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
    $script:installed = $true
}
function Disable-TerminalHUDEscape {
    if (!$script:installed) { return }
    $current = @(Get-PSReadLineKeyHandler -Bound | Where-Object { $_.Key -eq 'Escape' })
    # Leave a handler installed by somebody else after us intact.
    if ($current.Count -eq 1 -and $current[0].Function -eq 'TerminalHUDPassive') {
        if ($script:previous) { Set-PSReadLineKeyHandler -Chord Escape -Function $script:previous }
        else { Remove-PSReadLineKeyHandler -Chord Escape }
    }
    $script:installed = $false
}
$ExecutionContext.SessionState.Module.OnRemove = { Disable-TerminalHUDEscape }
Export-ModuleMember -Function Enable-TerminalHUDEscape, Disable-TerminalHUDEscape
Enable-TerminalHUDEscape
