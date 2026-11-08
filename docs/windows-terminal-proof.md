# Controlled Windows Terminal portable proof

A separate, official unpackaged ZIP in a fresh task directory can avoid trusting
an installed process's cached profile catalog. Microsoft's [portable-mode
contract](https://learn.microsoft.com/en-us/windows/terminal/distributions)
keeps its settings beside the executable. The pinned
[FileUtils implementation](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/FileUtils.cpp#L20-L39)
requires an unpackaged executable plus adjacent `.portable` marker. The pinned
[WindowEmperor](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/WindowEmperor.cpp#L309-L358)
includes the unpackaged executable path in remoting identity, separating a fresh
path from the installed distribution's running server.

This is a fixture precondition, not a public runtime-profile recognition API.
Before running it, verify the exact official asset's release metadata, complete
ZIP SHA256, and Microsoft Authenticode signature. Use exclusive task directories,
reject unsafe archive paths/reparse entries, and create only adjacent synthetic
settings. The fixture profile selects its explicit command; both the profile and
profile defaults select `elevate:false` and zero-GUID `connectionType`. Set empty
`startupActions` and `firstWindowPreference:defaultProfile`; pass the fixture
command explicitly on the CLI. Never select an ordinary user shell as fallback.

Portable settings still read external fragments. The pinned
[loader](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L330-L445)
searches shared fragment directories and app extensions. The proof's read-only
`sources` guard enumerates those namespaces and rejects unknown ones or discovery
errors before launch. Disable all observed namespaces and built-in dynamic
sources in the synthetic settings; repeat discovery afterward and reject changed
observations. This does not protect against a hostile concurrent configuration
change and is not a security boundary. No normal settings, fragments, aliases,
registry entries, running sessions, or installed executables may be changed.

The foreground monitor installs an out-of-context foreground WinEvent hook on a
short-lived message-pumping thread before creation. It records intermediate
foreground HWNDs, rather than relying only on the final foreground after launch.
Stop/join it after the bounded launch observation. Only the probe owns this
observer; the production backend retains the caller-owned event-loop contract.

Portable rendering evidence would apply to that exact same-version fixture.
It cannot establish installed Stable fragment-cache recognition, Preview behavior,
or complete ordinary-profile cloning. Those require separate evidence.

Terminal can normalize and rewrite its own synthetic settings, including adding
inbox profiles. Its pinned [loader](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L849-L857)
clears commandline/name/source/GUID from profile defaults. A default command line
is not a launch-safety guarantee; use the explicit CLI fixture command with known
non-elevated ConPTY defaults. Preserve normalized bytes as evidence; reusing the
fixture requires reviewing/restoring only owned configuration and stale markers.

After preparing the reviewed isolated configuration, build the native example
and explicitly run `terminal-proof probe <absolute-task-directory>`. It briefly
shows an owned blue primary-monitor sink and the dedicated test Terminal, and
sends one guarded click only inside that sink. The probe rejects unknown sources,
stale handshake/exit markers, and unsafe startup/profile settings. It never
installs a global fragment. Native CI compiles it but does not run desktop proof.
