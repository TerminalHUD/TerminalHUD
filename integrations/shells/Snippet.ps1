function New-TerminalHUDSnippet {
    param([string] $Shell, [string] $InstallRoot, [string] $BashInstallRoot)
    if ($Shell -eq 'PowerShell') {
        $literal = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($InstallRoot))
        return @'
try {
    $__thRoot = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('__ROOT__'))
    $__thReceipt = Get-Content -LiteralPath (Join-Path $__thRoot 'receipt.json') -Raw | ConvertFrom-Json
    if ($__thReceipt.schema_version -eq 1 -and $__thReceipt.product -ceq 'TerminalHUD' -and
        $__thReceipt.installed -eq $true -and $__thReceipt.active_build -cmatch '^[A-Za-z0-9_-]{1,100}$') {
        Import-Module (Join-Path $__thRoot "versions\$($__thReceipt.active_build)\integrations\shells\powershell\TerminalHUD.psm1")
    }
} catch { Write-Warning 'TerminalHUD: integration unavailable; use Win+Backtick.' }
'@.Replace('__ROOT__', $literal)
    }
    if (!$BashInstallRoot -or !$BashInstallRoot.StartsWith('/') -or $BashInstallRoot.Contains("`n") -or $BashInstallRoot.Contains("`r")) {
        throw '-BashInstallRoot must name the shell-visible absolute installation directory'
    }
    $literal = $BashInstallRoot.Replace("'", "'\''")
    # The installer writes a canonical indented JSON receipt. Read only its single
    # active_build field; the restricted alphabet is never interpreted as code.
    return @'
__th_root='__ROOT__'
__th_build=$(sed -n 's/^[[:space:]]*"active_build":[[:space:]]*"\([A-Za-z0-9_-]\{1,100\}\)",[[:space:]]*$/\1/p' "$__th_root/receipt.json" 2>/dev/null)
if [[ $__th_build =~ ^[A-Za-z0-9_-]{1,100}$ && -f "$__th_root/versions/$__th_build/integrations/shells/bash/terminalhud.bash" ]]; then
    source "$__th_root/versions/$__th_build/integrations/shells/bash/terminalhud.bash"
else
    printf '%s\n' 'TerminalHUD: integration unavailable; use Win+Backtick.' >&2
fi
unset __th_root __th_build
'@.Replace('__ROOT__', $literal)
}
