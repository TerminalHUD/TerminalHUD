# Invoked by Bash through Windows interoperability, never by a global hook.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Client.ps1')
Invoke-TerminalHUDPassive
