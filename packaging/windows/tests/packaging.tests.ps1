# No live registry, user installation, GUI, terminal, or shortcut access.
$ErrorActionPreference = 'Stop'
foreach ($module in @('Paths', 'Shortcut', 'Receipt', 'Package', 'Install', 'Startup', 'Uninstall', 'OperationLock')) { . (Join-Path $PSScriptRoot "..\$module.ps1") }
function Assert($Condition, [string] $Message) { if (!$Condition) { throw $Message } }
function Reject([scriptblock] $Action) { $failed = $false; try { & $Action | Out-Null } catch { $failed = $true }; Assert $failed 'Expected rejection' }
$temp = Join-Path ([IO.Path]::GetTempPath()) ('terminalhud-package-test-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $temp
try {
    $script:fixtureShortcut = Join-Path $temp 'Programs\TerminalHUD.lnk'
    function Get-ShortcutPath { return $script:fixtureShortcut }
    $create = { param($path, $target) [IO.File]::WriteAllText($path, $target) }
    $package = Join-Path $temp 'package'; $root = Join-Path $temp 'user\install'
    $null = New-Item -ItemType Directory -Path $package
    $paths = @('terminalhud.exe', 'terminalhud-shell.exe', 'BUILD-INFO.json', 'LICENSE', 'THIRD-PARTY-NOTICES.txt')
    foreach ($path in $paths) { [IO.File]::WriteAllText((Join-Path $package $path), "fixture:$path") }
    $files = @($paths | ForEach-Object { [pscustomobject]@{ path = $_; sha256 = Get-FileDigest (Join-Path $package $_) } })
    $manifest = [pscustomobject]@{ schema_version = 1; product = 'TerminalHUD'; version = '0.1.0'; source_commit = ('a' * 40); build_id = 'fixture1'; files = $files }
    $manifestPath = Join-Path $package 'package.json'
    function Write-Manifest { [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 6)) }
    Write-Manifest
    $null = New-InstallPlan $package $root
    Assert (!(Test-Path -LiteralPath (Split-Path -Parent $root))) 'Dry run created directories'
    foreach ($unsafe in @('../x', 'C:/x', '/x', 'a\x', 'a/../x', 'a/file:stream', 'a/NUL.txt', 'a/ends.')) { Reject { Resolve-PayloadPath $root $unsafe } }
    $settings = Join-Path (Split-Path -Parent $root) 'settings.json'
    $null = New-Item -ItemType Directory -Path (Split-Path -Parent $root)
    [IO.File]::WriteAllText($settings, '{ malformed user settings preserved }')
    $settingsBefore = Get-FileDigest $settings
    $null = Invoke-OperationLock $root { Invoke-Install $package $root $create }
    Reject { Invoke-Install $package $root $create }
    Assert ((Get-FileDigest $settings) -ceq $settingsBefore) 'Settings changed'
    $receiptBefore = [IO.File]::ReadAllBytes((Join-Path $root 'receipt.json'))
    $oversized = Read-Receipt $root
    $oversized.startup_commands = @('x' * 1MB)
    Reject { Save-Receipt $root $oversized }
    Assert ([Convert]::ToBase64String($receiptBefore) -ceq [Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $root 'receipt.json')))) 'Oversized receipt replaced existing ownership'
    $oldCli = Join-Path $root 'versions\fixture1\terminalhud.exe'
    $manifest.build_id = 'fixture2'; Write-Manifest
    $shortcutBytes = [IO.File]::ReadAllBytes($script:fixtureShortcut)
    [IO.File]::WriteAllText($script:fixtureShortcut, 'modified shortcut')
    Reject { New-InstallPlan $package $root }
    [IO.File]::WriteAllBytes($script:fixtureShortcut, $shortcutBytes)
    Reject { Invoke-OperationLock $root { Invoke-OperationLock $root { throw 'Must never enter second operation' } } }
    $null = Invoke-OperationLock $root { Invoke-Install $package $root $create }
    Assert (Test-Path -LiteralPath $oldCli) 'Upgrade removed old bridge'
    Assert ([IO.File]::ReadAllText($script:fixtureShortcut).Contains('fixture2')) 'Shortcut did not upgrade'
    $script:fakeStartup = [pscustomobject]@{ kind = 'String'; value = (Read-Receipt $root).startup_commands[0] }
    $read = { $script:fakeStartup }
    $remove = { param($expected) Assert ($expected.value -ceq $script:fakeStartup.value) 'Expected startup mismatch'; $script:fakeStartup = $null }
    $null = New-UninstallPlan $root $read
    Assert ($null -ne $script:fakeStartup) 'Planning mutated registry'
    $null = Invoke-Uninstall $root $read $remove
    Assert ($null -eq $script:fakeStartup) 'Owned startup was not removed'
    Assert (!(Test-Path -LiteralPath $script:fixtureShortcut)) 'Owned shortcut was not removed'
    Assert (Test-Path -LiteralPath $oldCli) 'Uninstall removed bridge used by live shell'
    $script:fakeStartup = [pscustomobject]@{ kind = 'ExpandString'; value = 'foreign' }
    Reject { New-UninstallPlan $root $read }
    $script:fakeStartup = $null
    [IO.File]::WriteAllText($oldCli, 'modified-user-file')
    $unowned = Join-Path $root 'versions\fixture1\user.txt'; [IO.File]::WriteAllText($unowned, 'user')
    $lockedRelative = 'versions/fixture2/LICENSE'
    $locked = [IO.File]::Open((Join-Path $root $lockedRelative), [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
    try { $result = Invoke-Cleanup $root } finally { $locked.Dispose() }
    Assert ($result.pending -contains $lockedRelative) 'In-use file was not reported pending'
    Assert ($result.pending -contains 'versions/fixture1/terminalhud.exe') 'Modified owned file was removed'
    Assert (Test-Path -LiteralPath $unowned) 'Unowned file removed'
    Assert ((Get-FileDigest $settings) -ceq $settingsBefore) 'Uninstall changed settings'
    $junction = Join-Path $temp 'junction'
    $null = New-Item -ItemType Junction -Path $junction -Target $package
    Reject { Resolve-PayloadPath $temp 'junction/terminalhud.exe' }
    [IO.Directory]::Delete($junction)
    $manifest.build_id = 'fixture3'; Write-Manifest
    Reject { Invoke-Install $package $root { param($path, $target) throw 'Injected shortcut failure' } }
    Assert (Test-Path -LiteralPath (Join-Path $root 'versions\fixture3\terminalhud.exe')) 'Partial installation ownership lost'
    $null = Read-Receipt $root
    [IO.File]::WriteAllText($script:fixtureShortcut, 'unowned shortcut')
    Reject { New-InstallPlan $package (Join-Path $temp 'fresh-root') }
    Assert ([IO.File]::ReadAllText($script:fixtureShortcut) -ceq 'unowned shortcut') 'Unowned shortcut changed'
    Remove-Item -LiteralPath $script:fixtureShortcut
    $manifest.files[0].sha256 = '0' * 64; Write-Manifest
    Reject { New-InstallPlan $package (Join-Path $temp 'new-root') }
    $unownedRoot = Join-Path $temp 'unowned-root'; $null = New-Item -ItemType Directory -Path $unownedRoot
    Reject { Read-Receipt $unownedRoot }
    Write-Output 'Packaging filesystem and fake-registry tests passed'
} finally { Remove-Item -LiteralPath $temp -Recurse -Force }
