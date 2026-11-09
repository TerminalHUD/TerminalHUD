Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-SafeTree([string] $Path) {
    $itemPath = [IO.Path]::GetFullPath($Path)
    while ($itemPath) {
        if (Test-Path -LiteralPath $itemPath) {
            $item = Get-Item -LiteralPath $itemPath -Force
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point refused: $itemPath" }
        }
        $parent = [IO.Path]::GetDirectoryName($itemPath)
        if ($parent -eq $itemPath) { break }
        $itemPath = $parent
    }
}
function Assert-BuildId([string] $BuildId) {
    if ($BuildId -cnotmatch '^[A-Za-z0-9_-]{1,100}$' -or $BuildId -match '^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$') { throw 'Unsafe build identity' }
}
function Resolve-PayloadPath([string] $Root, [string] $Relative) {
    if (!$Relative -or $Relative.Length -gt 240 -or $Relative.Contains('\') -or $Relative.Contains(':')) { throw "Unsafe payload path: $Relative" }
    foreach ($part in $Relative.Split('/')) {
        if ($part -notmatch '^[A-Za-z0-9_.][A-Za-z0-9_. +()-]*$' -or $part -eq '.' -or $part -eq '..' -or $part.EndsWith('.') -or $part.EndsWith(' ') -or
            $part -match '^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)') { throw "Unsafe payload path: $Relative" }
    }
    $path = Join-Path $Root ($Relative.Replace('/', [IO.Path]::DirectorySeparatorChar))
    Assert-SafeTree $path
    return $path
}
function Get-InstallRoot {
    $local = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
    if (!$local -or ![IO.Path]::IsPathRooted($local)) { throw 'Cannot resolve current-user Local App Data' }
    $root = Join-Path $local 'TerminalHUD\install'
    Assert-SafeTree $root
    return $root
}
function Get-FileDigest([string] $Path) {
    Assert-SafeTree $Path
    if (!(Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Regular file required: $Path" }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}
