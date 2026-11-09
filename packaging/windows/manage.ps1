[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('install', 'uninstall', 'cleanup')][string] $Operation,
    [string] $Package,
    [switch] $Apply,
    [switch] $SessionsClosed
)
$ErrorActionPreference = 'Stop'
foreach ($module in @('Paths', 'Shortcut', 'Receipt', 'Package', 'Install', 'Startup', 'Uninstall', 'OperationLock')) { . (Join-Path $PSScriptRoot "$module.ps1") }
$root = Get-InstallRoot
$action = {
    switch ($Operation) {
        'install' {
            if (!$Package) { throw '-Package is required for install' }
            $resolved = [IO.Path]::GetFullPath($Package)
            if ($Apply) { Invoke-Install $resolved $root { param($path, $target) New-Shortcut $path $target } } else { New-InstallPlan $resolved $root }
        }
        'uninstall' {
            if ($Apply) { Invoke-Uninstall $root { Read-Startup } { param($expected) Remove-OwnedStartup $expected } }
            else { New-UninstallPlan $root { Read-Startup } }
        }
        'cleanup' {
            if ($Apply -and !$SessionsClosed) { throw 'Cleanup application requires -SessionsClosed after closing all terminal sessions' }
            if ($Apply) { Invoke-Cleanup $root } else { New-CleanupPlan $root }
        }
    }
}
try {
    $result = if ($Apply) { Invoke-OperationLock $root $action } else { & $action }
    $result | ConvertTo-Json -Depth 10
} catch { Write-Error $_; exit 2 }
