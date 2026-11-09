function Test-ProfileCleanupRequired([string] $Root) {
    $local = Split-Path -Parent $Root
    foreach ($name in @('terminal-profile-receipt.json', 'terminal-profile-pending.json', 'terminal-profile-receipt.lock')) {
        if (Test-Path -LiteralPath (Join-Path $local $name)) { return $true }
    }
    return $false
}
function New-UninstallPlan([string] $Root, [scriptblock] $ReadStartup) {
    $receipt = Read-Receipt $Root
    if ($null -eq $receipt) { throw 'No installation receipt' }
    $shortcutHash = Assert-OwnedShortcut $receipt
    $startup = & $ReadStartup
    Assert-OwnedStartup $startup $receipt
    return [pscustomobject]@{
        operation = 'uninstall'; root = $Root; startup = $startup; pending_cleanup = $true; shortcut = Get-ShortcutPath; shortcut_sha256 = $shortcutHash
        profile_cleanup_required = (Test-ProfileCleanupRequired $Root)
        preserved_versions = @($receipt.versions | ForEach-Object { $_.build_id })
        reason = 'Preserved for live terminal sessions. Cleanup requires explicit confirmation that sessions are closed.'
    }
}
function Invoke-Uninstall([string] $Root, [scriptblock] $ReadStartup, [scriptblock] $RemoveStartup) {
    $plan = New-UninstallPlan $Root $ReadStartup
    if ($plan.profile_cleanup_required) { throw 'Review and remove the owned Terminal profile before uninstall: terminalhud profiles remove-plan, then profiles remove. Preserve any pending journal and backup.' }
    try {
        & $RemoveStartup $plan.startup
        $receipt = Read-Receipt $Root
        Remove-Shortcut $receipt
        $receipt.shortcuts = @()
        $receipt.installed = $false
        Save-Receipt $Root $receipt
    } catch { throw "Uninstall may be partial: startup or shortcut removal may have completed. Re-read the plan before retrying. $($_.Exception.Message)" }
    return $plan
}
function New-CleanupPlan([string] $Root) {
    if (Test-ProfileCleanupRequired $Root) { throw 'Owned Terminal profile or pending transaction remains; preserve installed CLI and review profile removal first' }
    $receipt = Read-Receipt $Root
    if ($null -eq $receipt -or $receipt.installed) { throw 'Uninstall before cleanup' }
    $remove = @(); $preserve = @()
    foreach ($version in $receipt.versions) {
        foreach ($file in $version.files) {
            $path = Resolve-PayloadPath $Root $file.path
            if (!(Test-Path -LiteralPath $path)) { continue }
            try {
                if ((Get-FileDigest $path) -ceq $file.sha256) { $remove += $file.path }
                else { $preserve += $file.path }
            } catch { $preserve += $file.path }
        }
    }
    return [pscustomobject]@{ operation = 'cleanup'; root = $Root; remove = $remove; preserve = $preserve }
}
function Invoke-Cleanup([string] $Root) {
    $plan = New-CleanupPlan $Root
    $pending = @($plan.preserve)
    foreach ($relative in $plan.remove) {
        try {
            $path = Resolve-PayloadPath $Root $relative
            $receipt = Read-Receipt $Root
            $entry = @($receipt.versions | ForEach-Object { $_.files } | Where-Object { $_.path -ceq $relative })
            if ($entry.Count -ne 1 -or (Get-FileDigest $path) -cne $entry[0].sha256) { throw 'File changed since cleanup planning' }
            Remove-Item -LiteralPath $path -ErrorAction Stop
        } catch { $pending += $relative; Write-Warning "Pending cleanup: $relative ($($_.Exception.Message))" }
    }
    # Retain ownership receipt and directories, including unowned files. Never recurse-delete.
    return [pscustomobject]@{ operation = 'cleanup'; root = $Root; pending = $pending; receipt_retained = $true }
}
