# Temporary files only. No actual profiles, installation, registry or desktop.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\Profile.ps1')
. (Join-Path $PSScriptRoot '..\Snippet.ps1')
. (Join-Path $PSScriptRoot '..\powershell\Client.ps1')
function Assert($Condition, [string] $Message) { if (!$Condition) { throw $Message } }
function Reject([scriptblock] $Action) { $failed = $false; try { & $Action | Out-Null } catch { $failed = $true }; Assert $failed 'Expected rejection' }
function BytesEqual($First, $Second) { return [Convert]::ToBase64String($First) -ceq [Convert]::ToBase64String($Second) }
$temp = Join-Path ([IO.Path]::GetTempPath()) ('terminalhud-profile-test-' + [Guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($temp)
try {
    $root = Join-Path $temp "user's install"
    foreach ($shell in @('PowerShell', 'WSLBash', 'GitBash')) {
        $snippet = New-TerminalHUDSnippet $shell $root "/c/Users/operator's dir/install"
        foreach ($encoding in @((New-Object Text.UTF8Encoding($false)), (New-Object Text.UTF8Encoding($true)), (New-Object Text.UnicodeEncoding($false, $true)))) {
            $path = Join-Path $temp ([Guid]::NewGuid().ToString('N') + '.profile')
            [IO.File]::WriteAllText($path, "# custom caf$([char]0xe9)`r`nunaltered without final newline", $encoding)
            $original = [IO.File]::ReadAllBytes($path)
            $install = New-TerminalHUDProfilePlan $path $snippet Install
            Assert (BytesEqual $original ([IO.File]::ReadAllBytes($path))) 'Preview changed profile'
            Assert (!(Test-Path -LiteralPath $install.backup)) 'Preview wrote backup'
            Invoke-TerminalHUDProfilePlan $install
            Assert (BytesEqual $original ([IO.File]::ReadAllBytes($install.backup))) 'Backup did not preserve original bytes'
            Assert (!(New-TerminalHUDProfilePlan $path $snippet Install).changed) 'Repeated install changed profile'
            # User edits outside the marker survive removal, including no final newline.
            $current = Read-TerminalHUDProfile $path
            [IO.File]::WriteAllBytes($path, $current.encoding.GetBytes($current.text + '# later user edit'))
            $remove = New-TerminalHUDProfilePlan $path $snippet Remove
            Invoke-TerminalHUDProfilePlan $remove
            $expected = New-Object byte[] ($original.Length + $current.encoding.GetByteCount('# later user edit'))
            [Array]::Copy($original, $expected, $original.Length)
            [Array]::Copy($current.encoding.GetBytes('# later user edit'), 0, $expected, $original.Length, $current.encoding.GetByteCount('# later user edit'))
            Assert (BytesEqual $expected ([IO.File]::ReadAllBytes($path))) 'Removal changed unrelated bytes'
            Assert (!(New-TerminalHUDProfilePlan $path $snippet Remove).changed) 'Repeated removal changed profile'
        }
    }
    $path = Join-Path $temp 'new.profile'
    $snippet = New-TerminalHUDSnippet PowerShell $root ''
    $install = New-TerminalHUDProfilePlan $path $snippet Install
    Assert (!(Test-Path -LiteralPath $path)) 'Preview created a profile'
    Invoke-TerminalHUDProfilePlan $install
    [IO.File]::AppendAllText($path, "`n# >>> TerminalHUD prompt Escape >>>")
    Reject { New-TerminalHUDProfilePlan $path $snippet Remove }
    [IO.File]::WriteAllText($path, 'changed after plan')
    Reject { Invoke-TerminalHUDProfilePlan $install }
    Reject { New-TerminalHUDSnippet GitBash $root "relative/path" }
    $null = [IO.Directory]::CreateDirectory($root)
    function Write-TestReceipt($Build) {
        [IO.File]::WriteAllText((Join-Path $root 'receipt.json'), (@{ schema_version = 1; product = 'TerminalHUD'; installed = $true; active_build = $Build } | ConvertTo-Json))
    }
    Write-TestReceipt 'build1'
    Assert ((Get-TerminalHUDActiveDirectory $root).EndsWith('versions\build1')) 'Active build did not resolve'
    Write-TestReceipt 'build2'
    Assert ((Get-TerminalHUDActiveDirectory $root).EndsWith('versions\build2')) 'Upgrade did not resolve new active build'
    Write-TestReceipt '../escape'
    Reject { Get-TerminalHUDActiveDirectory $root }
    $managed = Join-Path $temp 'managed.profile'
    $manager = Join-Path $PSScriptRoot '..\manage.ps1'
    & $manager -Operation Preview -Shell PowerShell -ProfilePath $managed -InstallRoot $root | Out-Null
    Assert (!(Test-Path -LiteralPath $managed)) 'Manager preview created profile'
    & $manager -Operation Install -Shell PowerShell -ProfilePath $managed -InstallRoot $root | Out-Null
    & $manager -Operation Install -Shell PowerShell -ProfilePath $managed -InstallRoot $root | Out-Null
    & $manager -Operation Remove -Shell PowerShell -ProfilePath $managed -InstallRoot $root | Out-Null
    Assert (([IO.File]::ReadAllBytes($managed)).Length -eq 0) 'Manager round trip did not restore empty profile'
    Write-Output 'Profile preview/backup/idempotence/encoding and receipt tests passed'
} finally { Remove-Item -LiteralPath $temp -Recurse -Force }
