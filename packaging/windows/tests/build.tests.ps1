# Stages real application binaries without bundling a Terminal runtime.
# Does not install, launch the app, register startup, or create a real shortcut.
$ErrorActionPreference = 'Stop'
$temp = Join-Path ([IO.Path]::GetTempPath()) ('terminalhud-package-build-' + [Guid]::NewGuid().ToString('N'))
$output = Join-Path $temp 'package'
try {
    & (Join-Path $PSScriptRoot '..\build.ps1') -Output $output
    foreach ($module in @('Paths', 'Package')) { . (Join-Path $PSScriptRoot "..\$module.ps1") }
    $manifest = Read-Package $output
    if (Test-Path -LiteralPath (Join-Path $output 'runtime')) { throw 'Package unexpectedly contains a Terminal runtime' }
    if ($manifest.source_commit.Length -ne 40 -or !$manifest.version) { throw 'Build identity missing' }
    foreach ($name in @('terminalhud.exe', 'terminalhud-shell.exe')) {
        $bytes = [IO.File]::ReadAllBytes((Join-Path $output $name))
        if ($bytes.Length -lt 2 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { throw "Native Windows binary missing: $name" }
    }
    if (!(Test-Path -LiteralPath (Join-Path $output 'installer\manage.ps1'))) { throw 'Installer missing from staged package' }
    foreach ($asset in @('manage.ps1', 'bash\terminalhud.bash', 'powershell\TerminalHUD.psm1', 'powershell\Request-Passive.ps1', 'powershell\HandlerIdentity.ps1')) {
        if (!(Test-Path -LiteralPath (Join-Path $output "integrations\shells\$asset"))) { throw "Shell integration missing: $asset" }
    }
    if ($env:TERMINALHUD_PACKAGE_ARTIFACT) {
        Assert-SafeTree $env:TERMINALHUD_PACKAGE_ARTIFACT
        if (Test-Path -LiteralPath $env:TERMINALHUD_PACKAGE_ARTIFACT) { throw 'Review artifact destination already exists' }
        Copy-Item -LiteralPath $output -Destination $env:TERMINALHUD_PACKAGE_ARTIFACT -Recurse
    }
    Write-Output 'Native Windows package staged and verified (no bundled runtime or installation)'
} finally { if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force } }
