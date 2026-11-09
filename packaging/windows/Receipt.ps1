function Read-Receipt([string] $Root) {
    Assert-SafeTree $Root
    $path = Join-Path $Root 'receipt.json'
    if (!(Test-Path -LiteralPath $path)) {
        if (Test-Path -LiteralPath $Root) { throw "Unowned installation directory: $Root" }
        return $null
    }
    if ((Get-Item -LiteralPath $path).Length -gt 1MB) { throw 'Installation receipt too large' }
    Assert-SafeTree $path
    $receipt = Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($receipt.schema_version -ne 1 -or $receipt.product -cne 'TerminalHUD' -or $receipt.installed -isnot [bool]) { throw 'Invalid installation receipt' }
    Assert-BuildId $receipt.active_build
    $seen = @{}
    $builds = @{}
    foreach ($version in $receipt.versions) {
        Assert-BuildId $version.build_id
        if ($builds.ContainsKey($version.build_id.ToLowerInvariant())) { throw 'Duplicate receipted build' }
        $builds[$version.build_id.ToLowerInvariant()] = $true
        foreach ($file in $version.files) {
            $null = Resolve-PayloadPath $Root $file.path
            if (!$file.path.StartsWith("versions/$($version.build_id)/", [StringComparison]::Ordinal) -or $file.sha256 -cnotmatch '^[a-f0-9]{64}$' -or $seen.ContainsKey($file.path.ToLowerInvariant())) { throw 'Invalid file ownership receipt' }
            $seen[$file.path.ToLowerInvariant()] = $true
        }
    }
    if (!$builds.ContainsKey($receipt.active_build.ToLowerInvariant())) { throw 'Active build missing from receipt' }
    $commands = @($receipt.versions | ForEach-Object { '"' + (Join-Path $Root "versions\$($_.build_id)\terminalhud-shell.exe") + '"' })
    foreach ($command in $receipt.startup_commands) {
        if ($commands -cnotcontains $command) { throw 'Invalid startup ownership receipt' }
    }
    foreach ($shortcut in $receipt.shortcuts) {
        if ($shortcut.path -cne (Get-ShortcutPath) -or $shortcut.sha256 -cnotmatch '^[a-f0-9]{64}$' -or $receipt.startup_commands -cnotcontains $shortcut.target) { throw 'Invalid shortcut ownership receipt' }
    }
    return $receipt
}
function ConvertTo-ReceiptJson($Receipt) {
    $json = $Receipt | ConvertTo-Json -Depth 10
    if ([Text.Encoding]::UTF8.GetByteCount($json) -gt 1MB) { throw 'Installation receipt would exceed 1 MiB; existing ownership preserved' }
    return $json
}
function Save-Receipt([string] $Root, $Receipt) {
    Assert-SafeTree $Root
    $path = Join-Path $Root 'receipt.json'
    Assert-SafeTree $path
    $temporary = Join-Path $Root ([Guid]::NewGuid().ToString('N') + '.tmp')
    $json = ConvertTo-ReceiptJson $Receipt
    try {
        [IO.File]::WriteAllText($temporary, $json, (New-Object Text.UTF8Encoding($false)))
        if (Test-Path -LiteralPath $path) { [IO.File]::Replace($temporary, $path, [NullString]::Value) }
        else { [IO.File]::Move($temporary, $path) }
    } finally { if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary } }
}
