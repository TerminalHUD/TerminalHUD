$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$sources = @()
foreach ($root in @((Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\Fragments'), (Join-Path $env:ProgramData 'Microsoft\Windows Terminal\Fragments'))) {
    if (Test-Path -LiteralPath $root) {
        $sources += @(Get-ChildItem -LiteralPath $root -Directory | ForEach-Object { $_.Name })
    }
}
foreach ($package in Get-AppxPackage) {
    $manifest = Get-AppxPackageManifest -Package $package.PackageFullName -ErrorAction Stop
    $extensions = $manifest.SelectNodes('//*[local-name()="AppExtension" and @Name="com.microsoft.windows.terminal.settings"]')
    if ($extensions.Count -gt 0) { $sources += $package.PackageFamilyName }
}
ConvertTo-Json -InputObject @($sources | Sort-Object -Unique) -Compress
