function Read-TerminalHUDProfile([string] $Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $ancestor = $full
    while ($ancestor) {
        if (Test-Path -LiteralPath $ancestor) {
            if ((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Profile symlink/reparse point refused: $ancestor" }
        }
        $ancestor = [IO.Path]::GetDirectoryName($ancestor)
    }
    [byte[]] $bytes = @()
    if (Test-Path -LiteralPath $full) { $bytes = [IO.File]::ReadAllBytes($full) }
    $encoding = New-Object Text.UTF8Encoding($false, $true)
    if ($bytes.Length -ge 2 -and $bytes[0] -eq 255 -and $bytes[1] -eq 254) {
        $encoding = New-Object Text.UnicodeEncoding($false, $false, $true)
    } elseif ($bytes.Length -ge 2 -and $bytes[0] -eq 254 -and $bytes[1] -eq 255) {
        $encoding = New-Object Text.UnicodeEncoding($true, $false, $true)
    }
    $text = $encoding.GetString($bytes)
    return [pscustomobject]@{ path = $full; bytes = $bytes; encoding = $encoding; text = $text }
}
function New-TerminalHUDProfilePlan {
    param([string] $Path, [string] $Snippet, [string] $Action)
    $profile = Read-TerminalHUDProfile $Path
    $newline = if ($profile.text.Contains("`r`n")) { "`r`n" } else { "`n" }
    $begin = '# >>> TerminalHUD prompt Escape >>>'
    $end = '# <<< TerminalHUD prompt Escape <<<'
    $block = $newline + $begin + $newline + $Snippet.Replace("`r`n", "`n").TrimEnd("`n").Replace("`n", $newline) + $newline + $end + $newline
    $hasBegin = $profile.text.Contains($begin); $hasEnd = $profile.text.Contains($end)
    if ($hasBegin -or $hasEnd) {
        $index = $profile.text.IndexOf($block, [StringComparison]::Ordinal)
        if ($index -lt 0 -or $profile.text.IndexOf($begin, $profile.text.IndexOf($begin) + $begin.Length) -ge 0 -or
            $profile.text.IndexOf($end, $profile.text.IndexOf($end) + $end.Length) -ge 0) {
            throw 'TerminalHUD marker block was changed or duplicated; preserve it and reconcile manually'
        }
        $next = if ($Action -eq 'Remove') { $profile.text.Remove($index, $block.Length) } else { $profile.text }
    } else { $next = if ($Action -eq 'Install') { $profile.text + $block } else { $profile.text } }
    $changed = $next -cne $profile.text
    $backup = if ($changed -and (Test-Path -LiteralPath $profile.path)) { $profile.path + '.terminalhud-' + [Guid]::NewGuid().ToString('N') + '.bak' } else { $null }
    return [pscustomobject]@{
        action = $Action; profile = $profile.path; changed = $changed; backup = $backup
        snippet = $block; binding_status = 'Checked when the shell loads; unsupported modes/custom Escape handlers are preserved.'
        before = $profile.bytes; after = $profile.encoding.GetBytes($next)
    }
}
function Invoke-TerminalHUDProfilePlan($Plan) {
    $current = Read-TerminalHUDProfile $Plan.profile
    if ([Convert]::ToBase64String($current.bytes) -cne [Convert]::ToBase64String($Plan.before)) { throw 'Profile changed after preview; retry' }
    if (!$Plan.changed) { return }
    if ($Plan.backup) {
        $stream = [IO.File]::Open($Plan.backup, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
        try { $stream.Write($Plan.before, 0, $Plan.before.Length) } finally { $stream.Dispose() }
    }
    $parent = [IO.Path]::GetDirectoryName($Plan.profile)
    $null = [IO.Directory]::CreateDirectory($parent)
    # Recheck the path after backup/directory creation; never follow profile links.
    $latest = Read-TerminalHUDProfile $Plan.profile
    if ([Convert]::ToBase64String($latest.bytes) -cne [Convert]::ToBase64String($Plan.before)) { throw 'Profile changed before write; original backup preserved' }
    $temporary = Join-Path $parent ([Guid]::NewGuid().ToString('N') + '.tmp')
    try {
        [IO.File]::WriteAllBytes($temporary, $Plan.after)
        if (Test-Path -LiteralPath $Plan.profile) { [IO.File]::Replace($temporary, $Plan.profile, [NullString]::Value) }
        else { [IO.File]::Move($temporary, $Plan.profile) }
    } finally { if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary } }
}
