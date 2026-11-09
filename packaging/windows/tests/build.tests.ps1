# Stages real application binaries with an explicitly synthetic runtime payload.
# Does not install, launch the app, register startup, or create a real shortcut.
$ErrorActionPreference = 'Stop'
$temp = Join-Path ([IO.Path]::GetTempPath()) ('terminalhud-package-build-' + [Guid]::NewGuid().ToString('N'))
$runtime = Join-Path $temp 'runtime-input'
$output = Join-Path $temp 'package'
try {
    $null = New-Item -ItemType Directory -Path (Join-Path $runtime 'LICENSES') -Force
    [IO.File]::WriteAllText((Join-Path $runtime 'runtime.json'), '{"kind":"synthetic-test-fixture-not-a-launchable-runtime"}')
    [IO.File]::WriteAllText((Join-Path $runtime 'LICENSES\LICENSE'), 'Synthetic fixture created for testing, no external runtime included.')
    & (Join-Path $PSScriptRoot '..\build.ps1') -RuntimePayload $runtime -Output $output
    foreach ($module in @('Paths', 'Package')) { . (Join-Path $PSScriptRoot "..\$module.ps1") }
    $manifest = Read-Package $output
    if ($manifest.source_commit.Length -ne 40 -or !$manifest.version) { throw 'Build identity missing' }
    foreach ($name in @('terminalhud.exe', 'terminalhud-shell.exe')) {
        $bytes = [IO.File]::ReadAllBytes((Join-Path $output $name))
        if ($bytes.Length -lt 2 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { throw "Native Windows binary missing: $name" }
    }
    if (!(Test-Path -LiteralPath (Join-Path $output 'installer\manage.ps1'))) { throw 'Installer missing from staged package' }
    Write-Output 'Native Windows package staged and verified (synthetic runtime, no installation)'
} finally { if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force } }
