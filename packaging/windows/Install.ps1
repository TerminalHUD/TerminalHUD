function New-InstallPlan([string] $Package, [string] $Root) {
    $manifest = Read-Package $Package
    $receipt = Read-Receipt $Root
    $null = Assert-OwnedShortcut $receipt
    $versionPath = Join-Path $Root "versions\$($manifest.build_id)"
    Assert-SafeTree $versionPath
    if ($null -ne $receipt -and @($receipt.versions | Where-Object { $_.build_id -ieq $manifest.build_id }).Count) { throw "Build identity already receipted; use a fresh build: $($manifest.build_id)" }
    if (Test-Path -LiteralPath $versionPath) { throw "Version directory already exists; preserved: $versionPath" }
    $files = @($manifest.files | ForEach-Object {
        [pscustomobject]@{ path = "versions/$($manifest.build_id)/$($_.path)"; sha256 = $_.sha256 }
    })
    $version = [pscustomobject]@{ build_id = $manifest.build_id; files = $files }
    $versions = @(); $commands = @(); $shortcuts = @()
    if ($null -ne $receipt) { $versions = @($receipt.versions); $commands = @($receipt.startup_commands); $shortcuts = @($receipt.shortcuts) }
    $command = '"' + (Join-Path $versionPath 'terminalhud-shell.exe') + '"'
    $plan = [pscustomobject]@{
        operation = 'install'; root = $Root; package = $Package; build_id = $manifest.build_id
        startup_command = $command; shortcut = Get-ShortcutPath; files = $files
        receipt = [pscustomobject]@{ schema_version = 1; product = 'TerminalHUD'; installed = $true; active_build = $manifest.build_id; shortcuts = @($shortcuts); versions = @($versions) + @($version); startup_commands = @($commands) + @($command) }
    }
    $null = ConvertTo-ReceiptJson $plan.receipt
    return $plan
}
function Invoke-Install([string] $Package, [string] $Root, [scriptblock] $CreateShortcut) {
    # Plan again at application time: stale plans are never accepted as authority.
    $plan = New-InstallPlan $Package $Root
    # Receipt precedes file writes so an interrupted copy has recoverable ownership.
    $previous = Read-Receipt $Root
    $null = New-Item -ItemType Directory -Path $Root -Force
    $nextActive = $plan.receipt.active_build
    if ($null -ne $previous) {
        $plan.receipt.active_build = $previous.active_build
        $plan.receipt.installed = $previous.installed
    } else { $plan.receipt.installed = $false }
    Save-Receipt $Root $plan.receipt
    try {
        foreach ($file in $plan.files) {
            $relative = $file.path.Substring(("versions/$($plan.build_id)/").Length)
            $source = Resolve-PayloadPath $Package $relative
            $target = Resolve-PayloadPath $Root $file.path
            $null = New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force
            [IO.File]::Copy($source, $target, $false)
            if ((Get-FileDigest $target) -cne $file.sha256) { throw "Copied digest mismatch: $target" }
        }
    } catch { throw "Installation incomplete; receipt retained for cleanup/recovery. $($_.Exception.Message)" }
    $plan.receipt.active_build = $nextActive
    $plan.receipt.installed = $true
    Save-Receipt $Root $plan.receipt
    try { Install-Shortcut $Root $plan.receipt $CreateShortcut }
    catch { throw "Payload installed but shortcut activation failed; ownership receipt retained. Plan uninstall/recovery before retrying. $($_.Exception.Message)" }
    return $plan
}
