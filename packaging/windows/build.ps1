[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $Output,
    [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')][string] $Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Paths.ps1')
. (Join-Path $PSScriptRoot 'Licenses.ps1')
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$destination = [IO.Path]::GetFullPath($Output)
Assert-SafeTree $destination
if (Test-Path -LiteralPath $destination) { throw 'Output already exists; choose a new staging directory' }
Push-Location $repository
try {
    $hostInfo = & rustc -vV
    if ($LASTEXITCODE -ne 0 -or !($hostInfo -match '^host: .*pc-windows-msvc$')) { throw 'Use a native Windows MSVC Rust host' }
    $dirty = & git status --porcelain --untracked-files=no
    if ($LASTEXITCODE -ne 0 -or $dirty) { throw 'Commit tracked source changes before creating an identified package' }
    $commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[a-f0-9]{40}$') { throw 'Cannot determine source commit' }
    & cargo build --release --locked --target $Target -p terminalhud-app --bins
    if ($LASTEXITCODE -ne 0) { throw 'Application build failed' }
    $metadataText = & cargo metadata --locked --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed' }
    $metadata = $metadataText | ConvertFrom-Json
    $app = @($metadata.packages | Where-Object { $_.name -ceq 'terminalhud-app' })[0]
    $buildId = $app.version.Replace('.', '_') + '-' + $commit.Substring(0, 16)
    Assert-BuildId $buildId
    $null = New-Item -ItemType Directory -Path $destination
    foreach ($name in @('terminalhud.exe', 'terminalhud-shell.exe')) {
        Copy-Item -LiteralPath (Join-Path $metadata.target_directory "$Target\release\$name") -Destination (Join-Path $destination $name)
    }
    Copy-Item -LiteralPath (Join-Path $repository 'LICENSE') -Destination (Join-Path $destination 'LICENSE')
    Copy-DependencyLicenses $repository $destination $Target
    $installer = Join-Path $destination 'installer'
    $null = New-Item -ItemType Directory -Path $installer
    foreach ($name in @('manage', 'Paths', 'Shortcut', 'Receipt', 'Package', 'Install', 'Startup', 'Uninstall', 'OperationLock')) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot "$name.ps1") -Destination (Join-Path $installer "$name.ps1")
    }
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'README.md') -Destination (Join-Path $installer 'README.md')
    $shellSource = Join-Path $repository 'integrations\shells'
    $shellDestination = Join-Path $destination 'integrations\shells'
    $null = New-Item -ItemType Directory -Path $shellDestination -Force
    foreach ($name in @('manage.ps1', 'Profile.ps1', 'Snippet.ps1', 'README.md', 'bash', 'powershell')) {
        Copy-Item -LiteralPath (Join-Path $shellSource $name) -Destination (Join-Path $shellDestination $name) -Recurse
    }
    $identity = [pscustomobject]@{ version = $app.version; source_commit = $commit; build_id = $buildId; target = $Target }
    [IO.File]::WriteAllText((Join-Path $destination 'BUILD-INFO.json'), ($identity | ConvertTo-Json), (New-Object Text.UTF8Encoding($false)))
    $files = @(Get-ChildItem -LiteralPath $destination -Recurse -File | Sort-Object FullName | ForEach-Object {
        [pscustomobject]@{ path = $_.FullName.Substring($destination.TrimEnd('\').Length + 1).Replace('\', '/'); sha256 = Get-FileDigest $_.FullName }
    })
    $manifest = [pscustomobject]@{ schema_version = 1; product = 'TerminalHUD'; version = $app.version; source_commit = $commit; build_id = $buildId; target = $Target; files = $files }
    [IO.File]::WriteAllText((Join-Path $destination 'package.json'), ($manifest | ConvertTo-Json -Depth 6), (New-Object Text.UTF8Encoding($false)))
    Write-Output "Staged package: $destination (not installed)"
} finally { Pop-Location }
