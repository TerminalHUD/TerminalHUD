function Read-Package([string] $Package) {
    Assert-SafeTree $Package
    $manifestPath = Join-Path $Package 'package.json'
    Assert-SafeTree $manifestPath
    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($manifest.schema_version -ne 1 -or $manifest.product -cne 'TerminalHUD') { throw 'Invalid package manifest' }
    Assert-BuildId $manifest.build_id
    if (!$manifest.version -or $manifest.source_commit -cnotmatch '^[a-f0-9]{40}$') { throw 'Missing version/build identity' }
    $seen = @{}
    foreach ($file in $manifest.files) {
        $path = Resolve-PayloadPath $Package $file.path
        if ($seen.ContainsKey($file.path.ToLowerInvariant())) { throw 'Duplicate package path' }
        $seen[$file.path.ToLowerInvariant()] = $true
        if ($file.sha256 -cnotmatch '^[a-f0-9]{64}$' -or (Get-FileDigest $path) -cne $file.sha256) { throw "Package digest mismatch: $path" }
    }
    foreach ($required in @('terminalhud.exe', 'terminalhud-shell.exe', 'BUILD-INFO.json', 'LICENSE', 'THIRD-PARTY-NOTICES.txt', 'runtime/runtime.json')) {
        if (!$seen.ContainsKey($required.ToLowerInvariant())) { throw "Missing package file: $required" }
    }
    if (!@($manifest.files | Where-Object { $_.path.StartsWith('runtime/LICENSES/', [StringComparison]::Ordinal) }).Count) { throw 'Runtime license texts missing' }
    return $manifest
}
