function Get-TerminalHUDActiveDirectory {
    param([string] $InstallRoot)
    if (!$InstallRoot) {
        $local = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
        if (!$local) { throw 'Current-user Local App Data is unavailable' }
        $InstallRoot = Join-Path $local 'TerminalHUD\install'
    }
    $receiptPath = Join-Path $InstallRoot 'receipt.json'
    $item = Get-Item -LiteralPath $receiptPath -ErrorAction Stop
    if ($item.Length -gt 1MB) { throw 'Installation receipt too large' }
    $receipt = [IO.File]::ReadAllText($item.FullName) | ConvertFrom-Json
    if ($receipt.schema_version -ne 1 -or $receipt.product -cne 'TerminalHUD' -or
        $receipt.installed -ne $true -or $receipt.active_build -cnotmatch '^[A-Za-z0-9_-]{1,100}$') {
        throw 'TerminalHUD is not installed or its receipt is invalid'
    }
    return Join-Path $InstallRoot "versions\$($receipt.active_build)"
}
function Invoke-TerminalHUDPassive {
    param([string] $InstallRoot)
    try {
        $client = Join-Path (Get-TerminalHUDActiveDirectory $InstallRoot) 'terminalhud.exe'
        if (!(Test-Path -LiteralPath $client -PathType Leaf)) { return }
        # Avoid a visible console window and do not wait in the line editor.
        $start = New-Object Diagnostics.ProcessStartInfo
        $start.FileName = $client
        $start.Arguments = 'passive'
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $process = [Diagnostics.Process]::Start($start)
        if ($null -ne $process) { $process.Dispose() }
    } catch { } # Missing installation/owner is a prompt-safe no-op.
}
