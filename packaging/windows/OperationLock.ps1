function Invoke-OperationLock([string] $Root, [scriptblock] $Action) {
    Assert-SafeTree $Root
    $parent = Split-Path -Parent $Root
    $null = New-Item -ItemType Directory -Path $parent -Force
    $lockPath = $Root + '.operation.lock'
    Assert-SafeTree $lockPath
    # CreateNew preserves unowned/conflicting files. Never remove another operation's lock.
    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { & $Action }
    finally { $lock.Dispose(); Remove-Item -LiteralPath $lockPath }
}
