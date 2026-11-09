function Get-ShortcutPath {
    $programs = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
    if (!$programs -or ![IO.Path]::IsPathRooted($programs)) { throw 'Cannot resolve per-user Start menu Programs' }
    return Join-Path $programs 'TerminalHUD.lnk'
}
function New-Shortcut([string] $Path, [string] $Target) {
    $shortcut = $null
    $shell = New-Object -ComObject WScript.Shell
    try {
        $shortcut = $shell.CreateShortcut($Path)
        $shortcut.TargetPath = $Target
        $shortcut.Arguments = ''
        $shortcut.WorkingDirectory = Split-Path -Parent $Target
        $shortcut.Description = 'TerminalHUD'
        $shortcut.Save()
    } finally {
        if ($null -ne $shortcut) { $null = [Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut) }
        if ($null -ne $shell) { $null = [Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell) } }
}
function Assert-OwnedShortcut($Receipt) {
    $path = Get-ShortcutPath
    Assert-SafeTree $path
    if (!(Test-Path -LiteralPath $path)) { return $null }
    $digest = Get-FileDigest $path
    if ($null -eq $Receipt -or !@($Receipt.shortcuts | Where-Object { $_.path -ceq $path -and $_.sha256 -ceq $digest }).Count) { throw "Unowned or modified shortcut preserved: $path" }
    return $digest
}
function Install-Shortcut([string] $Root, $Receipt, [scriptblock] $CreateShortcut) {
    $path = Get-ShortcutPath
    $expected = Assert-OwnedShortcut (Read-Receipt $Root)
    $null = New-Item -ItemType Directory -Path (Split-Path -Parent $path) -Force
    $temporary = Join-Path (Split-Path -Parent $path) ([Guid]::NewGuid().ToString('N') + '.lnk')
    $command = '"' + (Join-Path $Root "versions\$($Receipt.active_build)\terminalhud-shell.exe") + '"'
    try {
        & $CreateShortcut $temporary $command.Substring(1, $command.Length - 2)
        $record = [pscustomobject]@{ path = $path; sha256 = Get-FileDigest $temporary; target = $command }
        if ((Assert-OwnedShortcut (Read-Receipt $Root)) -cne $expected) { throw 'Shortcut changed during installation; preserved' }
        # Record both previous and new hashes before atomic activation for crash recovery.
        $Receipt.shortcuts = @($Receipt.shortcuts) + @($record)
        Save-Receipt $Root $Receipt
        if ($null -ne $expected) { [IO.File]::Replace($temporary, $path, $null) }
        else { [IO.File]::Move($temporary, $path) }
        $Receipt.shortcuts = @($record)
        Save-Receipt $Root $Receipt
    } finally { if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary } }
}
function Remove-Shortcut($Receipt) {
    $path = Get-ShortcutPath
    $expected = Assert-OwnedShortcut $Receipt
    if ($null -eq $expected) { return }
    if ((Get-FileDigest $path) -cne $expected) { throw 'Shortcut changed during uninstall; preserved' }
    Remove-Item -LiteralPath $path -ErrorAction Stop
}
