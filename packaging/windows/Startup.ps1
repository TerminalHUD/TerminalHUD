# Registry access is confined to explicit uninstall application, never planning/install.
function Read-Startup {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run', $false)
    if ($null -eq $key) { return $null }
    try {
        if ($key.GetValueNames() -notcontains 'TerminalHUD') { return $null }
        return [pscustomobject]@{ kind = $key.GetValueKind('TerminalHUD').ToString(); value = $key.GetValue('TerminalHUD', $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) }
    } finally { $key.Dispose() }
}
function Assert-OwnedStartup($Current, $Receipt) {
    if ($null -ne $Current -and ($Current.kind -cne 'String' -or $Receipt.startup_commands -cnotcontains $Current.value)) { throw 'Unowned HKCU Run/TerminalHUD value preserved' }
}
function Remove-OwnedStartup($Expected) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run', $true)
    if ($null -eq $key) {
        if ($null -ne $Expected) { throw 'Startup entry changed since preparation' }
        return
    }
    try {
        $actual = $null
        if ($key.GetValueNames() -contains 'TerminalHUD') { $actual = [pscustomobject]@{ kind = $key.GetValueKind('TerminalHUD').ToString(); value = $key.GetValue('TerminalHUD', $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) } }
        if (($null -eq $Expected) -ne ($null -eq $actual) -or ($null -ne $Expected -and ($Expected.kind -cne $actual.kind -or $Expected.value -cne $actual.value))) { throw 'Startup entry changed since preparation; preserved' }
        if ($null -ne $actual) {
            $key.DeleteValue('TerminalHUD', $true)
            if ($key.GetValueNames() -contains 'TerminalHUD') { throw 'Startup deletion completed but value changed during verification; inspect a new plan' }
        }
    } finally { $key.Dispose() }
}
