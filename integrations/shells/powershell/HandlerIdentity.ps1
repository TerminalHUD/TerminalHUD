# PSReadLine's public KeyHandler metadata contains caller-controlled labels only.
# Read the actual Escape handler in the verified 2.0-2.4 layouts. Never write
# private state; unsupported layouts leave bindings untouched.
function Get-TerminalHUDEscapeIdentity {
    $failure = 'version'
    try {
        $type = [Microsoft.PowerShell.PSConsoleReadLine]
        $modules = @(Get-Module PSReadLine)
        if ($modules.Count -ne 1) { throw 'Ambiguous PSReadLine module' }
        $version = $modules[0].Version
        if ($version.Major -ne 2 -or $version.Minor -gt 4) { throw 'Unsupported PSReadLine version' }
        $failure = 'layout'
        $static = [Reflection.BindingFlags]'Static,Public,NonPublic'
        $instance = [Reflection.BindingFlags]'Instance,Public,NonPublic'
        $singletonField = $type.GetField('_singleton', $static)
        $dispatchField = $type.GetField('_dispatchTable', $instance)
        $keysType = $type.Assembly.GetType('Microsoft.PowerShell.Keys', $true)
        $escapeField = $keysType.GetField('Escape', $static)
        $handlerType = $type.GetNestedType('KeyHandler', [Reflection.BindingFlags]::NonPublic)
        $actionField = $handlerType.GetField('Action', $instance)
        $scriptField = $handlerType.GetField('ScriptBlock', $instance)
        if (($escapeField.FieldType -ne [ConsoleKeyInfo] -and
             ($escapeField.FieldType.FullName -cne 'Microsoft.PowerShell.PSKeyInfo' -or $escapeField.FieldType.Assembly -ne $type.Assembly)) -or
            $singletonField.FieldType -ne $type -or !$dispatchField.FieldType.IsGenericType -or
            $dispatchField.FieldType.GetGenericTypeDefinition().FullName -cne 'System.Collections.Generic.Dictionary`2' -or
            $dispatchField.FieldType.GetGenericArguments()[0] -ne $escapeField.FieldType -or
            $dispatchField.FieldType.GetGenericArguments()[1] -ne $handlerType -or
            $scriptField.FieldType -ne [scriptblock] -or
            $actionField.FieldType -ne [Action[Nullable[ConsoleKeyInfo],object]]) { throw 'Unsupported handler layout' }
        $owner = $singletonField.GetValue($null)
        $dispatch = $dispatchField.GetValue($owner)
        $escape = $escapeField.GetValue($null)
        $handler = if ($dispatch.ContainsKey($escape)) { $dispatch[$escape] } else { $null }
        $builtin = $null
        if ($null -ne $handler) {
            $action = $actionField.GetValue($handler)
            $block = $scriptField.GetValue($handler)
            if ($null -eq $block -and $action.GetInvocationList().Length -eq 1 -and $null -eq $action.Target) {
                foreach ($name in @('RevertLine', 'Abort')) {
                    $method = $type.GetMethod($name, [type[]]@([Nullable[ConsoleKeyInfo]], [object]))
                    if ($action.Method.Equals($method)) { $builtin = $name; break }
                }
            }
        }
        return [pscustomobject]@{ supported = $true; handler = $handler; builtin = $builtin }
    } catch {
        return [pscustomobject]@{ supported = $false; handler = $null; builtin = $null; failure = $failure }
    }
}
