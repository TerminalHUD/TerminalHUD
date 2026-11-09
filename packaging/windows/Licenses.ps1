function Copy-DependencyLicenses([string] $Repository, [string] $Destination, [string] $Target) {
    $metadataText = & cargo metadata --locked --format-version 1 --filter-platform $Target
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed' }
    $metadata = $metadataText | ConvertFrom-Json
    $packages = @{}; $nodes = @{}
    foreach ($package in $metadata.packages) { $packages[$package.id] = $package }
    foreach ($node in $metadata.resolve.nodes) { $nodes[$node.id] = $node }
    $app = @($metadata.packages | Where-Object { $_.name -ceq 'terminalhud-app' })[0]
    $queue = New-Object 'Collections.Generic.Queue[string]'
    $queue.Enqueue($app.id); $seen = @{}; $notices = @('TerminalHUD third-party dependency licenses', '')
    while ($queue.Count) {
        $id = $queue.Dequeue()
        if ($seen.ContainsKey($id)) { continue }
        $seen[$id] = $true
        $package = $packages[$id]
        foreach ($dependency in $nodes[$id].deps) {
            if (@($dependency.dep_kinds | Where-Object { $_.kind -ne 'dev' }).Count) { $queue.Enqueue($dependency.pkg) }
        }
        if ($null -eq $package.source) { continue }
        $directory = Split-Path -Parent $package.manifest_path
        $licenseFiles = @(Get-ChildItem -LiteralPath $directory -File | Where-Object { $_.Name -match '^(?i:LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT)(?:[._-].*)?$' })
        if ($package.license_file) {
            $specified = Join-Path $directory $package.license_file
            if (Test-Path -LiteralPath $specified -PathType Leaf) { $licenseFiles += Get-Item -LiteralPath $specified }
        }
        $licenseFiles = @($licenseFiles | Sort-Object FullName -Unique)
        if (!$licenseFiles.Count) { throw "License text missing for $($package.name) $($package.version); package cannot be staged" }
        $output = Join-Path $Destination "LICENSES\$($package.name)-$($package.version)"
        $null = New-Item -ItemType Directory -Path $output -Force
        foreach ($file in $licenseFiles) { Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $output $file.Name) }
        $notices += "$($package.name) $($package.version): $($package.license) (LICENSES/$($package.name)-$($package.version)/)"
    }
    [IO.File]::WriteAllLines((Join-Path $Destination 'THIRD-PARTY-NOTICES.txt'), $notices, (New-Object Text.UTF8Encoding($false)))
}
